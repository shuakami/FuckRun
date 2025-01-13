use anyhow::Result;
use log::info;
use crate::process::ProcessManager;
use crate::config::Config;
use crate::workspace::Workspace;

pub async fn handle_stop(
    workspace: &Workspace,
    config: &Config,
    name: Option<String>,
) -> Result<()> {
    if let Some(name) = name {
        info!("正在停止进程: {}", name);
        // 从配置文件获取进程配置
        match config.get_process_config(&name) {
            Some(process_config) => {
                let manager = ProcessManager::with_config(
                    workspace,
                    name.clone(),
                    process_config.process.clone()
                );
                manager.stop().await?;
            }
            None => {
                anyhow::bail!("未找到进程配置: {}", name);
            }
        }
    } else {
        // 使用默认配置
        info!("停止默认进程");
        let process_name = "default".to_string();
        let manager = ProcessManager::with_config(
            workspace,
            process_name,
            config.global.process.clone()
        );
        manager.stop().await?;
    }
    Ok(())
} 