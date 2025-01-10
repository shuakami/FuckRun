mod process;
mod state;
mod fs;
mod config;

use clap::{Parser, Subcommand};
use anyhow::Result;
use log::{info, error};
use std::path::PathBuf;
use crate::process::ProcessManager;
use crate::config::Config;
use tokio::signal;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动进程
    Start {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,
        
        /// Python解释器路径
        #[arg(long, default_value = "python")]
        python: String,
        
        /// 端口号
        #[arg(short, long, default_value_t = 5000)]
        port: u16,
    },
    /// 停止进程
    Stop {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,
    },
    /// 检查进程状态
    Status {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,
        
        /// Python解释器路径
        #[arg(long, default_value = "python")]
        python: String,
        
        /// 端口号
        #[arg(short, long, default_value_t = 5000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    // 加载配置文件
    let config = if let Some(config_path) = cli.config {
        info!("正在加载配置文件: {:?}", config_path);
        let cfg = Config::from_file(config_path)?;
        info!("配置文件加载成功");
        cfg
    } else {
        // 尝试加载默认配置文件
        let default_paths = ["config.yaml", "config.json"];
        let mut config = None;
        for path in default_paths.iter() {
            info!("尝试加载默认配置文件: {}", path);
            if let Ok(cfg) = Config::from_file(path) {
                config = Some(cfg);
                info!("默认配置文件加载成功: {}", path);
                break;
            }
        }
        config.unwrap_or_default()
    };

    // 设置Ctrl+C处理
    let ctrl_c = async {
        signal::ctrl_c().await.expect("无法监听Ctrl+C信号");
        info!("收到Ctrl+C信号，主进程退出");
        Ok(())
    };

    // 主逻辑
    let work = async {
        match cli.command {
            Commands::Start { name, python, port } => {
                if let Some(name) = name {
                    info!("正在启动进程: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::new();
                            
                            // 合并全局和进程环境变量
                            let mut env_vars = config.global.env.clone();
                            env_vars.extend(process_config.env.clone());
                            
                            manager.start(
                                &process_config.program,
                                &process_config.args,
                                &process_config.working_dir,
                                process_config.health_check_url.as_deref(),
                                Some(&env_vars),
                            ).await?;
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    // 使用命令行参数
                    info!("使用命令行参数启动进程");
                    let manager = ProcessManager::new();
                    manager.start(
                        &python,
                        &["examples/simple_web.py".to_string()],
                        &PathBuf::from("."),
                        Some(&format!("http://localhost:{}/health", port)),
                        None,
                    ).await?;
                }
            }
            Commands::Stop { name } => {
                if let Some(name) = name {
                    info!("正在停止进程: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::new();
                            manager.stop().await?;
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    // 使用默认配置
                    info!("停止默认进程");
                    let manager = ProcessManager::new();
                    manager.stop().await?;
                }
            }
            Commands::Status { name, python: _, port } => {
                if let Some(name) = name {
                    info!("检查进程状态: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::new();
                            let status = manager.status(
                                process_config.health_check_url.as_deref()
                            ).await?;
                            info!("进程状态: {}", if status { "运行中" } else { "未运行" });
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    // 使用命令行参数
                    info!("检查默认进程状态");
                    let manager = ProcessManager::new();
                    let status = manager.status(
                        Some(&format!("http://localhost:{}/health", port))
                    ).await?;
                    info!("进程状态: {}", if status { "运行中" } else { "未运行" });
                }
            }
        }
        Ok(())
    };

    // 等待任意一个完成
    tokio::select! {
        result = work => result,
        result = ctrl_c => result,
    }
}
