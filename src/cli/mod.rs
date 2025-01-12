use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 工作目录路径
    #[arg(short = 'w', long, value_name = "DIR")]
    pub workspace: Option<PathBuf>,

    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

        /// 以守护进程模式运行
        #[arg(long)]
        daemon: bool,

        /// 自动重启
        #[arg(long)]
        auto_restart: bool,
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
    /// 监控进程(内部命令)
    Monitor {
        /// 进程名称
        #[arg(long)]
        process_name: String,
        
        /// 要执行的程序
        #[arg(long)]
        program: String,
        
        /// 程序参数
        #[arg(long = "arg")]
        args: Vec<String>,
        
        /// 工作目录
        #[arg(long)]
        working_dir: PathBuf,
        
        /// 环境变量
        #[arg(long = "env")]
        env_vars: Vec<String>,
        
        /// 自动重启
        #[arg(long)]
        auto_restart: bool,

        /// 配置文件路径
        #[arg(long)]
        config: PathBuf,
    },
    /// 查看进程日志
    Logs {
        /// 进程名称
        #[arg(short, long)]
        name: String,

        /// 实时跟踪
        #[arg(short, long)]
        follow: bool,

        /// 日志类型 (stdout/stderr)
        #[arg(short = 't', long, default_value = "stdout")]
        log_type: String,

        /// 日期 (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// 查看系统日志
    SystemLogs {
        /// 实时跟踪
        #[arg(short, long)]
        follow: bool,

        /// 日期 (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },
} 