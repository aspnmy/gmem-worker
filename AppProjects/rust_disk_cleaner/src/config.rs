use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// 配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 扫描路径列表
    pub scan_paths: Vec<String>,
    /// 排除路径列表
    pub exclude_paths: Vec<String>,
    /// 文件最大年龄（天）
    pub max_age_days: u64,
    /// 最小文件大小（字节）
    pub min_file_size: u64,
    /// 是否为预览模式
    pub dry_run: bool,
    /// 是否输出详细信息
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        // 使用Windows环境变量获取系统临时目录
        let system_temp = std::env::var("TEMP")
            .or_else(|_| std::env::var("TMP"))
            .unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());

        Config {
            scan_paths: vec![
                system_temp.clone(),
                "C:\\Users".to_string(),
                "C:\\ProgramData\\Microsoft\\Windows\\WER".to_string(),
                "C:\\Windows\\SoftwareDistribution\\Download".to_string(),
                "C:\\Windows\\Logs".to_string(),
                "C:\\Windows\\Prefetch".to_string(),
            ],
            exclude_paths: vec![
                "Windows\\System32".to_string(),
                "Windows\\SysWOW64".to_string(),
                "Program Files".to_string(),
                "Program Files (x86)".to_string(),
                "ProgramData\\Packages".to_string(),
                "$Recycle.Bin".to_string(),
                "System Volume Information".to_string(),
                "Users\\*\\Documents".to_string(),
                "Users\\*\\Desktop".to_string(),
                "Users\\*\\Pictures".to_string(),
                "Users\\*\\Music".to_string(),
                "Users\\*\\Videos".to_string(),
            ],
            max_age_days: 30,
            min_file_size: 0,
            dry_run: true,
            verbose: true,
        }
    }
}

impl Config {
    /// 展开环境变量
    ///
    /// 参数:
    ///   - value: 包含环境变量占位符的字符串
    ///
    /// 返回值:
    ///   - String: 展开环境变量后的字符串
    fn expand_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        
        // 支持的环境变量列表
        let env_vars = vec![
            ("TEMP", "TEMP"),
            ("TMP", "TMP"),
            ("USERPROFILE", "USERPROFILE"),
            ("APPDATA", "APPDATA"),
            ("LOCALAPPDATA", "LOCALAPPDATA"),
            ("PROGRAMFILES", "PROGRAMFILES"),
            ("PROGRAMFILES(X86)", "PROGRAMFILES(X86)"),
        ];
        
        // 替换 ${VAR_NAME} 格式
        for (env_name, _) in env_vars {
            let pattern = format!("${{{}}}", env_name);
            if let Ok(env_value) = std::env::var(env_name) {
                result = result.replace(&pattern, &env_value);
            }
        }
        
        result
    }

    /// 从文件加载配置
    ///
    /// 参数:
    ///   - path: 配置文件路径
    ///
    /// 返回值:
    ///   - Ok(Config): 配置对象
    ///   - Err(String): 错误信息
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(format!("配置文件不存在: {}", path.display()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        // 展开环境变量
        let mut expanded_config = config.clone();
        expanded_config.scan_paths = config.scan_paths
            .iter()
            .map(|p| Self::expand_env_vars(p))
            .collect();
        expanded_config.exclude_paths = config.exclude_paths
            .iter()
            .map(|p| Self::expand_env_vars(p))
            .collect();

        Ok(expanded_config)
    }

    /// 保存配置到文件
    ///
    /// 参数:
    ///   - path: 配置文件路径
    ///
    /// 返回值:
    ///   - Ok(()): 保存成功
    ///   - Err(String): 错误信息
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        Ok(())
    }

    /// 确保配置文件存在，如果不存在则创建默认配置
    ///
    /// 参数:
    ///   - path: 配置文件路径
    ///
    /// 返回值:
    ///   - Ok(()): 配置文件已存在或创建成功
    ///   - Err(String): 错误信息
    pub fn ensure_config_file(path: &str) -> Result<(), String> {
        let config_path = Path::new(path);
        
        // 如果配置文件已存在，直接返回
        if config_path.exists() {
            return Ok(());
        }
        
        // 配置文件不存在，创建目录
        let config_dir = config_path.parent()
            .ok_or_else(|| "无法获取配置文件目录".to_string())?;
        
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)
                .map_err(|e| format!("创建配置目录失败: {} - {}", config_dir.display(), e))?;
        }
        
        // 硬编码的默认配置内容
        let default_config_content = r#"# C盘清理工具配置文件

# 扫描路径列表（按优先级排序）
# 注意：${TEMP} 会被替换为系统临时目录的环境变量值
scan_paths = [
    # Windows系统临时目录（使用环境变量）
    "${TEMP}",

    # 用户目录（包含所有用户数据）
    "C:\\Users",

    # 程序数据目录
    "C:\\ProgramData\\Microsoft\\Windows\\WER",

    # Windows更新缓存
    "C:\\Windows\\SoftwareDistribution\\Download",

    # Windows日志文件
    "C:\\Windows\\Logs",

    # Windows prefetch缓存
    "C:\\Windows\\Prefetch",
]

# 排除路径列表（不会被扫描的路径）
exclude_paths = [
    # Windows系统目录
    "Windows\\System32",
    "Windows\\SysWOW64",

    # 程序安装目录
    "Program Files",
    "Program Files (x86)",

    # 程序数据目录
    "ProgramData\\Packages",

    # 回收站
    "$Recycle.Bin",

    # 系统卷信息
    "System Volume Information",

    # 用户重要数据
    "Users\\*\\Documents",
    "Users\\*\\Desktop",
    "Users\\*\\Pictures",
    "Users\\*\\Music",
    "Users\\*\\Videos",
]

# 文件最大年龄（天）
max_age_days = 30

# 最小文件大小（字节，0表示不限制）
min_file_size = 0

# 是否为预览模式（true表示不实际删除文件）
dry_run = true

# 是否输出详细信息
verbose = true
"#;
        
        // 写入配置文件
        fs::write(path, default_config_content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        
        println!("已创建默认配置文件: {}", path);
        
        Ok(())
    }
}
