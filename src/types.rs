use std::path::PathBuf;
use std::time::Duration;
use serde::{Serialize, Deserialize};

/// 进程优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessPriority {
    System = 1,      // 系统进程
    Application = 2, // 应用进程
    Temporary = 3,   // 临时进程
}

/// 文件系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsConfig {
    /// 最大重试次数
    #[serde(default = "default_fs_retries")]
    pub max_retries: u32,
    
    /// 重试延迟(毫秒)
    #[serde(default = "default_fs_retry_delay")]
    pub retry_delay_ms: u64,
    
    /// 默认文件权限
    #[serde(default = "default_file_mode")]
    pub default_mode: u32,

    /// 进程退出等待时间(毫秒)
    #[serde(default = "default_exit_wait_ms")]
    pub exit_wait_ms: u64,

    /// 默认文件权限掩码
    #[serde(default = "default_file_mode")]
    pub default_file_mode: u32,

    /// 默认目录权限掩码
    #[serde(default = "default_dir_mode")]
    pub default_dir_mode: u32,
}

/// 进程管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// 默认端口
    #[serde(default = "default_port")]
    pub default_port: u16,
    
    /// 初始化等待时间(秒)
    #[serde(default = "default_init_wait")]
    pub init_wait_secs: u64,
    
    /// 健康检查超时时间(秒)
    #[serde(default = "default_health_check_timeout")]
    pub health_check_timeout_secs: u64,
    
    /// 健康检查重试次数
    #[serde(default = "default_health_check_retries")]
    pub health_check_retries: u32,
    
    /// 重试间隔时间(秒)
    #[serde(default = "default_retry_interval")]
    pub retry_interval_secs: u64,
    
    /// 优雅关闭超时时间(秒)
    #[serde(default = "default_graceful_shutdown_timeout")]
    pub graceful_shutdown_timeout_secs: u64,

    /// Windows进程创建标志
    #[serde(default = "default_windows_process_flags")]
    pub windows_process_flags: u32,

    /// 进程退出等待时间(毫秒)
    #[serde(default = "default_exit_wait_ms")]
    pub exit_wait_ms: u64,

    /// 默认Python解释器路径
    #[serde(default = "default_python_interpreter")]
    pub default_python_interpreter: String,

    /// 默认Python脚本路径
    #[serde(default = "default_script_path")]
    pub default_script_path: String,
}

/// 状态管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// 状态文件目录
    #[serde(default = "default_state_dir")]
    pub state_dir: PathBuf,
    
    /// 状态文件名
    #[serde(default = "default_state_filename")]
    pub state_filename: String,

    /// 默认工作目录
    #[serde(default = "default_working_dir")]
    pub default_working_dir: PathBuf,
}

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 默认配置文件路径
    #[serde(default = "default_config_paths")]
    pub default_config_paths: Vec<String>,
}

impl Default for FsConfig {
    fn default() -> Self {
        Self {
            max_retries: default_fs_retries(),
            retry_delay_ms: default_fs_retry_delay(),
            default_mode: default_file_mode(),
            exit_wait_ms: default_exit_wait_ms(),
            default_file_mode: default_file_mode(),
            default_dir_mode: default_dir_mode(),
        }
    }
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            default_port: default_port(),
            init_wait_secs: default_init_wait(),
            health_check_timeout_secs: default_health_check_timeout(),
            health_check_retries: default_health_check_retries(),
            retry_interval_secs: default_retry_interval(),
            graceful_shutdown_timeout_secs: default_graceful_shutdown_timeout(),
            windows_process_flags: default_windows_process_flags(),
            exit_wait_ms: default_exit_wait_ms(),
            default_python_interpreter: default_python_interpreter(),
            default_script_path: default_script_path(),
        }
    }
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            state_dir: default_state_dir(),
            state_filename: default_state_filename(),
            default_working_dir: default_working_dir(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_config_paths: default_config_paths(),
        }
    }
}

// 默认值函数
fn default_fs_retries() -> u32 { 3 }
fn default_fs_retry_delay() -> u64 { 100 }
fn default_file_mode() -> u32 { 0o644 }
fn default_dir_mode() -> u32 { 0o755 }
fn default_port() -> u16 { 5000 }
fn default_init_wait() -> u64 { 5 }
fn default_health_check_timeout() -> u64 { 5 }
fn default_health_check_retries() -> u32 { 10 }
fn default_retry_interval() -> u64 { 2 }
fn default_graceful_shutdown_timeout() -> u64 { 3 }
fn default_state_dir() -> PathBuf { PathBuf::from(".fuckrun") }
fn default_state_filename() -> String { String::from("state.json") }
fn default_working_dir() -> PathBuf { PathBuf::from(".") }
fn default_exit_wait_ms() -> u64 { 500 }
fn default_windows_process_flags() -> u32 { 0x00000200 | 0x00000008 } // CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
fn default_python_interpreter() -> String { String::from("python") }
fn default_script_path() -> String { String::from("examples/simple_web.py") }
fn default_config_paths() -> Vec<String> { vec!["config.yaml".to_string(), "config.json".to_string()] }

// 辅助函数
impl ProcessConfig {
    pub fn init_wait(&self) -> Duration {
        Duration::from_secs(self.init_wait_secs)
    }
    
    pub fn health_check_timeout(&self) -> Duration {
        Duration::from_secs(self.health_check_timeout_secs)
    }
    
    pub fn retry_interval(&self) -> Duration {
        Duration::from_secs(self.retry_interval_secs)
    }
    
    pub fn graceful_shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.graceful_shutdown_timeout_secs)
    }

    pub fn exit_wait(&self) -> Duration {
        Duration::from_millis(self.exit_wait_ms)
    }

    /// 验证配置的合法性
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.init_wait_secs == 0 {
            anyhow::bail!("初始化等待时间不能为0");
        }
        if self.health_check_timeout_secs == 0 {
            anyhow::bail!("健康检查超时时间不能为0");
        }
        if self.health_check_retries == 0 {
            anyhow::bail!("健康检查重试次数不能为0");
        }
        if self.retry_interval_secs == 0 {
            anyhow::bail!("重试间隔时间不能为0");
        }
        if self.graceful_shutdown_timeout_secs == 0 {
            anyhow::bail!("优雅关闭超时时间不能为0");
        }
        if self.exit_wait_ms == 0 {
            anyhow::bail!("进程退出等待时间不能为0");
        }
        Ok(())
    }
}

impl FsConfig {
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_ms)
    }

    pub fn exit_wait(&self) -> Duration {
        Duration::from_millis(self.exit_wait_ms)
    }

    /// 验证配置的合法性
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_retries == 0 {
            anyhow::bail!("最大重试次数不能为0");
        }
        if self.retry_delay_ms == 0 {
            anyhow::bail!("重试延迟时间不能为0");
        }
        if self.exit_wait_ms == 0 {
            anyhow::bail!("进程退出等待时间不能为0");
        }
        Ok(())
    }
}

impl ProcessPriority {
    /// 获取优先级的数值
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// 检查是否为系统级进程
    pub fn is_system(&self) -> bool {
        matches!(self, ProcessPriority::System)
    }

    /// 检查是否为应用级进程
    pub fn is_application(&self) -> bool {
        matches!(self, ProcessPriority::Application)
    }

    /// 检查是否为临时进程
    pub fn is_temporary(&self) -> bool {
        matches!(self, ProcessPriority::Temporary)
    }
}

impl StateConfig {
    /// 获取状态文件的完整路径
    pub fn state_file_path(&self) -> PathBuf {
        self.state_dir.join(&self.state_filename)
    }

    /// 验证配置的合法性
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.state_filename.is_empty() {
            anyhow::bail!("状态文件名不能为空");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_priority() {
        let priority = ProcessPriority::System;
        assert_eq!(priority.value(), 1);
        assert!(priority.is_system());
        assert!(!priority.is_application());
        assert!(!priority.is_temporary());
    }

    #[test]
    fn test_state_config() {
        let config = StateConfig::default();
        assert_eq!(config.state_file_path(), PathBuf::from(".fuckrun/state.json"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_process_config_validation() {
        let config = ProcessConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.init_wait_secs = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_fs_config_validation() {
        let config = FsConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.max_retries = 0;
        assert!(invalid_config.validate().is_err());
    }
} 