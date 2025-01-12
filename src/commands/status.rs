use anyhow::Result;
use log::info;
use crate::process::ProcessManager;
use crate::config::Config;
use crate::workspace::Workspace;

pub async fn handle_status(
    workspace: &Workspace,
    config: &Config,
    name: Option<String>,
    port: Option<u16>,
) -> Result<()> {
    if let Some(name) = name {
        info!("检查进程状态: {}", name);
        // 从配置文件获取进程配置
        match config.get_process_config(&name) {
            Some(process_config) => {
                info!("找到进程配置: {:?}", process_config);
                let manager = ProcessManager::with_config(
                    workspace,
                    name.clone(),
                    process_config.process.clone()
                );
                let status = manager.status(
                    process_config.health_check_url.as_deref()
                ).await?;
                if status {
                    info!("进程正在运行");
                } else {
                    info!("进程未运行");
                }
            }
            None => {
                anyhow::bail!("未找到进程配置: {}", name);
            }
        }
    } else {
        // 使用默认配置
        info!("检查默认进程状态");
        let process_name = "default".to_string();
        let manager = ProcessManager::with_config(
            workspace,
            process_name,
            config.global.process.clone()
        );
        let status = manager.status(
            Some(&format!("http://localhost:{}/health", port.unwrap_or(manager.get_config().default_port)))
        ).await?;
        if status {
            info!("进程正在运行");
        } else {
            info!("进程未运行");
        }
    }
    Ok(())
} 