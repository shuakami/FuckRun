use anyhow::{Result, Context};
use log::info;
use prettytable::{Table, row};
use std::time::UNIX_EPOCH;
use tokio::process::Command;
use std::path::PathBuf;
use serde::Serialize;
use crate::workspace::Workspace;
use crate::state::ProcessState;

// 定义进程信息结构体
#[derive(Debug, Serialize)]
struct ProcessInfo {
    name: String,
    pid: String,
    monitor_pid: String,
    status: String,
    restarts: u32,
    uptime: String,
    uptime_seconds: u64,  // 添加原始秒数用于筛选
    cpu: String,
    cpu_float: f64,      // 添加浮点数用于筛选
    mem: String,
    mem_bytes: u64,      // 添加字节数用于筛选
    last_start: String,
}

// 定义筛选选项
#[derive(Debug)]
pub struct ListFilter {
    pub name: Option<String>,
    pub status: Option<String>,
    pub min_uptime: Option<u64>,
    pub max_uptime: Option<u64>,
    pub min_cpu: Option<f64>,
    pub max_cpu: Option<f64>,
    pub min_mem: Option<u64>,
    pub max_mem: Option<u64>,
}

pub async fn handle_list(
    workspace: &Workspace, 
    app_dir: Option<PathBuf>,
    filter: Option<ListFilter>,
    json: bool,
    watch: bool,
) -> Result<()> {
    info!("列出所有运行中的进程");
    
    if watch {
        loop {
            // 清屏
            print!("\x1B[2J\x1B[1;1H");
            // 获取并显示进程列表
            let process_list = get_process_list(workspace, &app_dir, &filter).await?;
            // 输出结果
            output_process_list(&process_list, json)?;
            // 等待1秒
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    } else {
        // 获取进程列表
        let process_list = get_process_list(workspace, &app_dir, &filter).await?;
        // 输出结果
        output_process_list(&process_list, json)?;
    }
    
    Ok(())
}

async fn get_process_list(
    workspace: &Workspace,
    app_dir: &Option<PathBuf>,
    filter: &Option<ListFilter>,
) -> Result<Vec<ProcessInfo>> {
    // 获取进程状态目录
    let processes_dir = workspace.get_processes_dir();
    info!("进程状态目录: {:?}", processes_dir);

    // 如果指定了app_dir，记录日志
    if let Some(app_dir) = app_dir {
        info!("指定的应用程序目录: {:?}", app_dir);
    }

    // 扫描processes目录
    let mut entries = tokio::fs::read_dir(processes_dir).await?;
    
    // 收集进程信息
    let mut process_list = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let process_name = path.file_name()
                .and_then(|n| n.to_str())
                .context("无效的进程名称")?;
            
            info!("检查进程: {}", process_name);

            // 如果指定了app_dir，检查进程是否属于该目录
            if let Some(app_dir) = app_dir {
                let process_app_dir = workspace.get_process_app_dir(process_name);
                info!("进程应用目录: {:?}", process_app_dir);

                // 尝试规范化路径进行比较
                let app_dir = std::fs::canonicalize(app_dir)
                    .with_context(|| format!("无法规范化应用程序目录: {:?}", app_dir))?;
                let process_app_dir = std::fs::canonicalize(&process_app_dir)
                    .with_context(|| format!("无法规范化进程应用目录: {:?}", process_app_dir))?;
                
                info!("规范化后的应用程序目录: {:?}", app_dir);
                info!("规范化后的进程应用目录: {:?}", process_app_dir);

                if process_app_dir != app_dir {
                    info!("进程 {} 不属于指定的应用程序目录，跳过", process_name);
                    continue;
                }
            }

            // 读取进程状态
            if let Ok(state) = ProcessState::load(workspace, process_name) {
                let (status, uptime, uptime_seconds, cpu, cpu_float, mem, mem_bytes) = 
                    if let Some(pid) = state.pid {
                        info!("进程 {} 正在运行，PID: {}", process_name, pid);
                        get_process_info(pid).await?
                    } else {
                        info!("进程 {} 未运行", process_name);
                        ("stopped".into(), "0".into(), 0, "0%".into(), 0.0, "0MB".into(), 0)
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

                // 创建进程信息
                let process_info = ProcessInfo {
                    name: process_name.to_string(),
                    pid: state.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    monitor_pid: state.monitor_pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    status,
                    restarts: state.restart_count,
                    uptime,
                    uptime_seconds,
                    cpu,
                    cpu_float,
                    mem,
                    mem_bytes,
                    last_start,
                };

                // 应用筛选条件
                if let Some(filter) = filter {
                    // 按名称筛选
                    if let Some(name) = &filter.name {
                        if !process_info.name.contains(name) {
                            continue;
                        }
                    }

                    // 按状态筛选
                    if let Some(status) = &filter.status {
                        if &process_info.status != status {
                            continue;
                        }
                    }

                    // 按运行时间筛选
                    if let Some(min_uptime) = filter.min_uptime {
                        if process_info.uptime_seconds < min_uptime {
                            continue;
                        }
                    }
                    if let Some(max_uptime) = filter.max_uptime {
                        if process_info.uptime_seconds > max_uptime {
                            continue;
                        }
                    }

                    // 按CPU使用率筛选
                    if let Some(min_cpu) = filter.min_cpu {
                        if process_info.cpu_float < min_cpu {
                            continue;
                        }
                    }
                    if let Some(max_cpu) = filter.max_cpu {
                        if process_info.cpu_float > max_cpu {
                            continue;
                        }
                    }

                    // 按内存使用筛选
                    if let Some(min_mem) = filter.min_mem {
                        if process_info.mem_bytes < min_mem {
                            continue;
                        }
                    }
                    if let Some(max_mem) = filter.max_mem {
                        if process_info.mem_bytes > max_mem {
                            continue;
                        }
                    }
                }

                info!("添加进程 {} 的信息到列表", process_name);
                process_list.push(process_info);
            } else {
                info!("无法加载进程 {} 的状态", process_name);
            }
        }
    }

    info!("找到 {} 个进程", process_list.len());

    // 按进程名排序
    process_list.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(process_list)
}

fn output_process_list(process_list: &[ProcessInfo], json: bool) -> Result<()> {
    if json {
        // JSON输出
        println!("{}", serde_json::to_string_pretty(process_list)?);
    } else {
        // 表格输出
        let mut table = Table::new();
        table.add_row(row![
            "NAME", "PID", "MONITOR", "STATUS", "RESTARTS", "UPTIME", "CPU", "MEM", "LAST START"
        ]);

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

        table.printstd();
    }

    Ok(())
}

#[cfg(windows)]
async fn get_process_info(pid: i32) -> Result<(String, String, u64, String, f64, String, u64)> {
    // 检查进程是否存在并获取进程名
    let status = Command::new("C:\\Windows\\System32\\tasklist.exe")
        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
        .output()
        .await
        .context("执行 tasklist 命令失败")?;

    if status.stdout.is_empty() || String::from_utf8_lossy(&status.stdout).contains("No tasks") {
        return Ok(("stopped".into(), "0".into(), 0, "0%".into(), 0.0, "0MB".into(), 0));
    }

    // 解析进程名
    let output_str = String::from_utf8_lossy(&status.stdout);
    let parts: Vec<&str> = output_str.split(',').collect();
    if parts.is_empty() {
        return Ok(("online".into(), "N/A".into(), 0, "N/A".into(), 0.0, "N/A".into(), 0));
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
    let uptime_seconds = if uptime_str.trim().is_empty() {
        0
    } else {
        uptime_str.trim().parse::<f64>().unwrap_or(0.0) as u64
    };
    let uptime = format_uptime(uptime_seconds);

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
    let cpu_float = if cpu_str.trim().is_empty() {
        0.0
    } else {
        cpu_str.trim().parse::<f64>().unwrap_or(0.0)
    };
    let cpu = format!("{:.1}%", cpu_float.max(0.0));

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
    let mem_bytes = if mem_str.trim().is_empty() {
        0
    } else {
        mem_str.trim().parse::<u64>().unwrap_or(0)
    };
    let mem = format!("{}MB", mem_bytes / 1024 / 1024);

    Ok(("online".into(), uptime, uptime_seconds, cpu, cpu_float, mem, mem_bytes))
}

#[cfg(unix)]
async fn get_process_info(pid: i32) -> Result<(String, String, u64, String, f64, String, u64)> {
    use nix::unistd::Pid;
    use nix::sys::signal::{kill, Signal};
    // 检查进程是否存在
    if !kill(Pid::from_raw(pid), None).is_ok() {
        return Ok(("stopped".into(), "0".into(), 0, "0%".into(), 0.0, "0MB".into(), 0));
    }

    // 获取进程信息
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pid,etime,%cpu,%mem", "--no-headers"])
        .output()
        .await?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.split_whitespace().collect();
    
    if parts.len() >= 4 {
        // 解析运行时间
        let uptime_str = parts[1].to_string();
        let uptime_seconds = parse_ps_etime(&uptime_str);
        
        // 解析CPU使用率
        let cpu_float = parts[2].trim().parse::<f64>().unwrap_or(0.0);
        let cpu = format!("{}%", parts[2]);
        
        // 解析内存使用率并转换为字节
        let mem_percent = parts[3].trim().parse::<f64>().unwrap_or(0.0);
        let total_mem = get_total_memory()?;
        let mem_bytes = (total_mem as f64 * mem_percent / 100.0) as u64;
        let mem = format!("{}MB", mem_bytes / 1024 / 1024);
        
        Ok(("online".into(), uptime_str, uptime_seconds, cpu, cpu_float, mem, mem_bytes))
    } else {
        Ok(("online".into(), "N/A".into(), 0, "N/A".into(), 0.0, "N/A".into(), 0))
    }
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
fn parse_ps_etime(etime: &str) -> u64 {
    let parts: Vec<&str> = etime.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<u64>().unwrap_or(0),
        2 => {
            let minutes = parts[0].parse::<u64>().unwrap_or(0);
            let seconds = parts[1].parse::<u64>().unwrap_or(0);
            minutes * 60 + seconds
        }
        3 => {
            let hours = parts[0].parse::<u64>().unwrap_or(0);
            let minutes = parts[1].parse::<u64>().unwrap_or(0);
            let seconds = parts[2].parse::<u64>().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        }
        _ => 0
    }
}

#[cfg(unix)]
fn get_total_memory() -> Result<u64> {
    let output = std::process::Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()?;
    let mem_str = String::from_utf8_lossy(&output.stdout);
    Ok(mem_str.trim().parse::<u64>().unwrap_or(0))
} 