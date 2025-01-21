/// process/daemon.rs
use anyhow::{Result, Context};
use log::{info, warn, error};
use tokio::time::sleep;
use tokio::process::Command;
use std::process::Stdio;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::signal::ctrl_c;
use crate::state::ProcessState;

use super::manager::ProcessManager;

pub trait ProcessManagerDaemonExt {
    /// 守护进程启动
    fn start_daemon(
        &self,
        program: &str,
        args: &[String],
        working_dir: &PathBuf,
        health_check_url: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;

    /// 监控进程并自动重启
    fn monitor_and_restart(&self, child: tokio::process::Child)
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;
}

impl ProcessManagerDaemonExt for ProcessManager<'_> {
    fn start_daemon(
        &self,
        program: &str,
        args: &[String],
        working_dir: &PathBuf,
        health_check_url: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let process_name = self.process_name.clone();
        let auto_restart = self.auto_restart;
        let program = if cfg!(windows) && program.contains("python.exe") {
            String::from("py")
        } else {
            program.to_string()
        };
        let args = args.to_vec();
        let working_dir = working_dir.clone();
        let health_check_url = health_check_url.map(String::from);
        let env_vars = env_vars.cloned();
        let workspace = self.workspace.clone();
        let state = self.state.clone();
        let config = self.config.clone();

        Box::pin(async move {
            info!("以守护进程方式启动...");

            // 设置Ctrl+C信号处理
            let process_name_clone = process_name.clone();
            let workspace_clone = workspace.clone();
            tokio::spawn(async move {
                if let Ok(()) = ctrl_c().await {
                    info!("收到Ctrl+C信号,准备停止进程");
                    if let Ok(mut state) = ProcessState::load(&workspace_clone, &process_name_clone) {
                        state.update_stopped_state();
                        if let Err(e) = state.save(&workspace_clone, &process_name_clone) {
                            error!("保存进程状态失败: {}", e);
                        }
                    }
                    std::process::exit(0);
                }
            });

            #[cfg(windows)]
            {
                // Windows下启动一个独立的监控进程(monitor)
                let monitor_program = std::env::current_exe()?;
                
                // 使用传入的working_dir作为进程目录
                let process_dir = working_dir;
                info!("进程目录: {:?}", process_dir);
                
                // 获取配置文件的绝对路径
                let config_path = process_dir.join("config.yaml");
                info!("配置文件路径: {:?}", config_path);
                
                // 确保目录存在
                if !process_dir.exists() {
                    error!("进程目录不存在: {:?}", process_dir);
                    return Err(anyhow::anyhow!("进程目录不存在: {:?}", process_dir));
                }
                
                if !config_path.exists() {
                    error!("配置文件不存在: {:?}", config_path);
                    return Err(anyhow::anyhow!("配置文件不存在: {:?}", config_path));
                }
                
                let mut monitor_args = vec![
                    "monitor".to_string(),
                    "--process-name".to_string(),
                    process_name.clone(),
                    "--program".to_string(), 
                    program.clone(),
                    "--config".to_string(),
                    config_path.to_string_lossy().to_string(),
                ];
                
                // 修正Python脚本路径,使用绝对路径
                let mut args_iter = args.iter().peekable();
                while let Some(arg) = args_iter.next() {
                    if arg == "--host" || arg == "--port" {
                        // 处理参数对
                        monitor_args.push(arg.clone());
                        if let Some(value) = args_iter.next() {
                            monitor_args.push(value.clone());
                        }
                    } else if arg.ends_with(".py") {
                        let script_path = process_dir.join("app.py");
                        monitor_args.push("--args".to_string());
                        monitor_args.push(script_path.to_string_lossy().to_string());
                    } else {
                        monitor_args.push("--args".to_string());
                        monitor_args.push(arg.clone());
                    }
                }

                // 使用应用程序目录作为工作目录
                monitor_args.push("--working-dir".to_string());
                monitor_args.push(process_dir.to_string_lossy().to_string());

                if let Some(vars) = env_vars {
                    for (key, value) in vars {
                        info!("  {}={}", key, value);
                        monitor_args.push("--env".to_string());
                        monitor_args.push(format!("{}={}", key, value));
                    }
                }

                if auto_restart {
                    monitor_args.push("--auto-restart".to_string());
                }

                // 打印完整的monitor参数
                info!("监控进程启动参数:");
                for (i, arg) in monitor_args.iter().enumerate() {
                    info!("  参数 {}: {}", i, arg);
                }

                let mut monitor_cmd = Command::new(monitor_program);
                use winapi::um::winbase::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};
                monitor_cmd
                    .args(&monitor_args)
                    .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());

                info!("正在启动监控进程...");
                let child = monitor_cmd.spawn()?;
                let pid = child.id().unwrap();
                info!("Windows监控进程已启动, PID: {}", pid);

                // 保存monitor进程ID到状态文件
                let mut current_state = ProcessState::load(&workspace, &process_name)
                    .unwrap_or_else(|_| state.clone());
                current_state.monitor_pid = Some(pid as i32);
                current_state.save(&workspace, &process_name)?;

                // 等待进程初始化
                info!("等待进程初始化({:?}秒)...", config.init_wait_secs);
                sleep(std::time::Duration::from_secs(config.init_wait_secs)).await;

                // 检查监控进程是否存活
                let output = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                    .output()
                    .await
                    .context("检查监控进程状态失败")?;

                let output_str = String::from_utf8_lossy(&output.stdout);
                if !output_str.is_empty() && !output_str.contains("No tasks") {
                    info!("监控进程正在运行");
                    
                    // 进行健康检查
                    if let Some(url) = health_check_url {
                        info!("开始健康检查: {}", url);
                        let client = reqwest::Client::builder()
                            .user_agent("Mozilla/5.0")
                            .timeout(config.health_check_timeout())
                            .build()?;

                        for i in 0..config.health_check_retries {
                            info!("第{}次健康检查...", i + 1);
                            match client.get(&url).send().await {
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

                            if i < config.health_check_retries - 1 {
                                info!("等待{:?}后重试...", config.retry_interval());
                                sleep(config.retry_interval()).await;
                            }
                        }
                        return Err(anyhow::anyhow!("健康检查失败"));
                    }
                    return Ok(());
                } else {
                    return Err(anyhow::anyhow!("监控进程启动失败"));
                }
            }

            #[cfg(unix)]
            {
                use nix::unistd::{fork, ForkResult, setsid};
                use nix::sys::stat::Mode;
                // ☆ 去掉对 /dev/null 的多余 File::create 调用，改用 stdin 重定向：
                // 第一次fork
                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child }) => {
                        info!("守护进程第一次fork成功, 子进程PID: {}", child);
                        // 父进程直接返回
                        return Ok(());
                    }
                    Ok(ForkResult::Child) => {
                        setsid()?;
                        std::env::set_current_dir(&working_dir)?;
                        // 第二次fork
                        match unsafe { fork() } {
                            Ok(ForkResult::Parent { child }) => {
                                info!("守护进程第二次fork成功, 最终进程PID: {}", child);
                                std::process::exit(0);
                            }
                            Ok(ForkResult::Child) => {
                                // 设置umask
                                nix::sys::stat::umask(Mode::empty());

                                // 启动实际进程
                                let mut cmd = Command::new(&program);
                                cmd.args(&args)
                                   .current_dir(&working_dir)
                                   .stdout(Stdio::null())
                                   .stderr(Stdio::null())
                                   .stdin(Stdio::null()); // 确保无阻塞地脱离终端

                                if let Some(vars) = env_vars {
                                    cmd.envs(vars);
                                }

                                let mut child = cmd.spawn()?;
                                let pid = child.id().unwrap() as i32;

                                // 保存进程状态
                                let mut state = state.clone();
                                state.pid = Some(pid);
                                state.program = program.clone();
                                state.args = args.clone();
                                state.working_dir = working_dir.clone();
                                state.health_check_url = health_check_url.clone();
                                state.save(&workspace, &process_name)?;

                                if auto_restart {
                                    let monitor = ProcessManager {
                                        state,
                                        workspace: workspace.clone(),
                                        process_name: process_name.clone(),
                                        auto_restart,
                                        ..Default::default()
                                    };
                                    monitor.monitor_and_restart(child).await?;
                                }

                                Ok(())
                            }
                            Err(e) => Err(anyhow::anyhow!("Second fork failed: {}", e)),
                        }
                    }
                    Err(e) => Err(anyhow::anyhow!("First fork failed: {}", e)),
                }
            }
        })
    }

    fn monitor_and_restart(
        &self,
        mut child: tokio::process::Child
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let process_name = self.process_name.clone();
        let auto_restart = self.auto_restart;
        let workspace = self.workspace.clone();
        let state = self.state.clone();

        Box::pin(async move {
            if let Some(pid) = child.id() {
                info!("保存初始进程状态, PID: {}", pid);
                // 先加载现有状态
                let mut current_state = ProcessState::load(&workspace, &process_name)
                    .unwrap_or_else(|_| state.clone());
                info!("当前状态: {:?}", current_state);
                // 只更新pid,保持monitor_pid不变
                current_state.pid = Some(pid as i32);
                info!("更新后状态: {:?}", current_state);
                current_state.save(&workspace, &process_name)?;
            }

            let logger = crate::logger::Logger::new(workspace.clone());
            ProcessManager::handle_process_output(
                process_name.clone(),
                logger.clone(),
                child.stdout.take(),
                child.stderr.take()
            ).await;

            loop {
                match child.wait().await {
                    Ok(status) => {
                        if !status.success() && auto_restart {
                            warn!("进程异常退出({}), 准备重启...", status);
                            sleep(std::time::Duration::from_secs(3)).await;

                            // 加载完整的状态
                            let mut current_state = ProcessState::load(&workspace, &process_name)?;
                            
                            // 增加重启计数
                            current_state.restart_count += 1;
                            info!("进程重启次数: {}", current_state.restart_count);

                            // 获取python.exe的完整路径
                            let program = if cfg!(windows) {
                                which::which("python")
                                    .unwrap_or_else(|_| std::path::PathBuf::from("python"))
                                    .to_string_lossy()
                                    .into_owned()
                            } else {
                                current_state.program.clone()
                            };

                            info!("重启程序路径: {}", program);
                            info!("重启参数: {:?}", current_state.args);
                            info!("工作目录: {:?}", current_state.working_dir);

                            let mut cmd = Command::new(program);
                            cmd.args(&current_state.args)
                               .current_dir(&current_state.working_dir)
                               .stdout(Stdio::piped())
                               .stderr(Stdio::piped())
                               .stdin(Stdio::null());

                            #[cfg(windows)]
                            {
                                use winapi::um::winbase::CREATE_NO_WINDOW;
                                cmd.creation_flags(CREATE_NO_WINDOW);
                            }

                            match cmd.spawn() {
                                Ok(new_child) => {
                                    child = new_child;
                                    if let Some(pid) = child.id() {
                                        info!("进程已重启, 新PID: {}", pid);
                                        // 更新pid,保持重启计数
                                        current_state.pid = Some(pid as i32);
                                        // 不更新monitor_pid,保持原值
                                        info!("重启后更新状态: {:?}", current_state);
                                        current_state.save(&workspace, &process_name)?;

                                        ProcessManager::handle_process_output(
                                            process_name.clone(),
                                            logger.clone(),
                                            child.stdout.take(),
                                            child.stderr.take()
                                        ).await;
                                    }
                                }
                                Err(e) => {
                                    error!("重启进程失败: {}", e);
                                    return Err(e.into());
                                }
                            }
                        } else {
                            info!("进程正常退出: {}", status);
                            let mut current_state = ProcessState::load(&workspace, &process_name)?;
                            current_state.update_stopped_state();
                            current_state.save(&workspace, &process_name)?;
                            break;
                        }
                    }
                    Err(e) => {
                        error!("监控进程失败: {}", e);
                        return Err(e.into());
                    }
                }
            }
            Ok(())
        })
    }
}
