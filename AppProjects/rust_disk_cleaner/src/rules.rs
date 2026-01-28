use crate::scanner::{ScanRule, FileType};

/// 获取默认清理规则
///
/// 返回值:
///   - Vec<ScanRule>: 默认清理规则列表
pub fn get_default_rules() -> Vec<ScanRule> {
    vec![
        // 临时文件
        ScanRule {
            pattern: "*.tmp".to_string(),
            file_type: FileType::TempFile,
            max_age_days: Some(7),
        },
        ScanRule {
            pattern: "*.temp".to_string(),
            file_type: FileType::TempFile,
            max_age_days: Some(7),
        },

        // 缓存文件
        ScanRule {
            pattern: "*.cache".to_string(),
            file_type: FileType::CacheFile,
            max_age_days: Some(30),
        },
        ScanRule {
            pattern: "*.cach".to_string(),
            file_type: FileType::CacheFile,
            max_age_days: Some(30),
        },

        // 日志文件
        ScanRule {
            pattern: "*.log".to_string(),
            file_type: FileType::LogFile,
            max_age_days: Some(30),
        },

        // 更新备份文件
        ScanRule {
            pattern: "*.old".to_string(),
            file_type: FileType::UpdateBackup,
            max_age_days: Some(90),
        },
        ScanRule {
            pattern: "*.bak".to_string(),
            file_type: FileType::UpdateBackup,
            max_age_days: Some(90),
        },

        // 浏览器缓存
        ScanRule {
            pattern: "Cache".to_string(),
            file_type: FileType::BrowserCache,
            max_age_days: Some(30),
        },

        // 系统临时文件
        ScanRule {
            pattern: "~*".to_string(),
            file_type: FileType::SystemTemp,
            max_age_days: Some(7),
        },
    ]
}

/// 获取排除路径
///
/// 返回值:
///   - Vec<String>: 排除路径列表
pub fn get_exclude_paths() -> Vec<String> {
    vec![
        "Windows".to_string(),
        "Program Files".to_string(),
        "Program Files (x86)".to_string(),
        "ProgramData".to_string(),
        "$Recycle.Bin".to_string(),
        "System Volume Information".to_string(),
    ]
}

/// 获取特定目录的清理规则
///
/// 参数:
///   - directory: 目录路径
///
/// 返回值:
///   - Vec<ScanRule>: 该目录的清理规则列表
#[allow(dead_code)]
pub fn get_directory_rules(directory: &str) -> Vec<ScanRule> {
    match directory {
        "C:\\Windows\\Temp" => vec![
            ScanRule {
                pattern: "*".to_string(),
                file_type: FileType::SystemTemp,
                max_age_days: Some(7),
            },
        ],
        "C:\\Users" => vec![
            ScanRule {
                pattern: "*.tmp".to_string(),
                file_type: FileType::UserTemp,
                max_age_days: Some(7),
            },
            ScanRule {
                pattern: "*.log".to_string(),
                file_type: FileType::LogFile,
                max_age_days: Some(30),
            },
        ],
        _ => get_default_rules(),
    }
}
