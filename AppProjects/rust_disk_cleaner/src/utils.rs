use crate::scanner::FileType;

/// 格式化文件大小
///
/// 参数:
///   - size: 文件大小（字节）
///
/// 返回值:
///   - String: 格式化后的文件大小字符串
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// 格式化文件类型
///
/// 参数:
///   - file_type: 文件类型
///
/// 返回值:
///   - &str: 文件类型的中文名称
pub fn format_file_type(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::TempFile => "临时文件",
        FileType::CacheFile => "缓存文件",
        FileType::LogFile => "日志文件",
        FileType::RecycleBin => "回收站",
        FileType::UpdateBackup => "更新备份",
        FileType::BrowserCache => "浏览器缓存",
        FileType::SystemTemp => "系统临时文件",
        FileType::UserTemp => "用户临时文件",
        FileType::Other => "其他",
    }
}

/// 格式化时间戳
///
/// 参数:
///   - timestamp: Unix时间戳
///
/// 返回值:
///   - String: 格式化后的时间字符串
pub fn format_timestamp(timestamp: u64) -> String {
    use std::time::{UNIX_EPOCH, Duration};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    let datetime: chrono::DateTime<chrono::Local> = datetime.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 计算文件年龄（天）
///
/// 参数:
///   - timestamp: Unix时间戳
///
/// 返回值:
///   - u64: 文件年龄（天）
#[allow(dead_code)]
pub fn calculate_file_age_days(timestamp: u64) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let age_seconds = current_time.saturating_sub(timestamp);
    age_seconds / 86400
}

/// 创建目录（如果不存在）
///
/// 参数:
///   - path: 目录路径
///
/// 返回值:
///   - Ok(()): 创建成功
///   - Err(String): 错误信息
pub fn ensure_directory_exists(path: &str) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(path);

    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("创建目录失败: {} - {}", path.display(), e))?;
    }

    Ok(())
}

/// 验证路径是否存在
///
/// 参数:
///   - path: 路径
///
/// 返回值:
///   - true: 路径存在
///   - false: 路径不存在
#[allow(dead_code)]
pub fn path_exists(path: &str) -> bool {
    use std::path::Path;
    Path::new(path).exists()
}

/// 获取当前时间戳
///
/// 返回值:
///   - u64: 当前Unix时间戳
pub fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
