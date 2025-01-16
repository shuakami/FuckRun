use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::{info, warn};
use std::fs;

/// 工作区管理器
#[derive(Clone)]
pub struct Workspace {
    /// 项目根目录
    root: PathBuf,
    /// 用户应用目录
    app_dir: PathBuf,
    /// FuckRun工作目录
    fuckrun_dir: PathBuf,
}

impl Workspace {
    /// 创建新的工作区
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        // 将传入的路径转换为绝对路径
        let root = if root.as_ref() == Path::new(".") {
            // 如果是".",返回项目根目录
            std::env::current_exe()
                .expect("无法获取当前可执行文件路径")
                .parent()  // target/debug
                .expect("无法获取父目录")
                .parent()  // target
                .expect("无法获取父目录")
                .parent()  // 项目根目录
                .expect("无法获取父目录")
                .to_path_buf()
        } else if root.as_ref().is_absolute() {
            root.as_ref().to_path_buf()
        } else {
            std::env::current_dir()
                .expect("无法获取当前目录")
                .join(root)
        };
        
        info!("工作区根目录(绝对路径): {:?}", root);
        
        // 初始化时app_dir设为root/deployments
        let app_dir = root.join("deployments");
        let fuckrun_dir = root.join(".fuckrun");
        
        Self {
            root,
            app_dir,
            fuckrun_dir,
        }
    }

    /// 设置应用程序目录
    pub fn set_app_dir<P: AsRef<Path>>(&mut self, app_dir: P) {
        let app_dir = if app_dir.as_ref().is_absolute() {
            app_dir.as_ref().to_path_buf()
        } else {
            self.root.join(app_dir)
        };
        
        info!("设置应用程序目录: {:?}", app_dir);
        self.app_dir = app_dir;
    }

    /// 获取进程的应用目录
    pub fn get_process_app_dir(&self, process_name: &str) -> PathBuf {
        self.app_dir.join(process_name).join("app")
    }

    /// 初始化工作区目录结构
    pub fn init(&self) -> Result<()> {
        info!("初始化工作区目录结构");
        
        // 创建应用目录
        if !self.app_dir.exists() {
            info!("创建应用目录: {:?}", self.app_dir);
            fs::create_dir_all(&self.app_dir)
                .context("创建应用目录失败")?;
        }

        // 创建 FuckRun 工作目录
        if !self.fuckrun_dir.exists() {
            info!("创建 FuckRun 工作目录: {:?}", self.fuckrun_dir);
            fs::create_dir_all(&self.fuckrun_dir)
                .context("创建 FuckRun 工作目录失败")?;
        }

        // 创建进程管理目录
        let processes_dir = self.get_processes_dir();
        if !processes_dir.exists() {
            info!("创建进程管理目录: {:?}", processes_dir);
            fs::create_dir_all(&processes_dir)
                .context("创建进程管理目录失败")?;
        }

        info!("工作区初始化完成");
        Ok(())
    }

    /// 验证工作区结构
    pub fn validate(&self) -> Result<()> {
        info!("验证工作区结构");
        
        if !self.root.exists() {
            return Err(anyhow::anyhow!("项目根目录不存在: {:?}", self.root));
        }

        if !self.app_dir.exists() {
            return Err(anyhow::anyhow!("应用目录不存在: {:?}", self.app_dir));
        }

        if !self.fuckrun_dir.exists() {
            return Err(anyhow::anyhow!("FuckRun 工作目录不存在: {:?}", self.fuckrun_dir));
        }

        Ok(())
    }

    /// 获取进程工作目录
    pub fn get_process_dir(&self, process_name: &str) -> PathBuf {
        self.fuckrun_dir.join("processes").join(process_name)
    }

    /// 获取进程状态文件路径
    pub fn get_process_state_file(&self, process_name: &str) -> PathBuf {
        self.get_process_dir(process_name).join("state.json")
    }

    /// 获取进程日志目录
    pub fn get_process_log_dir(&self, process_name: &str) -> PathBuf {
        self.get_process_dir(process_name).join("logs")
    }

    /// 获取应用目录
    pub fn get_app_dir(&self) -> &Path {
        &self.app_dir
    }

    /// 获取项目根目录
    pub fn get_root_dir(&self) -> &Path {
        &self.root
    }

    /// 获取 FuckRun 工作目录
    pub fn get_fuckrun_dir(&self) -> &Path {
        &self.fuckrun_dir
    }

    /// 获取进程管理目录
    pub fn get_processes_dir(&self) -> PathBuf {
        self.fuckrun_dir.join("processes")
    }

    /// 确保进程目录结构存在
    pub fn ensure_process_dirs(&self, process_name: &str) -> Result<()> {
        let process_dir = self.get_process_dir(process_name);
        if !process_dir.exists() {
            info!("创建进程目录: {:?}", process_dir);
            fs::create_dir_all(&process_dir)
                .context("创建进程目录失败")?;
        }

        let log_dir = self.get_process_log_dir(process_name);
        if !log_dir.exists() {
            info!("创建进程日志目录: {:?}", log_dir);
            fs::create_dir_all(&log_dir)
                .context("创建进程日志目录失败")?;
        }

        Ok(())
    }
} 