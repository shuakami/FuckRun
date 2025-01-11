use std::path::PathBuf;
use anyhow::{Result, Context};
use log::{info, error, warn};
use tokio::process::Command;
use crate::state::ProcessState;
use crate::workspace::Workspace;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::process::Stdio;
use tokio::time::sleep;
use std::collections::HashMap;
use std::fs;
use crate::types::ProcessConfig;

pub struct ProcessManager<'a> {
    state: ProcessState,
    config: ProcessConfig,
    workspace: &'a Workspace,
    process_name: String,
}

impl<'a> ProcessManager<'a> {
    pub fn new(workspace: &'a Workspace, process_name: String) -> Self {
        Self {
            state: ProcessState::default(),
            config: ProcessConfig::default(),
            workspace,
            process_name,
        }
    }

    pub fn with_config(workspace: &'a Workspace, process_name: String, config: ProcessConfig) -> Self {
        Self {
            state: ProcessState::default(),
            config,
            workspace,
            process_name,
        }
    }

    /// 获取进程配置
    pub fn get_config(&self) -> &ProcessConfig {
        &self.config
    }

    pub async fn start(
        &self,
        program: &str,
        args: &[String],
        working_dir: &PathBuf,
        health_check_url: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> Result<()> {
        info!("准备启动进程...");
        info!("程序: {}", program);
        info!("参数: {:?}", args);
        info!("工作目录: {:?}", working_dir);
        
        // 确保工作目录存在
        fs::create_dir_all(working_dir).context("创建工作目录失败")?;
        info!("工作目录已创建/确认");
        
        // 确保进程目录结构存在
        self.workspace.ensure_process_dirs(&self.process_name)?;
        
        // 构建命令
        let mut cmd = Command::new(program);
        cmd.args(args)
           .current_dir(working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .kill_on_drop(false);  // 不随主进程退出而终止

        // Windows特定配置：创建新进程组并分离
        #[cfg(windows)]
        cmd.creation_flags(self.config.windows_process_flags);

        // 设置环境变量
        if let Some(vars) = env_vars {
            for (key, value) in vars {
                cmd.env(key, value);
            }
            info!("已设置环境变量: {:?}", vars);
        }
        
        info!("正在启动进程...");
        // 启动进程
        let mut child = cmd.spawn().context("启动进程失败")?;
        let pid = child.id().unwrap();
        info!("进程已启动, PID: {}", pid);

        // 保存进程状态
        let mut state = ProcessState::default();
        state.pid = Some(pid as i32);
        state.program = program.to_string();
        state.args = args.to_vec();
        state.working_dir = working_dir.clone();
        state.port = self.config.default_port;
        state.health_check_url = health_check_url.map(String::from);
        state.save(self.workspace, &self.process_name)?;
        info!("进程状态已保存");

        // 创建一个channel用于进程状态通知
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // 处理标准输出
        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            let mut reader = BufReader::new(stdout).lines();
            let log_dir = self.workspace.get_process_log_dir(&self.process_name);
            let stdout_log = log_dir.join("stdout.log");
            
            tokio::spawn(async move {
                use std::fs::OpenOptions;
                use std::io::Write;
                
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(stdout_log)
                    .unwrap_or_else(|_| panic!("无法打开标准输出日志文件"));
                
                while let Ok(Some(line)) = reader.next_line().await {
                    info!("[stdout] {}", line);
                    writeln!(file, "{}", line).unwrap_or_else(|_| error!("写入标准输出日志失败"));
                }
                let _ = tx.send(false).await;
            });
        }

        // 处理标准错误
        if let Some(stderr) = child.stderr.take() {
            let tx = tx.clone();
            let mut reader = BufReader::new(stderr).lines();
            let log_dir = self.workspace.get_process_log_dir(&self.process_name);
            let stderr_log = log_dir.join("stderr.log");
            
            tokio::spawn(async move {
                use std::fs::OpenOptions;
                use std::io::Write;
                
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(stderr_log)
                    .unwrap_or_else(|_| panic!("无法打开标准错误日志文件"));
                
                while let Ok(Some(line)) = reader.next_line().await {
                    info!("[stderr] {}", line);
                    writeln!(file, "{}", line).unwrap_or_else(|_| error!("写入标准错误日志失败"));
                }
                let _ = tx.send(false).await;
            });
        }

        // 监控进程状态
        let tx = tx.clone();
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        info!("进程正常退出: {:?}", status);
                    } else {
                        error!("进程异常退出: {:?}", status);
                    }
                }
                Err(e) => {
                    error!("监控进程失败: {}", e);
                }
            }
            let _ = tx.send(false).await;
        });

        // 等待进程初始化
        info!("等待进程初始化({:?})...", self.config.init_wait());
        tokio::select! {
            _ = sleep(self.config.init_wait()) => {
                info!("初始化等待完成");
            }
            Some(false) = rx.recv() => {
                error!("进程初始化失败");
                return Err(anyhow::anyhow!("进程初始化失败"));
            }
        }

        // 健康检查
        if let Some(url) = health_check_url {
            info!("开始健康检查: {}", url);
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0")
                .timeout(self.config.health_check_timeout())
                .build()?;
            
            for i in 0..self.config.health_check_retries {
                info!("第{}次健康检查...", i + 1);
                
                tokio::select! {
                    result = client.get(url).send() => {
                        match result {
                            Ok(response) => {
                                info!("收到响应: {}", response.status());
                                if response.status().is_success() {
                                    info!("健康检查通过，进程已在后台运行");
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                error!("健康检查失败: {}", e);
                            }
                        }
                    }
                    Some(false) = rx.recv() => {
                        error!("进程在健康检查过程中退出");
                        return Err(anyhow::anyhow!("进程在健康检查过程中退出"));
                    }
                }

                info!("等待{:?}后重试...", self.config.retry_interval());
                tokio::select! {
                    _ = sleep(self.config.retry_interval()) => {}
                    Some(false) = rx.recv() => {
                        error!("进程在等待过程中退出");
                        return Err(anyhow::anyhow!("进程在等待过程中退出"));
                    }
                }
            }
            
            error!("健康检查最终失败，终止进程");
            self.force_shutdown(pid as i32).await?;
            return Err(anyhow::anyhow!("进程健康检查失败"));
        }

        info!("进程启动成功，已在后台运行");
        Ok(())
    }

    /// 清理指定端口的所有进程
    async fn cleanup_port(&self, port: u16) -> Result<()> {
        info!("开始清理端口 {} 的所有进程", port);
        
        #[cfg(windows)]
        {
            // 使用 netstat 查找占用端口的进程
            let output = Command::new("netstat")
                .args(["-ano", "-p", "TCP"])
                .output()
                .await
                .context("执行 netstat 命令失败")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains(&format!(":{}", port)) {
                    if let Some(pid_str) = line.split_whitespace().last() {
                        if let Ok(pid) = pid_str.parse::<i32>() {
                            info!("发现占用端口 {} 的进程: {}", port, pid);
                            if let Err(e) = self.force_shutdown(pid).await {
                                warn!("终止进程 {} 失败: {}", pid, e);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(unix)]
        {
            // 使用 lsof 查找占用端口的进程
            let output = Command::new("lsof")
                .args(["-i", &format!(":{}", port), "-t"])
                .output()
                .await
                .context("执行 lsof 命令失败")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            for pid_str in output_str.lines() {
                if let Ok(pid) = pid_str.parse::<i32>() {
                    info!("发现占用端口 {} 的进程: {}", port, pid);
                    if let Err(e) = self.force_shutdown(pid).await {
                        warn!("终止进程 {} 失败: {}", pid, e);
                    }
                }
            }
        }

        info!("端口 {} 清理完成", port);
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("开始停止进程");
        
        // 加载进程状态
        let state = ProcessState::load(self.workspace, &self.process_name)
            .context("加载进程状态失败")?;
        
        // 记录当前端口
        let port = state.port;
        info!("当前进程使用端口: {}", port);

        let pid = match state.pid {
            Some(pid) => pid,
            None => {
                info!("没有找到运行中的进程 PID");
                // 即使没有主进程 PID，也尝试清理端口
                self.cleanup_port(port).await?;
                return Ok(());
            }
        };

        info!("正在停止 PID 为 {} 的主进程", pid);

        // 尝试优雅终止进程
        if !self.try_graceful_shutdown(pid).await {
            info!("优雅终止失败，将强制终止进程");
            // 如果优雅终止失败，则强制终止
            self.force_shutdown(pid).await?;
        }

        // 清理可能残留的占用相同端口的进程
        self.cleanup_port(port).await?;

        // 更新进程状态为已停止
        let mut state = state.clone();
        state.update_stopped_state();
        state.save(self.workspace, &self.process_name)
            .context("更新进程状态失败")?;
        
        info!("进程状态已更新为停止");
        info!("进程停止操作完成");
        Ok(())
    }

    async fn try_graceful_shutdown(&self, pid: i32) -> bool {
        info!("尝试优雅终止进程 {}", pid);
        
        #[cfg(windows)]
        {
            use std::process::Command;
            info!("Windows 平台：使用 taskkill 发送 CTRL_C_EVENT");
            // Windows下使用CTRL_C_EVENT信号
            match Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        info!("已发送终止信号到进程 {}", pid);
                        // 等待进程响应信号
                        tokio::time::sleep(self.config.graceful_shutdown_timeout()).await;
                        info!("优雅终止等待完成");
                        true
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        warn!("发送终止信号失败: {}", error);
                        false
                    }
                }
                Err(e) => {
                    warn!("执行 taskkill 命令失败: {}", e);
                    false
                }
            }
        }

        #[cfg(unix)]
        {
            info!("Unix 平台：发送 SIGTERM 信号");
            // Unix下使用SIGTERM信号
            match nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGTERM,
            ) {
                Ok(()) => {
                    info!("已发送 SIGTERM 信号到进程 {}", pid);
                    // 等待进程响应信号
                    tokio::time::sleep(self.config.graceful_shutdown_timeout()).await;
                    info!("优雅终止等待完成");
                    true
                }
                Err(e) => {
                    warn!("发送 SIGTERM 信号失败: {}", e);
                    false
                }
            }
        }
    }

    async fn force_shutdown(&self, pid: i32) -> Result<()> {
        info!("开始强制终止进程 {}", pid);
        
        #[cfg(windows)]
        {
            info!("Windows 平台：使用 taskkill /F 强制终止");
            let output = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output()
                .await
                .context("执行 taskkill 命令失败")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("强制终止失败: {}", error);
                return Err(anyhow::anyhow!("强制终止进程失败: {}", error));
            }
        }

        #[cfg(unix)]
        {
            info!("Unix 平台：发送 SIGKILL 信号");
            nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGKILL,
            ).context("发送 SIGKILL 信号失败")?;
        }

        // 等待一段时间确保进程完全退出
        tokio::time::sleep(self.config.exit_wait()).await;
        info!("进程 {} 已强制终止", pid);
        
        Ok(())
    }

    pub async fn status(&self, health_check_url: Option<&str>) -> Result<bool> {
        info!("检查进程状态");
        
        // 加载进程状态
        match ProcessState::load(self.workspace, &self.process_name) {
            Ok(state) => {
                if let Some(pid) = state.pid {
                    info!("检查PID为{}的进程", pid);
                    
                    // 检查进程是否存在
                    #[cfg(windows)]
                    {
                        use std::process::Command;
                        let output = Command::new("tasklist")
                            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                            .output()
                            .context("检查进程状态失败")?;
                        
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if !output_str.is_empty() {
                            info!("进程正在运行");
                            
                            // 如果提供了健康检查URL，则进行HTTP请求
                            if let Some(url) = health_check_url {
                                info!("检查健康状态: {}", url);
                                match reqwest::get(url).await {
                                    Ok(response) => {
                                        let is_healthy = response.status().is_success();
                                        if is_healthy {
                                            info!("进程健康检查通过");
                                        } else {
                                            error!("进程健康检查失败: {}", response.status());
                                        }
                                        return Ok(is_healthy);
                                    }
                                    Err(e) => {
                                        error!("健康检查失败: {}", e);
                                        return Ok(false);
                                    }
                                }
                            }
                            return Ok(true);
                        } else {
                            info!("进程未运行");
                        }
                    }

                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{kill, Signal};
                        use nix::unistd::Pid;
                        if kill(Pid::from_raw(pid), Signal::SIGZERO).is_ok() {
                            info!("进程正在运行");
                            
                            // 如果提供了健康检查URL，则进行HTTP请求
                            if let Some(url) = health_check_url {
                                info!("检查健康状态: {}", url);
                                match reqwest::get(url).await {
                                    Ok(response) => {
                                        let is_healthy = response.status().is_success();
                                        if is_healthy {
                                            info!("进程健康检查通过");
                                        } else {
                                            error!("进程健康检查失败: {}", response.status());
                                        }
                                        return Ok(is_healthy);
                                    }
                                    Err(e) => {
                                        error!("健康检查失败: {}", e);
                                        return Ok(false);
                                    }
                                }
                            }
                            return Ok(true);
                        } else {
                            info!("进程未运行");
                        }
                    }
                } else {
                    info!("没有找到进程PID");
                }
            }
            Err(e) => {
                error!("加载进程状态失败: {}", e);
            }
        }
        
        Ok(false)
    }

    pub async fn update_stopped_state(&self) -> Result<()> {
        // 读取当前状态
        let mut state = ProcessState::load(self.workspace, &self.process_name)?;
        
        // 更新状态
        state.pid = None;
        
        // 保存状态
        state.save(self.workspace, &self.process_name)?;
        
        Ok(())
    }
} 