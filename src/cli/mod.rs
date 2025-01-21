use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 工作目录
    #[arg(short, long)]
    pub workspace: Option<PathBuf>,

    /// 配置文件路径
    #[arg(short, long)]
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

        /// 是否使用Python解释器
        #[arg(short = 'y', long)]
        python: bool,

        /// 端口号
        #[arg(short, long)]
        port: Option<u16>,

        /// 是否分离运行
        #[arg(short, long)]
        detach: bool,

        /// 是否作为守护进程运行
        #[arg(long)]
        daemon: bool,

        /// 是否自动重启
        #[arg(long)]
        auto_restart: bool,

        /// 应用程序目录
        #[arg(long)]
        app_dir: Option<PathBuf>,
    },

    /// 停止进程
    Stop {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,

        /// 应用程序目录
        #[arg(long)]
        app_dir: Option<PathBuf>,
    },

    /// 查看进程状态
    Status {
        /// 进程名称
        #[arg(short, long)]
        name: Option<String>,

        /// 是否使用Python解释器
        #[arg(short, long)]
        python: bool,

        /// 端口号
        #[arg(short, long)]
        port: Option<u16>,

        /// 应用程序目录
        #[arg(long)]
        app_dir: Option<PathBuf>,
    },

    /// 监控进程
    Monitor {
        /// 进程名称
        #[arg(short = 'n', long)]
        process_name: String,

        /// 程序路径
        #[arg(short, long)]
        program: String,

        /// 程序参数
        #[arg(short, long)]
        args: Vec<String>,

        /// 工作目录
        #[arg(short, long)]
        working_dir: PathBuf,

        /// 环境变量
        #[arg(short, long)]
        env_vars: Vec<String>,

        /// 是否自动重启
        #[arg(long)]
        auto_restart: bool,

        /// 配置文件路径
        #[arg(short, long)]
        config: PathBuf,

        /// 主机地址
        #[arg(long)]
        host: Option<String>,

        /// 端口号
        #[arg(short = 'P', long)]
        port: Option<u16>,
    },

    /// 查看日志
    Logs {
        /// 进程名称
        #[arg(short, long)]
        name: String,

        /// 是否实时查看
        #[arg(short, long)]
        follow: bool,

        /// 日志类型(stdout/stderr)
        #[arg(short, long, default_value = "stdout")]
        log_type: String,

        /// 日期(YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,

        /// 应用程序目录
        #[arg(long)]
        app_dir: Option<PathBuf>,
    },

    /// 查看系统日志
    SystemLogs {
        /// 是否实时查看
        #[arg(short, long)]
        follow: bool,

        /// 日期(YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },

    /// 列出所有进程
    List {
        /// 应用程序目录
        #[arg(long)]
        app_dir: Option<PathBuf>,

        /// 按名称筛选
        #[arg(long)]
        name: Option<String>,

        /// 按状态筛选(running/stopped)
        #[arg(long)]
        status: Option<String>,

        /// 最小运行时间(秒)
        #[arg(long)]
        min_uptime: Option<u64>,

        /// 最大运行时间(秒)
        #[arg(long)]
        max_uptime: Option<u64>,

        /// 最小CPU使用率(%)
        #[arg(long)]
        min_cpu: Option<f64>,

        /// 最大CPU使用率(%)
        #[arg(long)]
        max_cpu: Option<f64>,

        /// 最小内存使用(MB)
        #[arg(long)]
        min_mem: Option<u64>,

        /// 最大内存使用(MB)
        #[arg(long)]
        max_mem: Option<u64>,

        /// 输出JSON格式
        #[arg(long)]
        json: bool,

        /// 实时更新
        #[arg(short, long)]
        watch: bool,
    },
} 