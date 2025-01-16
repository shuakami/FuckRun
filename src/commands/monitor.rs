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
use std::io::Write;

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
    
    // 检查Python解释器是否存在
    let program = if cfg!(windows) && program.contains("python.exe") {
        // 在Windows上，优先使用py启动器
        String::from("py")
    } else {
        program
    };
    
    info!("使用Python解释器: {}", program);
    
    info!("参数: {:?}", args);
    info!("工作目录: {:?}", working_dir);
    
    // 确保工作目录是绝对路径
    let working_dir = if working_dir.is_absolute() {
        working_dir
    } else {
        workspace.get_root_dir().join(working_dir)
    };
    
    info!("工作目录(绝对路径): {:?}", working_dir);
    info!("工作目录是否存在: {}", working_dir.exists());
    if working_dir.exists() {
        info!("工作目录内容:");
        if let Ok(entries) = std::fs::read_dir(&working_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    info!("  - {:?}", entry.path());
                }
            }
        }
    }
    
    // 检查工作目录是否存在
    if !working_dir.exists() {
        error!("工作目录不存在: {:?}", working_dir);
        return Err(anyhow::anyhow!("工作目录不存在: {:?}", working_dir));
    }
    
    // 检查并修正参数中的文件路径
    let mut fixed_args = Vec::new();
    for arg in args {
        if arg.ends_with(".py") {
            let script_path = if arg.contains('\\') || arg.contains('/') {
                // 如果参数中已经包含路径分隔符,说明是完整路径
                PathBuf::from(arg)
            } else {
                // 否则基于工作目录构建路径
                working_dir.join(arg)
            };
            info!("Python脚本路径: {:?}", script_path);
            info!("Python脚本是否存在: {}", script_path.exists());
            if script_path.exists() {
                info!("Python脚本大小: {} bytes", std::fs::metadata(&script_path).map(|m| m.len()).unwrap_or(0));
            }
            if !script_path.exists() {
                error!("Python脚本不存在: {:?}", script_path);
                return Err(anyhow::anyhow!("Python脚本不存在: {:?}", script_path));
            }
            fixed_args.push(script_path.to_string_lossy().to_string());
        } else {
            fixed_args.push(arg);
        }
    }
    
    info!("修正后的参数: {:?}", fixed_args);
    
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
    cmd.args(&fixed_args)
       .current_dir(&working_dir)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .stdin(Stdio::null());

    cmd.envs(&env_map);

    #[cfg(windows)]
    {
        use winapi::um::winbase::CREATE_NO_WINDOW;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // 创建日志文件
    let process_dir = workspace.get_process_dir(&process_name);
    let log_dir = process_dir.join("logs").join("monitor");
    std::fs::create_dir_all(&log_dir)?;
    
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = log_dir.join(format!("{}.log", today));
    
    info!("监控进程日志文件: {:?}", log_path);
    
    // 定义一个辅助函数来写日志
    let write_log = |message: &str| -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        writeln!(file, "{} {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message)?;
        Ok(())
    };
    
    // 写入启动信息
    write_log("[INFO] 启动监控进程")?;
    write_log(&format!("[INFO] 进程名称: {}", process_name))?;
    write_log(&format!("[INFO] Python解释器: {}", program))?;
    write_log(&format!("[INFO] 参数: {:?}", fixed_args))?;
    write_log(&format!("[INFO] 工作目录: {:?}", working_dir))?;
    
    let mut child = cmd.spawn()?;
    let pid = child.id().unwrap_or(0);
    info!("进程已启动, PID: {}", pid);
    write_log(&format!("[INFO] 进程已启动, PID: {}", pid))?;

    // 等待进程初始化
    info!("等待进程初始化({:?}秒)...", config.global.process.init_wait_secs);
    write_log(&format!("[INFO] 等待进程初始化({:?}秒)...", config.global.process.init_wait_secs))?;
    sleep(Duration::from_secs(config.global.process.init_wait_secs)).await;

    // 获取进程配置
    let process_config = config.get_process_config(&process_name);
    let health_check_url = process_config.as_ref().and_then(|cfg| cfg.health_check_url.as_deref());

    // 保存进程状态
    let mut state = ProcessState::load(workspace, &process_name)
        .unwrap_or_else(|_| ProcessState::default());
    info!("加载现有状态: {:?}", state);
    write_log(&format!("[INFO] 加载现有状态: {:?}", state))?;
    
    let monitor_pid = state.monitor_pid;  // 保存monitor_pid
    state.pid = Some(pid as i32);
    state.program = program.clone();
    state.args = fixed_args.clone();
    state.working_dir = working_dir.clone();
    state.port = config.global.process.default_port;
    state.health_check_url = health_check_url.map(String::from);
    state.monitor_pid = monitor_pid;  // 恢复monitor_pid
    info!("更新后状态: {:?}", state);
    write_log(&format!("[INFO] 更新后状态: {:?}", state))?;
    
    state.save(workspace, &process_name)?;
    info!("进程状态已保存");
    write_log("[INFO] 进程状态已保存")?;

    // 处理进程输出
    ProcessManager::handle_process_output(
        process_name.clone(),
        logger,
        child.stdout.take(),
        child.stderr.take()
    ).await;
    
    info!("开始监控进程...");
    write_log("[INFO] 开始监控进程...")?;
    
    // 等待监控完成
    match manager.monitor_and_restart(child).await {
        Ok(_) => {
            info!("进程监控结束");
            write_log("[INFO] 进程监控结束")?;
            Ok(())
        }
        Err(e) => {
            error!("进程监控失败: {}", e);
            write_log(&format!("[ERROR] 进程监控失败: {}", e))?;
            Err(e)
        }
    }
} 