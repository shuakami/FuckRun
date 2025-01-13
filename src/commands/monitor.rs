use anyhow::Result;
use log::{info, error};
use std::collections::HashMap;
use tokio::process::Command;
use std::process::Stdio;
use crate::process::ProcessManager;
use crate::config::Config;
use crate::workspace::Workspace;
use crate::logger::Logger;
use std::path::PathBuf;
use crate::process::daemon::ProcessManagerDaemonExt;
use tokio::time::sleep;
use std::time::Duration;
use crate::state::ProcessState;

pub async fn handle_monitor(
    workspace: &Workspace,
    config: &Config,
    process_name: String,
    program: String,
    args: Vec<String>,
    working_dir: PathBuf,
    env_vars: Vec<String>,
    auto_restart: bool,
    logger: Logger,
) -> Result<()> {
    info!("启动进程监控");
    info!("进程名称: {}", process_name);
    info!("程序: {}", program);
    info!("参数: {:?}", args);
    info!("工作目录: {:?}", working_dir);
    
    // 解析环境变量
    let mut env_map = HashMap::new();
    for env in env_vars {
        if let Some((key, value)) = env.split_once('=') {
            env_map.insert(key.to_string(), value.to_string());
        }
    }
    
    // 创建进程管理器
    let mut manager = ProcessManager::with_config(
        workspace,
        process_name.clone(),
        config.global.process.clone()
    );
    
    // 设置自动重启
    manager.set_auto_restart(auto_restart);
    
    // 启动并监控进程
    let mut cmd = Command::new(&program);
    cmd.args(&args)
       .current_dir(&working_dir)
       .envs(&env_map)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    // Windows平台特定配置
    #[cfg(windows)]
    cmd.creation_flags(config.global.process.windows_process_flags);
    
    let mut child = cmd.spawn()?;
    let pid = child.id().unwrap_or(0);
    info!("进程已启动, PID: {}", pid);

    // 等待进程初始化
    info!("等待进程初始化({:?}秒)...", config.global.process.init_wait_secs);
    sleep(Duration::from_secs(config.global.process.init_wait_secs)).await;

    // 获取进程配置
    let process_config = config.get_process_config(&process_name);
    let health_check_url = process_config.as_ref().and_then(|cfg| cfg.health_check_url.as_deref());

    // 保存进程状态
    let mut state = ProcessState::load(workspace, &process_name)
        .unwrap_or_else(|_| ProcessState::default());
    info!("加载现有状态: {:?}", state);
    let monitor_pid = state.monitor_pid;  // 保存monitor_pid
    state.pid = Some(pid as i32);
    state.program = program.clone();
    state.args = args.clone();
    state.working_dir = working_dir.clone();
    state.port = config.global.process.default_port;
    state.health_check_url = health_check_url.map(String::from);
    state.monitor_pid = monitor_pid;  // 恢复monitor_pid
    info!("更新后状态: {:?}", state);
    state.save(workspace, &process_name)?;
    info!("进程状态已保存");

    // 处理进程输出
    ProcessManager::handle_process_output(
        process_name.clone(),
        logger,
        child.stdout.take(),
        child.stderr.take()
    ).await;
    
    info!("开始监控进程...");
    
    // 等待监控完成
    // ❤ 这里调用的是 daemon 的 monitor_and_restart 方法
    match manager.monitor_and_restart(child).await {
        Ok(_) => {
            info!("进程监控结束");
            Ok(())
        }
        Err(e) => {
            error!("进程监控失败: {}", e);
            Err(e)
        }
    }
} 