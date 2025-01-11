use anyhow::Result;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use log::{info, error};
use crate::daemon::DaemonManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// 服务名称
    pub name: String,
    /// 可执行文件路径
    pub executable: PathBuf,
    /// 工作目录
    pub working_dir: PathBuf,
    /// 启动参数
    pub args: Vec<String>,
    /// 环境变量
    pub env: std::collections::HashMap<String, String>,
    /// 描述
    pub description: String,
    /// 是否自动重启
    pub auto_restart: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            executable: PathBuf::new(),
            working_dir: PathBuf::from("."),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            description: String::new(),
            auto_restart: true,
        }
    }
}

pub struct ServiceManager {
    config: ServiceConfig,
    daemon: Option<DaemonManager>,
}

impl ServiceManager {
    pub fn new(config: ServiceConfig) -> Self {
        Self { 
            config,
            daemon: None,
        }
    }

    /// 安装系统服务
    pub async fn install(&self) -> Result<()> {
        info!("开始安装系统服务: {}", self.config.name);
        
        #[cfg(windows)]
        {
            self.install_windows_service().await
        }

        #[cfg(target_os = "linux")]
        {
            self.install_systemd_service().await
        }

        #[cfg(target_os = "macos")]
        {
            self.install_launchd_service().await
        }
    }

    /// 卸载系统服务
    pub async fn uninstall(&self) -> Result<()> {
        info!("开始卸载系统服务: {}", self.config.name);
        
        #[cfg(windows)]
        {
            self.uninstall_windows_service().await
        }

        #[cfg(target_os = "linux")]
        {
            self.uninstall_systemd_service().await
        }

        #[cfg(target_os = "macos")]
        {
            self.uninstall_launchd_service().await
        }
    }

    /// 启动服务
    pub async fn start(&mut self) -> Result<()> {
        info!("启动服务: {}", self.config.name);
        
        // 创建守护进程管理器
        let mut daemon = DaemonManager::new(
            self.config.name.clone(),
            self.config.executable.clone(),
            self.config.working_dir.clone(),
            self.config.args.clone(),
            self.config.env.clone(),
            self.config.auto_restart,
        );
        
        // 启动守护进程
        daemon.start().await?;
        
        // 保存守护进程管理器
        self.daemon = Some(daemon);
        
        Ok(())
    }

    /// 停止服务
    pub async fn stop(&mut self) -> Result<()> {
        info!("停止服务: {}", self.config.name);
        
        // 停止守护进程
        if let Some(daemon) = self.daemon.as_mut() {
            daemon.stop().await?;
        }
        
        // 清理守护进程管理器
        self.daemon = None;
        
        Ok(())
    }

    /// 检查服务状态
    pub async fn status(&self) -> Result<bool> {
        info!("检查系统服务状态: {}", self.config.name);
        
        #[cfg(windows)]
        {
            self.check_windows_service().await
        }

        #[cfg(target_os = "linux")]
        {
            self.check_systemd_service().await
        }

        #[cfg(target_os = "macos")]
        {
            self.check_launchd_service().await
        }
    }

    #[cfg(windows)]
    async fn install_windows_service(&self) -> Result<()> {
        use tokio::process::Command;
        
        // 生成服务配置
        let service_path = format!(r#""{}" service"#, self.config.executable.display());
        
        // 使用sc.exe创建服务
        let output = Command::new("sc")
            .args([
                "create",
                &self.config.name,
                "binPath=",
                &service_path,
                "start=",
                "auto",
                "DisplayName=",
                &self.config.name,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("安装Windows服务失败: {}", error);
            return Err(anyhow::anyhow!("安装Windows服务失败: {}", error));
        }

        info!("Windows服务安装成功");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn install_systemd_service(&self) -> Result<()> {
        use tokio::fs;
        use std::fmt::Write;
        
        // 生成systemd服务配置
        let mut service_content = String::new();
        writeln!(service_content, "[Unit]")?;
        writeln!(service_content, "Description={}", self.config.description)?;
        writeln!(service_content, "After=network.target")?;
        writeln!(service_content, "\n[Service]")?;
        writeln!(service_content, "Type=simple")?;
        writeln!(service_content, "ExecStart={} service", self.config.executable.display())?;
        writeln!(service_content, "WorkingDirectory={}", self.config.working_dir.display())?;
        
        // 环境变量
        for (key, value) in &self.config.env {
            writeln!(service_content, "Environment={}={}", key, value)?;
        }

        if self.config.auto_restart {
            writeln!(service_content, "Restart=always")?;
            writeln!(service_content, "RestartSec=3")?;
        }

        writeln!(service_content, "\n[Install]")?;
        writeln!(service_content, "WantedBy=multi-user.target")?;

        // 写入服务文件
        let service_path = format!("/etc/systemd/system/{}.service", self.config.name);
        fs::write(&service_path, service_content).await?;

        // 重新加载systemd配置
        let output = Command::new("systemctl")
            .args(["daemon-reload"])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("重新加载systemd配置失败: {}", error);
            return Err(anyhow::anyhow!("重新加载systemd配置失败: {}", error));
        }

        // 启用服务
        let output = Command::new("systemctl")
            .args(["enable", &self.config.name])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("启用systemd服务失败: {}", error);
            return Err(anyhow::anyhow!("启用systemd服务失败: {}", error));
        }

        info!("systemd服务安装成功");
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn install_launchd_service(&self) -> Result<()> {
        use tokio::fs;
        use std::fmt::Write;
        
        // 生成launchd配置
        let mut plist_content = String::new();
        writeln!(plist_content, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(plist_content, "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">")?;
        writeln!(plist_content, "<plist version=\"1.0\">")?;
        writeln!(plist_content, "<dict>")?;
        writeln!(plist_content, "    <key>Label</key>")?;
        writeln!(plist_content, "    <string>{}</string>", self.config.name)?;
        writeln!(plist_content, "    <key>ProgramArguments</key>")?;
        writeln!(plist_content, "    <array>")?;
        writeln!(plist_content, "        <string>{}</string>", self.config.executable.display())?;
        writeln!(plist_content, "        <string>service</string>")?;
        writeln!(plist_content, "    </array>")?;
        writeln!(plist_content, "    <key>WorkingDirectory</key>")?;
        writeln!(plist_content, "    <string>{}</string>", self.config.working_dir.display())?;
        
        if self.config.auto_restart {
            writeln!(plist_content, "    <key>KeepAlive</key>")?;
            writeln!(plist_content, "    <true/>")?;
        }

        // 环境变量
        if !self.config.env.is_empty() {
            writeln!(plist_content, "    <key>EnvironmentVariables</key>")?;
            writeln!(plist_content, "    <dict>")?;
            for (key, value) in &self.config.env {
                writeln!(plist_content, "        <key>{}</key>", key)?;
                writeln!(plist_content, "        <string>{}</string>", value)?;
            }
            writeln!(plist_content, "    </dict>")?;
        }

        writeln!(plist_content, "</dict>")?;
        writeln!(plist_content, "</plist>")?;

        // 写入配置文件
        let plist_path = format!("/Library/LaunchDaemons/{}.plist", self.config.name);
        fs::write(&plist_path, plist_content).await?;

        // 加载服务
        let output = Command::new("launchctl")
            .args(["load", "-w", &plist_path])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("加载launchd服务失败: {}", error);
            return Err(anyhow::anyhow!("加载launchd服务失败: {}", error));
        }

        info!("launchd服务安装成功");
        Ok(())
    }

    // 卸载服务的实现...
    #[cfg(windows)]
    async fn uninstall_windows_service(&self) -> Result<()> {
        use tokio::process::Command;
        
        // 停止服务
        let _ = Command::new("sc")
            .args(["stop", &self.config.name])
            .output()
            .await?;

        // 删除服务
        let output = Command::new("sc")
            .args(["delete", &self.config.name])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("卸载Windows服务失败: {}", error);
            return Err(anyhow::anyhow!("卸载Windows服务失败: {}", error));
        }

        info!("Windows服务卸载成功");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn uninstall_systemd_service(&self) -> Result<()> {
        use tokio::process::Command;
        use tokio::fs;

        // 停止服务
        let _ = Command::new("systemctl")
            .args(["stop", &self.config.name])
            .output()
            .await?;

        // 禁用服务
        let _ = Command::new("systemctl")
            .args(["disable", &self.config.name])
            .output()
            .await?;

        // 删除服务文件
        let service_path = format!("/etc/systemd/system/{}.service", self.config.name);
        fs::remove_file(&service_path).await?;

        // 重新加载systemd配置
        let output = Command::new("systemctl")
            .args(["daemon-reload"])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("重新加载systemd配置失败: {}", error);
            return Err(anyhow::anyhow!("重新加载systemd配置失败: {}", error));
        }

        info!("systemd服务卸载成功");
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn uninstall_launchd_service(&self) -> Result<()> {
        use tokio::process::Command;
        use tokio::fs;

        let plist_path = format!("/Library/LaunchDaemons/{}.plist", self.config.name);

        // 卸载服务
        let output = Command::new("launchctl")
            .args(["unload", "-w", &plist_path])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("卸载launchd服务失败: {}", error);
            return Err(anyhow::anyhow!("卸载launchd服务失败: {}", error));
        }

        // 删除配置文件
        fs::remove_file(&plist_path).await?;

        info!("launchd服务卸载成功");
        Ok(())
    }

    // 检查服务状态的实现...
    #[cfg(windows)]
    async fn check_windows_service(&self) -> Result<bool> {
        use tokio::process::Command;
        
        let output = Command::new("sc")
            .args(["query", &self.config.name])
            .output()
            .await?;

        Ok(output.status.success())
    }

    #[cfg(target_os = "linux")]
    async fn check_systemd_service(&self) -> Result<bool> {
        use tokio::process::Command;
        
        let output = Command::new("systemctl")
            .args(["is-active", &self.config.name])
            .output()
            .await?;

        Ok(output.status.success())
    }

    #[cfg(target_os = "macos")]
    async fn check_launchd_service(&self) -> Result<bool> {
        use tokio::process::Command;
        
        let output = Command::new("launchctl")
            .args(["list", &self.config.name])
            .output()
            .await?;

        Ok(output.status.success())
    }
} 