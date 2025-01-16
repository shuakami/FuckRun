use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use std::fs;
use std::collections::HashMap;
use crate::types::{ProcessConfig as TypesProcessConfig, FsConfig, StateConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// 进程名称
    pub name: String,
    /// 可执行文件路径
    pub program: String,
    /// 命令行参数
    #[serde(default)]
    pub args: Vec<String>,
    /// 工作目录
    #[serde(default = "default_working_dir")]
    pub working_dir: PathBuf,
    /// 环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// 自动重启
    #[serde(default)]
    pub auto_restart: bool,
    /// 启动延迟（秒）
    #[serde(default)]
    pub start_delay: u64,
    /// 最大重启次数
    #[serde(default = "default_max_restarts")]
    pub max_restarts: u32,
    /// 依赖的其他进程
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// 健康检查URL
    pub health_check_url: Option<String>,
    /// 日志配置
    #[serde(default)]
    pub log: LogConfig,
    /// 进程管理配置
    #[serde(default)]
    pub process: TypesProcessConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志文件路径
    pub file: Option<PathBuf>,
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 最大文件大小（MB）
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    /// 保留的日志文件数量
    #[serde(default = "default_max_files")]
    pub max_files: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// 全局配置
    #[serde(default)]
    pub global: GlobalConfig,
    /// 进程配置
    pub processes: HashMap<String, ProcessConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// 全局工作目录
    pub working_dir: Option<PathBuf>,
    /// 全局环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// 全局日志配置
    #[serde(default)]
    pub log: LogConfig,
    /// 文件系统配置
    #[serde(default)]
    pub fs: FsConfig,
    /// 状态管理配置
    #[serde(default)]
    pub state: StateConfig,
    /// 进程管理配置
    #[serde(default)]
    pub process: TypesProcessConfig,
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string().replace('\\', "/");
        let path = Path::new(&path_str);
        log::info!("尝试加载配置文件: {:?}", path);
        
        if !path.exists() {
            log::error!("配置文件不存在: {:?}", path);
            if let Some(parent) = path.parent() {
                log::info!("父目录是否存在: {}", parent.exists());
                if parent.exists() {
                    log::info!("父目录内容:");
                    if let Ok(entries) = fs::read_dir(parent) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                log::info!("  - {:?}", entry.path().to_string_lossy().to_string().replace('\\', "/"));
                            }
                        }
                    }
                }
            }
            return Err(anyhow::anyhow!("配置文件不存在: {}", path.display()));
        }
        
        log::info!("配置文件存在，准备读取");
        let content = fs::read_to_string(path)
            .context("读取配置文件失败")?;
        
        let config = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&content).context("解析JSON配置文件失败")?
        } else {
            serde_yaml::from_str(&content).context("解析YAML配置文件失败")?
        };

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = if path.as_ref().extension().map_or(false, |ext| ext == "json") {
            serde_json::to_string_pretty(self).context("序列化为JSON失败")?
        } else {
            serde_yaml::to_string(self).context("序列化为YAML失败")?
        };

        fs::write(path, content).context("保存配置文件失败")?;
        Ok(())
    }

    /// 获取进程的完整配置（合并全局配置）
    pub fn get_process_config(&self, name: &str) -> Option<ProcessConfig> {
        let mut config = self.processes.get(name)?.clone();
        
        // 合并全局工作目录
        if config.working_dir == PathBuf::from(".") {
            if let Some(ref dir) = self.global.working_dir {
                config.working_dir = dir.clone();
            }
        }

        // 合并全局环境变量
        for (key, value) in &self.global.env {
            config.env.entry(key.clone())
                .or_insert_with(|| value.clone());
        }

        // 合并日志配置
        if config.log.file.is_none() {
            config.log.file = self.global.log.file.clone();
        }
        if config.log.level == "info" {
            config.log.level = self.global.log.level.clone();
        }
        if config.log.max_size == 100 {
            config.log.max_size = self.global.log.max_size;
        }
        if config.log.max_files == 5 {
            config.log.max_files = self.global.log.max_files;
        }

        Some(config)
    }

    /// 查找配置文件
    pub fn find_config_file(process_name: Option<&str>, workspace: &crate::workspace::Workspace) -> PathBuf {
        if let Some(name) = process_name {
            // 1. 检查进程目录
            let process_config = workspace.get_app_dir()
                .join(name)
                .join("config.yaml");
            if process_config.exists() {
                return process_config;
            }

            // 2. 检查.fuckrun进程目录
            let fuckrun_config = workspace.get_process_dir(name)
                .join("config.yaml");
            if fuckrun_config.exists() {
                return fuckrun_config;
            }
        }

        // 3. 使用根目录配置
        workspace.get_root_dir().join("config.yaml")
    }

    /// 获取配置文件路径
    pub fn get_config_path(&self, process_name: Option<&str>, workspace: &crate::workspace::Workspace) -> Result<PathBuf> {
        Ok(Self::find_config_file(process_name, workspace))
    }
}

fn default_working_dir() -> PathBuf {
    PathBuf::from(".")
}

fn default_max_restarts() -> u32 {
    3
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_size() -> u64 {
    100
}

fn default_max_files() -> u32 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config() -> Result<()> {
        let config = Config {
            global: GlobalConfig {
                working_dir: Some(PathBuf::from("/app")),
                env: {
                    let mut map = HashMap::new();
                    map.insert("RUST_LOG".to_string(), "info".to_string());
                    map
                },
                log: LogConfig {
                    file: Some(PathBuf::from("app.log")),
                    level: "debug".to_string(),
                    max_size: 200,
                    max_files: 10,
                },
                fs: FsConfig::default(),
                state: StateConfig::default(),
                process: TypesProcessConfig::default(),
            },
            processes: {
                let mut map = HashMap::new();
                map.insert("web".to_string(), ProcessConfig {
                    name: "web".to_string(),
                    program: "python".to_string(),
                    args: vec!["app.py".to_string()],
                    working_dir: PathBuf::from("."),
                    env: HashMap::new(),
                    auto_restart: true,
                    start_delay: 0,
                    max_restarts: 3,
                    depends_on: vec![],
                    health_check_url: Some("http://localhost:8000/health".to_string()),
                    log: LogConfig {
                        file: None,
                        level: "info".to_string(),
                        max_size: 100,
                        max_files: 5,
                    },
                    process: TypesProcessConfig::default(),
                });
                map
            },
        };

        // 测试JSON序列化
        let dir = tempdir()?;
        let json_path = dir.path().join("config.json");
        config.save_to_file(&json_path)?;
        let loaded = Config::from_file(&json_path)?;
        assert_eq!(
            loaded.global.working_dir,
            Some(PathBuf::from("/app"))
        );

        // 测试YAML序列化
        let yaml_path = dir.path().join("config.yaml");
        config.save_to_file(&yaml_path)?;
        let loaded = Config::from_file(&yaml_path)?;
        assert_eq!(
            loaded.processes["web"].health_check_url,
            Some("http://localhost:8000/health".to_string())
        );

        // 测试配置合并
        let process_config = loaded.get_process_config("web").unwrap();
        assert_eq!(process_config.working_dir, PathBuf::from("/app"));
        assert_eq!(process_config.log.level, "debug");
        assert_eq!(process_config.log.max_size, 200);

        Ok(())
    }
} 