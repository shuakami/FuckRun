use anyhow::Result;
use std::path::PathBuf;
use log::{info, error};
use std::process::Command;

pub struct DaemonManager {
    /// 进程名称
    name: String,
    /// 可执行文件路径
    executable: PathBuf,
    /// 工作目录
    working_dir: PathBuf,
    /// 启动参数
    args: Vec<String>,
    /// 环境变量
    env: std::collections::HashMap<String, String>,
    /// 是否自动重启
    auto_restart: bool,
}

impl DaemonManager {
    pub fn new(
        name: String,
        executable: PathBuf,
        working_dir: PathBuf,
        args: Vec<String>,
        env: std::collections::HashMap<String, String>,
        auto_restart: bool,
    ) -> Self {
        Self {
            name,
            executable,
            working_dir,
            args,
            env,
            auto_restart,
        }
    }

    /// 启动守护进程
    pub async fn start(&mut self) -> Result<()> {
        info!("启动守护进程: {}", self.name);
        
        #[cfg(windows)]
        {
            // TODO: 实现Windows版本
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            self.start_linux_daemon().await
        }

        #[cfg(target_os = "macos")]
        {
            self.start_macos_daemon().await
        }
    }

    /// 停止守护进程
    pub async fn stop(&mut self) -> Result<()> {
        info!("停止守护进程: {}", self.name);
        
        #[cfg(windows)]
        {
            // TODO: 实现Windows版本
            Ok(())
        }

        #[cfg(target_os = "linux")]
        {
            self.stop_linux_daemon().await
        }

        #[cfg(target_os = "macos")]
        {
            self.stop_macos_daemon().await
        }
    }

    #[cfg(target_os = "linux")]
    async fn start_linux_daemon(&mut self) -> Result<()> {
        // 使用double fork技术创建守护进程
        match unsafe { libc::fork() } {
            -1 => return Err(anyhow::anyhow!("First fork failed")),
            0 => {
                // 第一个子进程
                if unsafe { libc::setsid() } < 0 {
                    return Err(anyhow::anyhow!("setsid failed"));
                }
                
                match unsafe { libc::fork() } {
                    -1 => return Err(anyhow::anyhow!("Second fork failed")),
                    0 => {
                        // 第二个子进程(守护进程)
                        let mut command = Command::new(&self.executable);
                        command
                            .args(&self.args)
                            .current_dir(&self.working_dir)
                            .envs(&self.env);
                            
                        command.spawn()?;
                        std::process::exit(0);
                    }
                    _ => std::process::exit(0),
                }
            }
            _ => Ok(()),
        }
    }

    #[cfg(target_os = "linux")]
    async fn stop_linux_daemon(&mut self) -> Result<()> {
        // 查找并终止守护进程
        // TODO: 实现Linux下的进程终止
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn start_macos_daemon(&mut self) -> Result<()> {
        // macOS下的实现类似Linux
        self.start_linux_daemon().await
    }

    #[cfg(target_os = "macos")]
    async fn stop_macos_daemon(&mut self) -> Result<()> {
        // macOS下的实现类似Linux
        self.stop_linux_daemon().await
    }
} 