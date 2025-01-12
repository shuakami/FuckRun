use anyhow::Result;
use log::{info, error};
use tokio::process::Command;
use std::process::Stdio;
use chrono::Local;
use crate::workspace::Workspace;

pub async fn handle_logs(
    workspace: &Workspace,
    name: String,
    follow: bool,
    log_type: String,
    date: Option<String>,
) -> Result<()> {
    let date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let log_path = if log_type == "stderr" {
        workspace.get_process_dir(&name)
            .join("logs")
            .join(&date)
            .join("stderr.log")
    } else {
        workspace.get_process_dir(&name)
            .join("logs")
            .join(&date)
            .join("stdout.log")
    };

    if !log_path.exists() {
        error!("日志文件不存在: {:?}", log_path);
        anyhow::bail!("日志文件不存在");
    }

    #[cfg(windows)]
    {
        let mut cmd = if follow {
            let mut cmd = Command::new("powershell");
            cmd.args([
                "Get-Content",
                "-Path",
                &log_path.to_string_lossy(),
                "-Tail",
                "1000",
                "-Wait"
            ]);
            cmd
        } else {
            let mut cmd = Command::new("powershell");
            cmd.args([
                "Get-Content",
                "-Path",
                &log_path.to_string_lossy(),
            ]);
            cmd
        };
        cmd.stdout(Stdio::inherit())
           .stderr(Stdio::inherit());
        let mut child = cmd.spawn()?;
        child.wait().await?;
    }

    #[cfg(unix)]
    {
        let mut cmd = if follow {
            let mut cmd = Command::new("tail");
            cmd.args(["-f", "-n", "1000"]);
            cmd
        } else {
            let mut cmd = Command::new("less");
            cmd.args(["+F"]);
            cmd
        };
        cmd.arg(&log_path)
           .stdout(Stdio::inherit())
           .stderr(Stdio::inherit());
        let mut child = cmd.spawn()?;
        child.wait().await?;
    }

    Ok(())
}

pub async fn handle_system_logs(
    workspace: &Workspace,
    follow: bool,
    date: Option<String>,
) -> Result<()> {
    let date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let log_path = workspace.get_fuckrun_dir()
        .join("logs")
        .join(&date)
        .join("fuckrun.log");

    if !log_path.exists() {
        error!("日志文件不存在: {:?}", log_path);
        anyhow::bail!("日志文件不存在");
    }

    #[cfg(windows)]
    {
        // 设置UTF-8编码
        let _ = Command::new("chcp")
            .arg("65001")
            .output();

        let mut cmd = if follow {
            let mut cmd = Command::new("powershell");
            cmd.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Get-Content -Path '{}' -Tail 1000 -Wait -Encoding UTF8", log_path.display())
            ]);
            cmd
        } else {
            let mut cmd = Command::new("powershell");
            cmd.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("Get-Content -Path '{}' -Encoding UTF8", log_path.display())
            ]);
            cmd
        };
        cmd.stdout(Stdio::inherit())
           .stderr(Stdio::inherit());
        let mut child = cmd.spawn()?;
        child.wait().await?;
    }

    #[cfg(unix)]
    {
        let mut cmd = if follow {
            let mut cmd = Command::new("tail");
            cmd.args(["-f", "-n", "1000"]);
            cmd
        } else {
            let mut cmd = Command::new("less");
            cmd.args(["+F"]);
            cmd
        };
        cmd.arg(&log_path)
           .stdout(Stdio::inherit())
           .stderr(Stdio::inherit());
        let mut child = cmd.spawn()?;
        child.wait().await?;
    }

    Ok(())
} 