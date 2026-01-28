use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 项目名称
    pub project_name: Option<String>,
    /// DeepSeek API 密钥
    pub deepseek_api_key: Option<String>,
    /// 记忆文件路径
    pub memory_path: Option<String>,
    /// 备份格式
    pub backup_format: Option<String>,
    /// 备份间隔（毫秒）
    pub backup_interval: Option<u64>,
    /// 备份目录
    pub backup_dir: Option<String>,
    /// 最大备份数
    pub max_backups: Option<usize>,
    /// 是否压缩备份
    pub compress_backups: Option<bool>,
    /// 是否启用日志
    pub logs_enabled: Option<bool>,
    /// 日志目录
    pub logs_dir: Option<String>,
    /// 日志文件最大大小（字节）
    pub logs_max_size: Option<u64>,
    /// 日志级别
    pub logs_level: Option<String>,
    /// 是否启用debug模式
    pub debug_enabled: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_name: Some("global-memory-rule".to_string()),
            deepseek_api_key: None,
            memory_path: None,
            backup_format: Some("markdown".to_string()),
            backup_interval: Some(7200000),
            backup_dir: None,
            max_backups: Some(20),
            compress_backups: Some(true),
            logs_enabled: Some(false),
            logs_dir: Some("logs/debug".to_string()),
            logs_max_size: Some(1048576), // 1MB
            logs_level: Some("info".to_string()),
            debug_enabled: Some(false),
        }
    }
}

/// 加载配置文件
///
/// # 参数
/// * `config_path` - 配置文件路径（可选）
///
/// # 返回
/// 配置结构体
pub fn load_config(config_path: Option<&str>) -> Config {
    let config_file = resolve_config_path(config_path);

    if !config_file.exists() {
        create_default_config(&config_file);
        return Config::default();
    }

    let content = match fs::read_to_string(&config_file) {
        Ok(content) => content,
        Err(_) => return Config::default(),
    };

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(_) => Config::default(),
    }
}

/// 创建默认配置文件
///
/// # 参数
/// * `config_file` - 配置文件路径
fn create_default_config(config_file: &Path) {
    let config_dir = config_file.parent().unwrap();
    
    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(config_dir) {
            eprintln!("Warning: Failed to create config directory: {}", e);
            return;
        }
    }

    let default_content = r#"# gmem_rust_memory_store 配置文件

# 项目名称
project_name = "global-memory-rule"

# DeepSeek API 密钥（可选：启用 LLM 压缩功能）
deepseek_api_key = ""

# 记忆文件路径（支持相对路径或绝对路径）
memory_path = ".copilot-memory.json"

# 备份格式
backup_format = "markdown"

# 备份间隔（毫秒）
backup_interval = 7200000

# 备份目录
backup_dir = ""

# 最大备份数
max_backups = 20

# 是否压缩备份
compress_backups = true

# 日志配置
logs_enabled = false
logs_dir = "logs/debug"
logs_max_size = 1048576
logs_level = "info"

# Debug配置
debug_enabled = false
"#;

    if let Err(e) = fs::write(config_file, default_content) {
        eprintln!("Warning: Failed to create default config file: {}", e);
    }
}

/// 解析配置文件路径
///
/// # 参数
/// * `config_path` - 配置文件路径（可选）
///
/// # 返回
/// 配置文件的完整路径
fn resolve_config_path(config_path: Option<&str>) -> PathBuf {
    if let Some(path) = config_path {
        let p = Path::new(path);
        if p.is_absolute() {
            return p.to_path_buf();
        }
        return std::env::current_dir().unwrap().join(path);
    }

    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
    
    exe_dir.join("config").join(".env.toml")
}

/// 获取配置文件的绝对路径
///
/// # 参数
/// * `config_path` - 配置文件路径（可选）
///
/// # 返回
/// 配置文件的绝对路径字符串
pub fn get_config_file_path(config_path: Option<&str>) -> String {
    let config_file = resolve_config_path(config_path);
    config_file.to_str().unwrap_or("").to_string()
}
