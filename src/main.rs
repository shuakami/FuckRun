mod process;
mod commands;
mod cli;
mod config;
mod workspace;
mod state;
mod logger;
mod fs;
mod types;

use anyhow::{Result, Context};
use log::{info, warn, error};
use clap::Parser;
use tokio::signal;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::workspace::Workspace;
use crate::logger::Logger;
use crate::commands::start::handle_start;
use crate::commands::stop::handle_stop;
use crate::commands::status::handle_status;
use crate::commands::monitor::handle_monitor;
use crate::commands::logs::handle_logs;
use crate::commands::list::{handle_list, ListFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 初始化工作区
    let mut workspace = Workspace::new(cli.workspace.unwrap_or_else(|| std::path::PathBuf::from(".")));
    workspace.init()?;
    
    // 如果是 list 命令,直接执行不需要加载配置
    if let Commands::List { 
        app_dir,
        name,
        status,
        min_uptime,
        max_uptime,
        min_cpu,
        max_cpu,
        min_mem,
        max_mem,
        json,
        watch,
    } = cli.command {
        let filter = ListFilter {
            name,
            status,
            min_uptime,
            max_uptime,
            min_cpu,
            max_cpu,
            min_mem: min_mem.map(|m| m * 1024 * 1024),  // 转换MB为字节
            max_mem: max_mem.map(|m| m * 1024 * 1024),  // 转换MB为字节
        };
        return handle_list(&workspace, app_dir, Some(filter), json, watch).await;
    }

    // 如果指定了app_dir，更新workspace的app_dir
    match &cli.command {
        Commands::Start { app_dir: Some(app_dir), .. } |
        Commands::Stop { app_dir: Some(app_dir), .. } |
        Commands::Status { app_dir: Some(app_dir), .. } |
        Commands::Logs { app_dir: Some(app_dir), .. } => {
            workspace.set_app_dir(app_dir);
        }
        _ => {}
    }

    // 初始化日志系统
    let logger = Logger::new(workspace.clone());
    logger.init()?;
    
    info!("程序启动");

    // 检查是否是Monitor命令
    let is_monitor = matches!(cli.command, Commands::Monitor { .. });

    // 加载配置文件(Monitor命令不需要全局配置)
    let config = if is_monitor {
        Config::default()
    } else if let Some(config_path) = cli.config {
        info!("正在加载指定的配置文件: {:?}", config_path);
        Config::from_file(config_path)?
    } else {
        // 根据进程名自动查找配置文件
        let process_name = match &cli.command {
            Commands::Start { name, .. } |
            Commands::Stop { name, .. } |
            Commands::Status { name, .. } => name.as_deref(),
            Commands::Logs { name, .. } => Some(name.as_str()),
            _ => None
        };
        
        let config_path = Config::find_config_file(process_name, &workspace);
        info!("使用配置文件: {:?}", config_path);
        Config::from_file(config_path)?
    };

    // 设置Ctrl+C处理
    let ctrl_c = async {
        signal::ctrl_c().await.expect("无法监听Ctrl+C信号");
        info!("收到Ctrl+C信号");
        Ok::<(), anyhow::Error>(())
    };

    // 主逻辑
    let work = async {
        match cli.command {
            Commands::Start { name, python, port, detach, daemon, auto_restart, app_dir: _ } => {
                let name_for_status = name.clone();
                if let Some(n) = &name_for_status {
                    println!("STATUS:STARTING:{}", n);
                }
                commands::handle_start(
                    &workspace,
                    &config,
                    name,
                    Some(python.to_string()),
                    port,
                    detach,
                    daemon,
                    auto_restart,
                ).await?;
                
                if let Some(n) = &name_for_status {
                    if let Ok(state) = crate::state::ProcessState::load(&workspace, n) {
                        if let Some(pid) = state.pid {
                            println!("STATUS:STARTED:{}:{}", n, pid);
                        }
                    }
                }
                Ok(())
            }
            Commands::Stop { name, app_dir: _ } => {
                let name_for_status = name.clone();
                if let Some(n) = &name_for_status {
                    println!("STATUS:STOPPING:{}", n);
                }
                commands::handle_stop(
                    &workspace,
                    &config,
                    name,
                ).await?;
                
                if let Some(n) = &name_for_status {
                    println!("STATUS:STOPPED:{}", n);
                }
                Ok(())
            }
            Commands::Status { name, python: _, port, app_dir: _ } => {
                commands::handle_status(
                    &workspace,
                    &config,
                    name.clone(),
                    port,
                ).await?;
                
                if let Some(n) = &name {
                    if let Ok(state) = crate::state::ProcessState::load(&workspace, n) {
                        if let Some(pid) = state.pid {
                            println!("STATUS:RUNNING:{}:{}", n, pid);
                        } else {
                            println!("STATUS:STOPPED:{}", n);
                        }
                    }
                }
                Ok(())
            }
            Commands::Monitor { 
                process_name, 
                program, 
                mut args, 
                working_dir,
                env_vars,
                auto_restart,
                config,
                host,
                port,
            } => {
                // 获取配置文件的绝对路径
                let config = std::fs::canonicalize(&config)
                    .context(format!("无法获取配置文件的绝对路径: {:?}", config))?;
                info!("配置文件绝对路径: {:?}", config);

                // 加载配置
                info!("正在加载配置文件...");
                let monitor_config = Config::from_file(&config)?;
                info!("配置文件加载成功");

                // 合并host和port参数
                if let Some(host) = host {
                    args.extend_from_slice(&["--host".to_string(), host]);
                }
                if let Some(port) = port {
                    args.extend_from_slice(&["--port".to_string(), port.to_string()]);
                }

                // 启动监控
                handle_monitor(
                    &workspace,
                    &monitor_config,
                    process_name,
                    program,
                    args,
                    working_dir,
                    env_vars,
                    auto_restart,
                    logger,
                ).await
            }
            Commands::Logs { name, follow, log_type, date, app_dir: _ } => {
                commands::handle_logs(
                    &workspace,
                    name.clone(),
                    follow,
                    log_type,
                    date,
                ).await
            }
            Commands::SystemLogs { follow, date } => {
                commands::handle_system_logs(
                    &workspace,
                    follow,
                    date,
                ).await
            }
            Commands::List { .. } => {
                // 已在前面处理
                Ok(())
            }
        }
    };

    // 等待工作完成或Ctrl+C
    tokio::select! {
        result = work => result,
        _ = ctrl_c => {
            info!("正在退出...");
            Ok(())
        }
    }
}
