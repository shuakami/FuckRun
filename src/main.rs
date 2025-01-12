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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 初始化工作区
    let workspace = Workspace::new(cli.workspace.unwrap_or_else(|| std::path::PathBuf::from(".")));
    workspace.init()?;
    
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
            Commands::Start { name, python, port, detach, daemon, auto_restart } => {
                commands::handle_start(
                    &workspace,
                    &config,
                    name,
                    python,
                    port,
                    detach,
                    daemon,
                    auto_restart,
                ).await?;
            }
            Commands::Stop { name } => {
                commands::handle_stop(
                    &workspace,
                    &config,
                    name,
                ).await?;
            }
            Commands::Status { name, python: _, port } => {
                commands::handle_status(
                    &workspace,
                    &config,
                    name,
                    port,
                ).await?;
            }
            Commands::Monitor { 
                process_name, 
                program, 
                args, 
                working_dir,
                env_vars,
                auto_restart,
                config,
            } => {
                info!("处理Monitor命令");
                info!("进程名称: {}", process_name);
                info!("程序: {}", program);
                info!("参数: {:?}", args);
                info!("工作目录: {:?}", working_dir);
                info!("环境变量: {:?}", env_vars);
                info!("自动重启: {}", auto_restart);
                info!("配置文件: {:?}", config);

                // 获取配置文件的绝对路径
                let config = std::fs::canonicalize(&config)
                    .context(format!("无法获取配置文件的绝对路径: {:?}", config))?;
                info!("配置文件绝对路径: {:?}", config);

                // 加载配置
                info!("正在加载配置文件...");
                let monitor_config = Config::from_file(&config)?;
                info!("配置文件加载成功");

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
                ).await?;
            }
            Commands::Logs { name, follow, log_type, date } => {
                commands::handle_logs(
                    &workspace,
                    name,
                    follow,
                    log_type,
                    date,
                ).await?;
            }
            Commands::SystemLogs { follow, date } => {
                commands::handle_system_logs(
                    &workspace,
                    follow,
                    date,
                ).await?;
            }
        }
        Ok(())
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
