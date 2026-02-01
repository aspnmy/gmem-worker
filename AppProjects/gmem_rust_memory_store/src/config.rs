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
    /// 记忆分类映射（标签到分类的映射）
    pub category_mapping: Option<std::collections::HashMap<String, String>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut category_mapping = std::collections::HashMap::new();
        
        // 默认记忆分类映射
        category_mapping.insert("rust".to_string(), "rust".to_string());
        category_mapping.insert("git".to_string(), "git".to_string());
        category_mapping.insert("ide".to_string(), "ide".to_string());
        category_mapping.insert("rules".to_string(), "rules".to_string());
        category_mapping.insert("config".to_string(), "config".to_string());
        category_mapping.insert("files".to_string(), "files".to_string());
        category_mapping.insert("directory".to_string(), "directory".to_string());
        category_mapping.insert("wsl".to_string(), "wsl".to_string());
        category_mapping.insert("command-line".to_string(), "command-line".to_string());
        category_mapping.insert("ai_worker".to_string(), "ai_worker".to_string());
        category_mapping.insert("csdn".to_string(), "blog".to_string());
        category_mapping.insert("blog".to_string(), "blog".to_string());
        category_mapping.insert("workflow".to_string(), "workflow".to_string());
        category_mapping.insert("usage".to_string(), "usage".to_string());
        category_mapping.insert("high".to_string(), "priority".to_string());
        category_mapping.insert("medium".to_string(), "priority".to_string());
        category_mapping.insert("markdown".to_string(), "default".to_string());
        category_mapping.insert("file".to_string(), "default".to_string());
        category_mapping.insert("temp".to_string(), "default".to_string());
        
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
            category_mapping: Some(category_mapping),
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

    match toml::from_str::<Config>(&content) {
        Ok(mut config) => {
            // 确保分类映射存在
            if config.category_mapping.is_none() {
                config.category_mapping = Config::default().category_mapping;
            }
            config
        }
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
memory_path = "E:\\GmemWorkerHome"

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

# 记忆分类映射（标签到分类的映射）
# 格式：标签名 = 分类名
# 当添加记忆时，会根据标签自动选择对应的分类文件
[category_mapping]
rust = "rust"
git = "git"
ide = "ide"
rules = "rules"
config = "config"
files = "files"
directory = "directory"
wsl = "wsl"
command-line = "command-line"
ai_worker = "ai_worker"
csdn = "blog"
blog = "blog"
workflow = "workflow"
usage = "usage"
high = "priority"
medium = "priority"
markdown = "default"
file = "default"
temp = "default"
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

/// 根据标签获取分类
///
/// # 参数
/// * `config` - 配置结构体
/// * `tags` - 标签列表
///
/// # 返回
/// 分类名称
pub fn get_category_for_tags(config: &Config, tags: &[String]) -> String {
    let mapping = config.category_mapping.as_ref();
    
    if let Some(mapping) = mapping {
        // 检查是否包含特定标签并映射到相应分类
        for tag in tags {
            if let Some(category) = mapping.get(tag) {
                return category.clone();
            }
        }
    }
    
    // 默认分类
    "default".to_string()
}

/// 获取记忆存储路径
///
/// # 参数
/// * `config` - 配置结构体
///
/// # 返回
/// 记忆存储路径
pub fn get_memory_path(config: &Config) -> String {
    let raw_path = config.memory_path.as_ref()
        .unwrap_or(&"E:\\GmemWorkerHome".to_string())
        .clone();
    
    resolve_config_path_with_fallback(&raw_path)
}

/// 获取日志目录路径
///
/// # 参数
/// * `config` - 配置结构体
///
/// # 返回
/// 日志目录路径
pub fn get_logs_dir(config: &Config) -> String {
    let raw_path = config.logs_dir.as_ref()
        .unwrap_or(&"logs/debug".to_string())
        .clone();
    
    resolve_config_path_with_fallback(&raw_path)
}

/// 解析配置路径，支持环境变量替换和备选值
///
/// # 参数
/// * `raw_path` - 原始配置路径，可能包含环境变量和备选值
///
/// # 返回
/// 解析后的有效路径
fn resolve_config_path_with_fallback(raw_path: &str) -> String {
    let paths: Vec<&str> = raw_path.split('|').collect();
    
    for path in paths {
        let resolved = expand_environment_variables(path.trim());
        if !resolved.is_empty() {
            return resolved;
        }
    }
    
    raw_path.to_string()
}

/// 展开环境变量（支持 %VAR% 格式）
///
/// # 参数
/// * `input` - 输入字符串，可能包含环境变量
///
/// # 返回
/// 展开环境变量后的字符串
fn expand_environment_variables(input: &str) -> String {
    let mut result = input.to_string();
    
    let re = regex::Regex::new(r"%([^%]+)%").unwrap();
    while let Some(caps) = re.captures(&result) {
        if let Some(var_name) = caps.get(1) {
            let var_name_str = var_name.as_str();
            if let Ok(var_value) = std::env::var(var_name_str) {
                result = result.replace(&format!("%{}%", var_name_str), &var_value);
            } else {
                return String::new();
            }
        }
    }
    
    result
}

/// 获取配置字符串值，支持环境变量替换和备选值
///
/// # 参数
/// * `config_value` - 配置值，可能包含环境变量和备选值
/// * `default_value` - 默认值（当配置值无效时使用）
///
/// # 返回
/// 解析后的有效配置值
pub fn get_config_string(config_value: &Option<String>, default_value: &str) -> String {
    match config_value {
        Some(value) => resolve_config_path_with_fallback(value),
        None => default_value.to_string(),
    }
}

/// 获取配置路径值，支持环境变量替换和备选值
///
/// # 参数
/// * `config_value` - 配置值，可能包含环境变量和备选值
/// * `default_value` - 默认值（当配置值无效时使用）
/// * `base_dir` - 基础目录（用于相对路径）
///
/// # 返回
/// 解析后的绝对路径
pub fn get_config_path(config_value: &Option<String>, default_value: &str, base_dir: Option<&Path>) -> PathBuf {
    let resolved = get_config_string(config_value, default_value);
    let path = Path::new(&resolved);
    
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        match base_dir {
            Some(dir) => dir.join(path),
            None => std::env::current_dir().unwrap().join(path),
        }
    }
}
