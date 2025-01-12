/// process/manager.rs
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use log::{info, error, warn};
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::time::sleep;
use tokio::sync::mpsc;
use std::process::Stdio;

use crate::state::ProcessState;
use crate::workspace::Workspace;
use crate::types::ProcessConfig;

pub struct ProcessManager<'a> {
    pub state: ProcessState,
    pub config: ProcessConfig,
    pub workspace: &'a Workspace,
    pub process_name: String,
    pub daemon_mode: bool,
    pub auto_restart: bool,
}

impl<'a> ProcessManager<'a> {
    pub fn new(workspace: &'a Workspace, process_name: String) -> Self {
        Self {
            state: ProcessState::default(),
            config: ProcessConfig::default(),
            workspace,
            process_name,
            daemon_mode: false,
            auto_restart: false,
        }
    }

    pub fn with_config(workspace: &'a Workspace, process_name: String, config: ProcessConfig) -> Self {
        Self {
            state: ProcessState::default(),
            config,
            workspace,
            process_name,
            daemon_mode: false,
            auto_restart: false,
        }
    }

    pub fn set_daemon_mode(&mut self, enabled: bool) {
        self.daemon_mode = enabled;
    }

    pub fn set_auto_restart(&mut self, enabled: bool) {
        self.auto_restart = enabled;
    }

    pub fn get_config(&self) -> &ProcessConfig {
        &self.config
    }

    /// 保存进程状态(给守护进程、重启等场景用)
    pub async fn save_process_state(
        &self,
        pid: i32,
        program: &str,
        args: &[String],
        working_dir: &PathBuf,
        health_check_url: Option<&str>,
    ) -> Result<()> {
        let mut state = ProcessState::default();
        state.pid = Some(pid);
        state.program = program.to_string();
        state.args = args.to_vec();
        state.working_dir = working_dir.clone();
        state.port = self.config.default_port;
        state.health_check_url = health_check_url.map(String::from);
        state.save(self.workspace, &self.process_name)?;
        info!("进程状态已保存");
        Ok(())
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

        if self.status(health_check_url).await? {
            error!("进程已在运行");
            return Err(anyhow::anyhow!("进程已在运行,如需重启请先停止进程"));
        }

        info!("程序: {}", program);
        info!("参数: {:?}", args);
        info!("工作目录: {:?}", working_dir);

        if self.daemon_mode {
            use super::daemon::ProcessManagerDaemonExt;
            return self.start_daemon(program, args, working_dir, health_check_url, env_vars).await;
        }

        fs::create_dir_all(working_dir).context("创建工作目录失败")?;
        info!("工作目录已创建/确认");

        self.workspace.ensure_process_dirs(&self.process_name)?;

        let mut cmd = Command::new(program);
        cmd.args(args)
           .current_dir(working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .stdin(Stdio::null()) // ☆ 保证非Daemon时也不阻塞主进程
           .kill_on_drop(false);

        #[cfg(windows)]
        cmd.creation_flags(self.config.windows_process_flags);

        if let Some(vars) = env_vars {
            for (key, value) in vars {
                cmd.env(key, value);
            }
            info!("已设置环境变量: {:?}", vars);
        }

        info!("正在启动进程...");
        let mut child = cmd.spawn().context("启动进程失败")?;
        let pid = child.id().unwrap();
        info!("进程已启动, PID: {}", pid);

        let mut state = ProcessState::default();
        state.pid = Some(pid as i32);
        state.program = program.to_string();
        state.args = args.to_vec();
        state.working_dir = working_dir.clone();
        state.port = self.config.default_port;
        state.health_check_url = health_check_url.map(String::from);
        state.save(self.workspace, &self.process_name)?;
        info!("进程状态已保存");

        let (tx, mut rx) = mpsc::channel(1);

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

    pub async fn stop(&self) -> Result<()> {
        info!("开始停止进程");

        let state = ProcessState::load(self.workspace, &self.process_name)
            .context("加载进程状态失败")?;

        let port = state.port;
        info!("当前进程使用端口: {}", port);

        // 先停止用户进程
        if let Some(pid) = state.pid {
            info!("检查用户进程 {} 是否存在", pid);
            let exists = self.check_process_exists(pid).await?;
            if exists {
                info!("正在停止 PID 为 {} 的用户进程", pid);
                if !self.try_graceful_shutdown(pid).await {
                    info!("优雅终止失败，将强制终止进程");
                    self.force_shutdown(pid).await?;
                }
            } else {
                info!("用户进程 {} 已不存在", pid);
            }
        } else {
            info!("没有找到运行中的用户进程");
        }

        // 再停止monitor进程
        if let Some(monitor_pid) = state.monitor_pid {
            info!("检查监控进程 {} 是否存在", monitor_pid);
            let exists = self.check_process_exists(monitor_pid).await?;
            if exists {
                info!("正在停止 PID 为 {} 的监控进程", monitor_pid);
                if !self.try_graceful_shutdown(monitor_pid).await {
                    info!("优雅终止失败，将强制终止监控进程");
                    self.force_shutdown(monitor_pid).await?;
                }
            } else {
                info!("监控进程 {} 已不存在,尝试查找实际运行的fuckrun进程", monitor_pid);
                // 查找实际运行的fuckrun进程
                let output = Command::new("tasklist")
                    .args(["/FI", "IMAGENAME eq fuckrun.exe", "/NH", "/FO", "CSV"])
                    .output()
                    .await
                    .context("查找fuckrun进程失败")?;

                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if let Some(pid_str) = line.split(',').nth(1) {
                        if let Ok(pid) = pid_str.trim_matches('"').parse::<i32>() {
                            info!("找到运行中的fuckrun进程: {}", pid);
                            if !self.try_graceful_shutdown(pid).await {
                                info!("优雅终止失败，将强制终止监控进程");
                                self.force_shutdown(pid).await?;
                            }
                            break;
                        }
                    }
                }
            }
        } else {
            info!("没有找到运行中的监控进程");
        }

        self.cleanup_port(port).await?;

        let mut state = state;
        state.update_stopped_state();
        state.save(self.workspace, &self.process_name)
            .context("更新进程状态失败")?;

        info!("进程状态已更新为停止");
        info!("进程停止操作完成");
        Ok(())
    }

    /// 检查进程是否存在
    async fn check_process_exists(&self, pid: i32) -> Result<bool> {
        #[cfg(windows)]
        {
            let output = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                .output()
                .await
                .context("检查进程状态失败")?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(!output_str.is_empty() && !output_str.contains("No tasks"))
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            Ok(kill(Pid::from_raw(pid), Signal::SIGZERO).is_ok())
        }
    }

    pub async fn status(&self, health_check_url: Option<&str>) -> Result<bool> {
        info!("检查进程状态");

        match ProcessState::load(self.workspace, &self.process_name) {
            Ok(state) => {
                if let Some(pid) = state.pid {
                    info!("检查PID为{}的进程", pid);
                    #[cfg(windows)]
                    {
                        let output = Command::new("tasklist")
                            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                            .output()
                            .await
                            .context("检查进程状态失败")?;

                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if !output_str.is_empty() && !output_str.contains("No tasks") {
                            info!("进程正在运行");
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
        let mut state = ProcessState::load(self.workspace, &self.process_name)?;
        state.pid = None;
        state.save(self.workspace, &self.process_name)?;
        Ok(())
    }

    async fn try_graceful_shutdown(&self, pid: i32) -> bool {
        info!("尝试优雅终止进程 {}", pid);

        #[cfg(windows)]
        {
            info!("Windows 平台：使用 taskkill /PID 发送终止信号");
            match Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output()
                .await
            {
                Ok(output) => {
                    if output.status.success() {
                        info!("已发送终止信号到进程 {}", pid);
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
            match nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGTERM,
            ) {
                Ok(()) => {
                    info!("已发送 SIGTERM 信号到进程 {}", pid);
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
                warn!("强制终止进程失败: {}", error);
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

        tokio::time::sleep(self.config.exit_wait()).await;
        info!("进程 {} 已强制终止", pid);

        Ok(())
    }

    async fn cleanup_port(&self, port: u16) -> Result<()> {
        info!("开始清理端口 {} 的所有进程", port);

        #[cfg(windows)]
        {
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

    pub async fn handle_process_output(
        process_name: String,
        logger: crate::logger::Logger,
        mut stdout: Option<tokio::process::ChildStdout>,
        mut stderr: Option<tokio::process::ChildStderr>,
    ) {
        if let Some(stdout) = stdout.take() {
            let process_name = process_name.clone();
            let logger = logger.clone();
            tokio::spawn(async move {
                use tokio::io::{BufReader, AsyncBufReadExt};
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Err(e) = logger.write_stdout(&process_name, &line) {
                        error!("写入标准输出日志失败: {}", e);
                    }
                }
            });
        }

        if let Some(stderr) = stderr.take() {
            let process_name = process_name.clone();
            let logger = logger.clone();
            tokio::spawn(async move {
                use tokio::io::{BufReader, AsyncBufReadExt};
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Err(e) = logger.write_stderr(&process_name, &line) {
                        error!("写入标准错误日志失败: {}", e);
                    }
                }
            });
        }
    }
}
