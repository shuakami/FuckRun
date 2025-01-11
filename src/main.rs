mod process;
mod state;
mod fs;
mod config;
mod types;
mod workspace;
mod service;
mod daemon;

use clap::{Parser, Subcommand};
use anyhow::Result;
use log::{info, error, LevelFilter};
use std::path::PathBuf;
use crate::process::ProcessManager;
use crate::config::Config;
use crate::types::AppConfig;
use crate::workspace::Workspace;
use crate::service::{ServiceManager, ServiceConfig};
use tokio::signal;
use env_logger::Env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 工作目录路径
    #[arg(short = 'w', long, value_name = "DIR")]
    workspace: Option<PathBuf>,

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
        #[arg(long)]
        python: Option<String>,
        
        /// 端口号
        #[arg(short, long)]
        port: Option<u16>,

        /// 启动后立即退出主进程
        #[arg(short, long)]
        detach: bool,
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
        #[arg(long)]
        python: Option<String>,
        
        /// 端口号
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// 安装为系统服务
    Install {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,
    },
    /// 卸载系统服务
    Uninstall {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,
    },
    /// 以服务模式运行
    Service {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 设置默认的日志级别为 Info
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)  // 不显示时间戳
        .format_module_path(false)  // 不显示模块路径
        .format_target(false)  // 不显示目标
        .init();
    
    info!("程序启动");
    let cli = Cli::parse();
    
    // 初始化工作区
    let workspace = Workspace::new(cli.workspace.unwrap_or_else(|| PathBuf::from(".")));
    workspace.init()?;
    
    // 加载配置文件
    let config = if let Some(config_path) = cli.config {
        info!("正在加载配置文件: {:?}", config_path);
        let cfg = Config::from_file(config_path)?;
        info!("配置文件加载成功");
        cfg
    } else {
        // 尝试加载默认配置文件
        let app_config = AppConfig::default();
        let mut config = None;
        for path in app_config.default_config_paths.iter() {
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
        info!("收到Ctrl+C信号");
        Ok::<(), anyhow::Error>(())
    };

    // 主逻辑
    let work = async {
        match cli.command {
            Commands::Start { name, python, port, detach } => {
                if let Some(name) = name {
                    info!("正在启动进程: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::with_config(
                                &workspace,
                                name.clone(),
                                process_config.process.clone()
                            );
                            
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

                            if detach {
                                info!("进程已启动，主进程即将退出");
                                return Ok(());
                            }
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    // 使用命令行参数
                    info!("使用命令行参数启动进程");
                    let process_name = "default".to_string();
                    let manager = ProcessManager::with_config(
                        &workspace,
                        process_name,
                        config.global.process.clone()
                    );
                    let python = python.unwrap_or_else(|| manager.get_config().default_python_interpreter.clone());
                    let port = port.unwrap_or(manager.get_config().default_port);
                    
                    manager.start(
                        &python,
                        &[manager.get_config().default_script_path.clone()],
                        &config.global.state.default_working_dir,
                        Some(&format!("http://localhost:{}/health", port)),
                        None,
                    ).await?;

                    if detach {
                        info!("进程已启动，主进程即将退出");
                        return Ok(());
                    }
                }
            }
            Commands::Stop { name } => {
                if let Some(name) = name {
                    info!("正在停止进程: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::with_config(
                                &workspace,
                                name.clone(),
                                process_config.process.clone()
                            );
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
                    let process_name = "default".to_string();
                    let manager = ProcessManager::with_config(
                        &workspace,
                        process_name,
                        config.global.process.clone()
                    );
                    manager.stop().await?;
                }
            }
            Commands::Status { name, python, port } => {
                if let Some(name) = name {
                    info!("检查进程状态: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::with_config(
                                &workspace,
                                name.clone(),
                                process_config.process.clone()
                            );
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
                    let process_name = "default".to_string();
                    let manager = ProcessManager::with_config(
                        &workspace,
                        process_name,
                        config.global.process.clone()
                    );
                    let port = port.unwrap_or(manager.get_config().default_port);
                    let status = manager.status(
                        Some(&format!("http://localhost:{}/health", port))
                    ).await?;
                    info!("进程状态: {}", if status { "运行中" } else { "未运行" });
                }
            }
            Commands::Install { name } => {
                if let Some(name) = name {
                    info!("准备安装系统服务: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            
                            // 获取当前可执行文件路径
                            let executable = std::env::current_exe()?;
                            
                            // 创建服务配置
                            let service_config = ServiceConfig {
                                name: name.clone(),
                                executable,
                                working_dir: process_config.working_dir.clone(),
                                args: process_config.args.clone(),
                                env: process_config.env.clone(),
                                description: format!("FuckRun managed process: {}", name),
                                auto_restart: process_config.auto_restart,
                            };

                            // 安装服务
                            let service_manager = ServiceManager::new(service_config);
                            service_manager.install().await?;
                            info!("服务安装成功");
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    error!("请指定进程名称");
                    return Err(anyhow::anyhow!("请指定进程名称"));
                }
            }
            Commands::Uninstall { name } => {
                if let Some(name) = name {
                    info!("准备卸载系统服务: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(_) => {
                            // 创建服务配置（卸载只需要服务名称）
                            let service_config = ServiceConfig {
                                name: name.clone(),
                                ..Default::default()
                            };

                            // 卸载服务
                            let service_manager = ServiceManager::new(service_config);
                            service_manager.uninstall().await?;
                            info!("服务卸载成功");
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    error!("请指定进程名称");
                    return Err(anyhow::anyhow!("请指定进程名称"));
                }
            }
            Commands::Service { name } => {
                if let Some(name) = name {
                    info!("以服务模式运行: {}", name);
                    // 从配置文件获取进程配置
                    match config.get_process_config(&name) {
                        Some(process_config) => {
                            info!("找到进程配置: {:?}", process_config);
                            let manager = ProcessManager::with_config(
                                &workspace,
                                name.clone(),
                                process_config.process.clone()
                            );
                            
                            // 合并全局和进程环境变量
                            let mut env_vars = config.global.env.clone();
                            env_vars.extend(process_config.env.clone());
                            
                            // 启动进程并等待
                            manager.start(
                                &process_config.program,
                                &process_config.args,
                                &process_config.working_dir,
                                process_config.health_check_url.as_deref(),
                                Some(&env_vars),
                            ).await?;

                            // 在服务模式下，我们需要等待进程结束
                            info!("进程已启动，等待进程结束...");
                            
                            // 等待Ctrl+C信号
                            signal::ctrl_c().await?;
                            info!("收到停止信号，正在停止进程...");
                            
                            // 停止进程并等待其完全退出
                            manager.stop().await?;
                            info!("进程已停止");
                            
                            // 更新进程状态
                            manager.update_stopped_state().await?;
                            info!("进程状态已更新");
                            
                            // 正常退出
                            info!("服务已停止，主进程退出");
                            return Ok(());
                        }
                        None => {
                            error!("未找到进程配置: {}", name);
                            return Err(anyhow::anyhow!("未找到进程配置: {}", name));
                        }
                    }
                } else {
                    error!("请指定进程名称");
                    return Err(anyhow::anyhow!("请指定进程名称"));
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    // 等待任意一个完成
    tokio::select! {
        result = work => result,
        result = ctrl_c => result,
    }
}
