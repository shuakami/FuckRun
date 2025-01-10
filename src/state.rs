use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessState {
    pub pid: Option<i32>,
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
            
            program,
            args,
            working_dir,
            port,
            health_check_url: None,
        }
    }

    pub fn save(&self) -> Result<()> {
        // 创建.fuckrun目录
        fs::create_dir_all(".fuckrun").context("创建状态目录失败")?;
        
        // 保存状态到文件
        let content = serde_json::to_string(self).context("序列化状态失败")?;
        fs::write(".fuckrun/state.json", content).context("保存状态文件失败")?;
        
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(".fuckrun/state.json").context("读取状态文件失败")?;
        let state = serde_json::from_str(&content).context("解析状态失败")?;
        Ok(state)
    }

    pub fn clear(&self) -> Result<()> {
        if let Ok(()) = fs::remove_file(".fuckrun/state.json") {
            Ok(())
        } else {
            Ok(()) // 忽略文件不存在的错误
        }
    }
} 