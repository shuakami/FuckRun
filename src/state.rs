use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::fs;
use crate::types::StateConfig;
use crate::workspace::Workspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessState {
    pub pid: Option<i32>,
    pub monitor_pid: Option<i32>,  // 添加monitor_pid字段
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub port: u16,
    pub health_check_url: Option<String>,
}

impl Default for ProcessState {
    fn default() -> Self {
        Self {
            pid: None,
            monitor_pid: None,  // 初始化monitor_pid
            program: String::new(),
            args: Vec::new(),
            working_dir: PathBuf::from("."),
            port: 5000,
            health_check_url: None,
        }
    }
}

impl ProcessState {
    pub fn new(program: String, args: Vec<String>, working_dir: PathBuf, port: u16) -> Self {
        Self {
            pid: None,
            monitor_pid: None,  // 初始化monitor_pid
            program,
            args,
            working_dir,
            port,
            health_check_url: None,
        }
    }

    pub fn save(&self, workspace: &Workspace, process_name: &str) -> Result<()> {
        // 确保进程目录存在
        workspace.ensure_process_dirs(process_name)?;
        
        // 保存状态到文件
        let state_file = workspace.get_process_state_file(process_name);
        let content = serde_json::to_string(self).context("序列化状态失败")?;
        fs::write(&state_file, content).context("保存状态文件失败")?;
        
        Ok(())
    }

    pub fn load(workspace: &Workspace, process_name: &str) -> Result<Self> {
        let state_file = workspace.get_process_state_file(process_name);
        
        let content = fs::read_to_string(&state_file).context("读取状态文件失败")?;
        let state = serde_json::from_str(&content).context("解析状态失败")?;
        Ok(state)
    }

    /// 更新为已停止状态
    pub fn update_stopped_state(&mut self) {
        self.pid = None;
        self.monitor_pid = None;  // 清除monitor_pid
    }

    /// 清除状态文件（仅在需要完全清理进程数据时使用）
    pub fn clear(&self, workspace: &Workspace, process_name: &str) -> Result<()> {
        let state_file = workspace.get_process_state_file(process_name);
        
        if let Ok(()) = fs::remove_file(state_file) {
            Ok(())
        } else {
            Ok(()) // 忽略文件不存在的错误
        }
    }
} 