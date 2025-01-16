use anyhow::Result;
use log::info;
use crate::process::ProcessManager;
use crate::config::Config;
use crate::workspace::Workspace;

pub async fn handle_start(
    workspace: &Workspace,
    config: &Config,
    name: Option<String>,
    python: Option<String>,
    port: Option<u16>,
    detach: bool,
    daemon: bool,
    auto_restart: bool,
) -> Result<()> {
    if let Some(name) = name {
        info!("启动进程: {}", name);
        // 从配置文件获取进程配置
        match config.get_process_config(&name) {
            Some(mut process_config) => {
                // 检查Python解释器
                if cfg!(windows) && process_config.program.contains("python.exe") {
                    info!("检测到python.exe，使用py启动器替代");
                    process_config.program = String::from("py");
                }

                let mut manager = ProcessManager::with_config(
                    workspace,
                    name.clone(),
                    process_config.process.clone()
                );

                // 设置自动重启
                manager.set_auto_restart(auto_restart);
                
                // 设置守护进程模式
                manager.set_daemon_mode(daemon);

                // 启动进程
                let working_dir = workspace.get_root_dir()
                    .join("deployments")
                    .join(&name)
                    .join("app")
                    .join(&name);
                
                manager.start(
                    &process_config.program,
                    &process_config.args,
                    &working_dir,
                    process_config.health_check_url.as_deref(),
                    Some(&process_config.env),
                ).await?;

                if detach {
                    info!("进程已启动，主进程即将退出");
                    return Ok(());
                }

                info!("进程启动成功");
            }
            None => {
                anyhow::bail!("未找到进程配置: {}", name);
            }
        }
    } else {
        // 使用默认配置启动
        info!("使用默认配置启动进程");
        let mut manager = ProcessManager::with_config(
            workspace,
            "default".to_string(),
            config.global.process.clone()
        );
        
        // 设置自动重启
        manager.set_auto_restart(auto_restart);
        
        // 设置守护进程模式
        manager.set_daemon_mode(daemon);

        // 启动进程
        let program = python.unwrap_or_else(|| "python".to_string());
        // 检查Python解释器
        let program = if cfg!(windows) && program.contains("python.exe") {
            String::from("py")
        } else {
            program
        };
        let args = vec!["-m".to_string(), "http.server".to_string(), port.unwrap_or(8000).to_string()];
        
        manager.start(
            &program,
            &args,
            &workspace.get_root_dir().to_path_buf(),
            None,
            None,
        ).await?;

        if detach {
            info!("进程已启动，主进程即将退出");
            return Ok(());
        }

        info!("进程启动成功");
    }
    Ok(())
} 