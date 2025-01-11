use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use log::{info, warn};
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::future::Future;
use std::pin::Pin;
use crate::types::{FsConfig, ProcessPriority};

pub struct FsManager {
    config: FsConfig,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    priority: ProcessPriority,
}

impl PartialEq for ProcessInfo {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}

impl Eq for ProcessInfo {}

impl Hash for ProcessInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pid.hash(state);
    }
}

impl Default for FsManager {
    fn default() -> Self {
        Self {
            config: FsConfig::default(),
        }
    }
}

impl FsManager {
    pub fn new(config: FsConfig) -> Self {
        Self { config }
    }

    /// 强制删除文件，处理各种特殊情况
    pub async fn force_remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // 先尝试终止使用此文件的进程
        self.kill_file_processes(path).await?;
        
        // 清理Git相关的锁定
        self.cleanup_git_locks(path).await?;
        
        let mut last_error = None;

        for i in 0..self.config.max_retries {
            match self.try_remove_file(path).await {
                Ok(_) => {
                    info!("成功删除文件: {:?}", path);
                    return Ok(());
                }
                Err(e) => {
                    warn!("第{}次删除失败: {}", i + 1, e);
                    last_error = Some(e);
                    sleep(self.config.retry_delay()).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("无法删除文件")))
    }

    /// 强制删除目录，处理各种特殊情况
    pub async fn force_remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // 先尝试终止使用此目录的进程
        self.kill_directory_processes(path).await?;
        
        // 清理Git相关的锁定
        self.cleanup_git_locks(path).await?;
        
        let mut last_error = None;

        for i in 0..self.config.max_retries {
            match self.try_remove_dir_all(path).await {
                Ok(_) => {
                    info!("成功删除目录: {:?}", path);
                    return Ok(());
                }
                Err(e) => {
                    warn!("第{}次删除失败: {}", i + 1, e);
                    last_error = Some(e);
                    sleep(self.config.retry_delay()).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("无法删除目录")))
    }

    /// 终止使用指定文件的所有进程
    async fn kill_file_processes(&self, path: &Path) -> Result<()> {
        let mut processes = self.find_locking_processes(path).await?;
        
        // 按优先级排序（优先级数字越大，实际优先级越低）
        processes.sort_by_key(|p| p.priority.value());
        
        for process in processes {
            info!("正在终止进程: {} (PID: {})", process.name, process.pid);
            self.kill_process(process.pid).await?;
        }

        // 等待进程完全退出
        sleep(self.config.exit_wait()).await;
        Ok(())
    }

    /// 查找锁定文件的所有进程
    async fn find_locking_processes(&self, path: &Path) -> Result<Vec<ProcessInfo>> {
        let mut processes = HashSet::new();

        #[cfg(windows)]
        {
            // 1. 查找Python进程
            let output = std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq python.exe", "/NH", "/FO", "CSV"])
                .output()
                .context("查找Python进程失败")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if let Some(pid) = line.split(',').nth(1) {
                    let pid = pid.trim_matches('"').parse::<u32>()?;
                    processes.insert(ProcessInfo {
                        pid,
                        name: "python.exe".to_string(),
                        priority: ProcessPriority::Application,
                    });
                }
            }

            // 2. 查找Git进程
            let git_processes = ["git.exe", "git-lfs.exe"];
            for git_proc in git_processes.iter() {
                let output = std::process::Command::new("tasklist")
                    .args(["/FI", &format!("IMAGENAME eq {}", git_proc), "/NH", "/FO", "CSV"])
                    .output()
                    .context("查找Git进程失败")?;
                
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if let Some(pid) = line.split(',').nth(1) {
                        let pid = pid.trim_matches('"').parse::<u32>()?;
                        processes.insert(ProcessInfo {
                            pid,
                            name: git_proc.to_string(),
                            priority: ProcessPriority::Application,
                        });
                    }
                }
            }

            // 3. 使用handle.exe查找系统进程（如果可用）
            if let Ok(output) = std::process::Command::new("handle.exe")
                .arg(path.to_str().unwrap())
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if let Some(pid) = line.split_whitespace().nth(2) {
                        if let Ok(pid) = pid.parse::<u32>() {
                            // 获取进程名
                            if let Ok(proc_output) = std::process::Command::new("tasklist")
                                .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
                                .output()
                            {
                                let proc_str = String::from_utf8_lossy(&proc_output.stdout);
                                if let Some(name) = proc_str.lines().next().and_then(|l| l.split(',').next()) {
                                    let name = name.trim_matches('"').to_string();
                                    let priority = if name.eq_ignore_ascii_case("explorer.exe") 
                                        || name.eq_ignore_ascii_case("system") {
                                        ProcessPriority::System
                                    } else {
                                        ProcessPriority::Application
                                    };
                                    processes.insert(ProcessInfo {
                                        pid,
                                        name,
                                        priority,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(unix)]
        {
            use std::process::Command;
            // 使用lsof查找所有使用文件的进程
            if let Ok(output) = Command::new("lsof")
                .arg(path.to_str().unwrap())
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines().skip(1) {  // 跳过标题行
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 2 {
                        if let Ok(pid) = fields[1].parse::<u32>() {
                            let name = fields[0].to_string();
                            let priority = if name.contains("python") || name.contains("git") {
                                ProcessPriority::Application
                            } else if name.eq_ignore_ascii_case("explorer") 
                                || name.eq_ignore_ascii_case("system") {
                                ProcessPriority::System
                            } else {
                                ProcessPriority::Temporary
                            };
                            processes.insert(ProcessInfo {
                                pid,
                                name,
                                priority,
                            });
                        }
                    }
                }
            }
        }

        Ok(processes.into_iter().collect())
    }

    /// 终止指定的进程
    async fn kill_process(&self, pid: u32) -> Result<()> {
        #[cfg(windows)]
        {
            std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .output()
                .context("终止进程失败")?;
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
                .context("终止进程失败")?;
        }

        Ok(())
    }

    /// 清理Git相关的锁定
    async fn cleanup_git_locks(&self, path: &Path) -> Result<()> {
        // 查找.git目录
        let mut current = path;
        while let Some(parent) = current.parent() {
            let git_dir = parent.join(".git");
            if git_dir.exists() {
                // 清理index.lock
                let index_lock = git_dir.join("index.lock");
                if index_lock.exists() {
                    if let Err(e) = fs::remove_file(&index_lock) {
                        warn!("清理git index锁定失败: {}", e);
                    }
                }
                
                // 清理其他Git锁定文件
                let refs_dir = git_dir.join("refs");
                if refs_dir.exists() {
                    self.remove_git_locks_inner(&refs_dir).await?;
                }
                
                break;
            }
            current = parent;
        }
        Ok(())
    }

    /// 递归清理Git锁定文件的内部实现
    fn remove_git_locks_inner<'a>(&'a self, dir: &'a Path) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        self.remove_git_locks_inner(&path).await?;
                    } else if path.extension().map_or(false, |ext| ext == "lock") {
                        if let Err(e) = fs::remove_file(&path) {
                            warn!("清理git锁定文件失败: {}", e);
                        }
                    }
                }
            }
            Ok(())
        })
    }

    /// 终止使用指定目录的所有进程
    async fn kill_directory_processes(&self, path: &Path) -> Result<()> {
        // 遍历目录中的所有Python文件
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "py") {
                self.kill_file_processes(&path).await?;
            }
        }
        Ok(())
    }

    async fn try_remove_file(&self, path: &Path) -> Result<()> {
        // 1. 尝试修改文件属性
        self.remove_readonly(path)?;
        
        // 2. 尝试强制删除
        #[cfg(windows)]
        {
            // 在Windows上使用特权API
            use std::process::Command;
            Command::new("cmd")
                .args(["/C", "del", "/F", "/A", path.to_str().unwrap()])
                .output()
                .context("执行del命令失败")?;
        }

        #[cfg(unix)]
        {
            // 在Unix上使用chmod
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(self.config.default_file_mode);
            fs::set_permissions(path, perms)?;
            fs::remove_file(path).context("删除文件失败")?;
        }

        Ok(())
    }

    async fn try_remove_dir_all(&self, path: &Path) -> Result<()> {
        // 1. 修改目录和所有子项的属性
        self.remove_readonly(path)?;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.remove_readonly(&path)?;
            }
        }

        // 2. 平台特定处理
        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("cmd")
                .args(["/C", "rmdir", "/S", "/Q", path.to_str().unwrap()])
                .output()
                .context("执行rmdir命令失败")?;
        }

        #[cfg(unix)]
        {
            // 在Unix上使用chmod递归修改权限
            use std::process::Command;
            Command::new("chmod")
                .args(["-R", &format!("{:o}", self.config.default_dir_mode), path.to_str().unwrap()])
                .output()
                .context("修改权限失败")?;
            fs::remove_dir_all(path).context("删除目录失败")?;
        }

        Ok(())
    }

    fn remove_readonly(&self, path: &Path) -> Result<()> {
        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_readonly(false);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = if metadata.is_dir() {
                    self.config.default_dir_mode
                } else {
                    self.config.default_file_mode
                };
                perms.set_mode(mode);
            }
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }
} 