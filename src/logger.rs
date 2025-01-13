use std::path::{Path, PathBuf};
use std::io::Write;
use anyhow::Result;
use log::{LevelFilter, info, error};
use log4rs::{
    append::rolling_file::RollingFileAppender,
    append::console::ConsoleAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use chrono::Local;
use crate::workspace::Workspace;

/// 日志管理器
#[derive(Clone)]
pub struct Logger {
    workspace: Workspace,
}

impl Logger {
    /// 创建新的日志管理器
    pub fn new(workspace: Workspace) -> Self {
        Self {
            workspace,
        }
    }

    /// 获取主程序日志目录
    fn get_main_log_dir(&self) -> PathBuf {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.workspace.get_fuckrun_dir().join("logs").join(today)
    }

    /// 获取进程日志根目录
    fn get_process_log_root(&self, process_name: &str) -> PathBuf {
        self.workspace.get_process_dir(process_name).join("logs")
    }

    /// 获取进程日志目录
    fn get_process_log_dir(&self, process_name: &str) -> PathBuf {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.get_process_log_root(process_name).join(today)
    }

    /// 初始化日志系统
    pub fn init(&self) -> Result<()> {
        // 创建主日志目录
        let main_log_dir = self.get_main_log_dir();
        std::fs::create_dir_all(&main_log_dir)?;

        // 配置控制台输出
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("[{l}] {f}:{L} - {m}{n}")))
            .build();

        // 配置主日志文件
        let main_log = main_log_dir.join("fuckrun.log");
        let main_policy = CompoundPolicy::new(
            Box::new(SizeTrigger::new(50 * 1024 * 1024)), // 50MB
            Box::new(FixedWindowRoller::builder()
                .build(&format!("{}.{{}}.gz", main_log.display()), 5)
                .unwrap()),
        );

        let main_appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} [{l}] {f}:{L} - {m}{n}")))
            .append(true)  // 追加模式
            .build(main_log, Box::new(main_policy))?;

        // 创建日志配置
        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("main", Box::new(main_appender)))
            .build(Root::builder()
                .appender("stdout")
                .appender("main")
                .build(LevelFilter::Info))?;

        // 应用配置
        log4rs::init_config(config)?;

        // 在Windows下设置UTF-8编码
        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("chcp")
                .arg("65001")
                .output()
                .ok();
        }

        Ok(())
    }

    /// 为进程创建日志文件
    pub fn create_process_logs(&self, process_name: &str) -> Result<(PathBuf, PathBuf)> {
        let process_dir = self.get_process_log_dir(process_name);
        std::fs::create_dir_all(&process_dir)?;

        let stdout_log = process_dir.join("stdout.log");
        let stderr_log = process_dir.join("stderr.log");

        // 创建空文件
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&stdout_log)?;
        
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&stderr_log)?;

        Ok((stdout_log, stderr_log))
    }

    /// 写入进程标准输出
    pub fn write_stdout(&self, process_name: &str, line: &str) -> Result<()> {
        let process_dir = self.get_process_log_dir(process_name);
        let stdout_log = process_dir.join("stdout.log");
        
        // 如果目录不存在,创建目录和文件
        if !process_dir.exists() {
            std::fs::create_dir_all(&process_dir)?;
            info!("为进程 {} 创建日志目录: {:?}", process_name, process_dir);
        }
        
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(stdout_log)?
            .write_all(format!("{}\n", line).as_bytes())?;
            
        info!("[{}][stdout] {}", process_name, line);
        Ok(())
    }

    /// 写入进程标准错误
    pub fn write_stderr(&self, process_name: &str, line: &str) -> Result<()> {
        let process_dir = self.get_process_log_dir(process_name);
        let stderr_log = process_dir.join("stderr.log");
        
        // 如果目录不存在,创建目录和文件
        if !process_dir.exists() {
            std::fs::create_dir_all(&process_dir)?;
            info!("为进程 {} 创建日志目录: {:?}", process_name, process_dir);
        }
        
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(stderr_log)?
            .write_all(format!("{}\n", line).as_bytes())?;
            
        info!("[{}][stderr] {}", process_name, line);
        Ok(())
    }
} 