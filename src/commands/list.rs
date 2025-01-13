use anyhow::{Result, Context};
use log::info;
use prettytable::{Table, row};
use std::time::UNIX_EPOCH;
use tokio::process::Command;
use crate::workspace::Workspace;
use crate::state::ProcessState;

// 定义进程信息结构体
#[derive(Debug)]
struct ProcessInfo {
    name: String,
    pid: String,
    monitor_pid: String,
    status: String, 
    restarts: u32,
    uptime: String,
    cpu: String,
    mem: String,
    last_start: String,
}

pub async fn handle_list(workspace: &Workspace) -> Result<()> {
    info!("列出所有运行中的进程");
    
    // 创建表格
    let mut table = Table::new();
    table.add_row(row![
        "NAME", "PID", "MONITOR", "STATUS", "RESTARTS", "UPTIME", "CPU", "MEM", "LAST START"
    ]);

    // 扫描processes目录
    let processes_dir = workspace.get_processes_dir();
    let mut entries = tokio::fs::read_dir(processes_dir).await?;
    
    // 收集进程信息
    let mut process_list = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let process_name = path.file_name()
                .and_then(|n| n.to_str())
                .context("无效的进程名称")?;

            // 读取进程状态
            if let Ok(state) = ProcessState::load(workspace, process_name) {
                let (status, uptime, cpu, mem) = if let Some(pid) = state.pid {
                    get_process_info(pid).await?
                } else {
                    ("stopped".into(), "0".into(), "0%".into(), "0MB".into())
                };

                // 获取最后启动时间
                let last_start = if let Ok(metadata) = tokio::fs::metadata(&path.join("state.json")).await {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                            chrono::DateTime::<chrono::Local>::from(
                                UNIX_EPOCH + duration
                            ).format("%Y-%m-%d %H:%M:%S").to_string()
                        } else {
                            "Unknown".into()
                        }
                    } else {
                        "Unknown".into()
                    }
                } else {
                    "Unknown".into()
                };

                // 添加进程信息到列表
                process_list.push(ProcessInfo {
                    name: process_name.to_string(),
                    pid: state.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    monitor_pid: state.monitor_pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    status,
                    restarts: state.restart_count,
                    uptime,
                    cpu,
                    mem,
                    last_start,
                });
            }
        }
    }

    // 按进程名排序
    process_list.sort_by(|a, b| a.name.cmp(&b.name));

    // 添加排序后的进程信息到表格
    for info in process_list {
        table.add_row(row![
            info.name,
            info.pid,
            info.monitor_pid,
            info.status,
            info.restarts,
            info.uptime,
            info.cpu,
            info.mem,
            info.last_start
        ]);
    }

    // 打印表格
    table.printstd();
    Ok(())
}

#[cfg(windows)]
async fn get_process_info(pid: i32) -> Result<(String, String, String, String)> {
    // 检查进程是否存在并获取进程名
    let status = Command::new("C:\\Windows\\System32\\tasklist.exe")
        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
        .output()
        .await
        .context("执行 tasklist 命令失败")?;

    if status.stdout.is_empty() || String::from_utf8_lossy(&status.stdout).contains("No tasks") {
        return Ok(("stopped".into(), "0".into(), "0%".into(), "0MB".into()));
    }

    // 解析进程名
    let output_str = String::from_utf8_lossy(&status.stdout);
    let parts: Vec<&str> = output_str.split(',').collect();
    if parts.is_empty() {
        return Ok(("online".into(), "N/A".into(), "N/A".into(), "N/A".into()));
    }
    
    // 去除引号获取进程名
    let process_name = parts[0].trim_matches('"');

    // 获取进程启动时间
    let uptime_output = Command::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            &format!("$p = Get-Process -Id {}; $uptime = (Get-Date) - $p.StartTime; Write-Output $uptime.TotalSeconds", pid)
        ])
        .output()
        .await
        .context("执行 PowerShell 命令获取进程运行时间失败")?;

    let uptime_str = String::from_utf8_lossy(&uptime_output.stdout);
    let uptime = if uptime_str.trim().is_empty() {
        "N/A".into()
    } else {
        let seconds = uptime_str.trim().parse::<f64>().unwrap_or(0.0) as u64;
        format_uptime(seconds)
    };

    // 获取CPU使用率
    let cpu_output = Command::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Get-Process -Id {} | Select-Object -ExpandProperty CPU", pid)
        ])
        .output()
        .await
        .context("执行 PowerShell 命令获取 CPU 使用率失败")?;

    let cpu_str = String::from_utf8_lossy(&cpu_output.stdout);
    let cpu = if cpu_str.trim().is_empty() {
        "N/A".into()
    } else {
        let cpu_value = cpu_str.trim().parse::<f64>().unwrap_or(0.0);
        format!("{:.1}%", cpu_value.max(0.0))
    };

    // 获取内存使用
    let mem_output = Command::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Get-Process -Id {} | Select-Object -ExpandProperty WorkingSet64", pid)
        ])
        .output()
        .await
        .context("执行 PowerShell 命令失败")?;

    let mem_str = String::from_utf8_lossy(&mem_output.stdout);
    let mem = if mem_str.trim().is_empty() {
        "N/A".into()
    } else {
        let bytes = mem_str.trim().parse::<u64>().unwrap_or(0);
        format!("{}MB", bytes / 1024 / 1024)
    };

    Ok(("online".into(), uptime, cpu, mem))
}

// 添加格式化运行时间的函数
fn format_uptime(seconds: u64) -> String {
    let days = seconds / (24 * 3600);
    let hours = (seconds % (24 * 3600)) / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes) 
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(unix)]
async fn get_process_info(pid: i32) -> Result<(String, String, String, String)> {
    // 检查进程是否存在
    if !nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok() {
        return Ok(("stopped".into(), "0".into(), "0%".into(), "0MB".into()));
    }

    // 获取进程信息
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pid,etime,%cpu,%mem", "--no-headers"])
        .output()
        .await?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.split_whitespace().collect();
    
    if parts.len() >= 4 {
        Ok((
            "online".into(),
            parts[1].to_string(),
            format!("{}%", parts[2]),
            format!("{}%", parts[3])
        ))
    } else {
        Ok(("online".into(), "N/A".into(), "N/A".into(), "N/A".into()))
    }
} 