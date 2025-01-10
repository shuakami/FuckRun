use std::path::PathBuf;
use anyhow::{Result, Context};
use log::{info, error};
use tokio::process::Command;
use crate::state::ProcessState;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::process::Stdio;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::fs;
use std::os::windows::process::CommandExt;

const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
const DETACHED_PROCESS: u32 = 0x00000008;

pub struct ProcessManager {
    state: ProcessState,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            state: ProcessState::default(),
        }
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
        
        // 构建命令
        let mut cmd = Command::new(program);
        cmd.args(args)
           .current_dir(working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .kill_on_drop(false);  // 不随主进程退出而终止

        // Windows特定配置：创建新进程组并分离
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);

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
        state.health_check_url = health_check_url.map(String::from);
        state.save()?;
        info!("进程状态已保存");

        // 创建一个channel用于进程状态通知
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // 处理标准输出
        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            let mut reader = BufReader::new(stdout).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    info!("进程输出: {}", line);
                }
                let _ = tx.send(false).await;
            });
        }

        // 处理标准错误
        if let Some(stderr) = child.stderr.take() {
            let tx = tx.clone();
            let mut reader = BufReader::new(stderr).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    error!("进程错误: {}", line);
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
        info!("等待进程初始化(5秒)...");
        tokio::select! {
            _ = sleep(Duration::from_secs(5)) => {
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
                .timeout(Duration::from_secs(5))
                .build()?;
            
            for i in 0..10 {
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

                info!("等待2秒后重试...");
                tokio::select! {
                    _ = sleep(Duration::from_secs(2)) => {}
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
        info!("停止进程");
        
        // 加载进程状态
        let state = ProcessState::load().context("加载进程状态失败")?;
        let pid = match state.pid {
            Some(pid) => pid,
            None => {
                info!("没有运行中的进程");
                return Ok(());
            }
        };

        info!("正在停止PID为{}的进程", pid);

        // 尝试优雅终止进程
        if !self.try_graceful_shutdown(pid).await {
            // 如果优雅终止失败,则强制终止
            self.force_shutdown(pid).await?;
        }

        // 清除进程状态
        state.clear().context("清除进程状态失败")?;
        info!("进程状态已清除");
        
        Ok(())
    }

    async fn try_graceful_shutdown(&self, pid: i32) -> bool {
        #[cfg(windows)]
        {
            use std::process::Command;
            // Windows下使用CTRL_C_EVENT信号
            match Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output()
            {
                Ok(output) => output.status.success(),
                Err(_) => false
            }
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            // Unix下发送SIGTERM信号
            match kill(Pid::from_raw(pid), Signal::SIGTERM) {
                Ok(_) => {
                    // 等待进程响应SIGTERM信号
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    // 检查进程是否还在运行
                    kill(Pid::from_raw(pid), Signal::SIGZERO).is_err()
                }
                Err(_) => false
            }
        }
    }

    async fn force_shutdown(&self, pid: i32) -> Result<()> {
        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output()
                .context("强制终止进程失败")?;
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            kill(Pid::from_raw(pid), Signal::SIGKILL)
                .context("强制终止进程失败")?;
        }

        info!("进程已强制终止");
        Ok(())
    }

    pub async fn status(&self, health_check_url: Option<&str>) -> Result<bool> {
        info!("检查进程状态");
        
        // 加载进程状态
        match ProcessState::load() {
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
} 