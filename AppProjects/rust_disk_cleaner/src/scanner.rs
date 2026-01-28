use std::fs;
use std::path::Path;

/// 文件信息结构体
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// 文件路径
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 文件类型
    pub file_type: FileType,
    /// 最后修改时间（Unix时间戳）
    pub last_modified: u64,
}

/// 文件类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum FileType {
    /// 临时文件
    TempFile,
    /// 缓存文件
    CacheFile,
    /// 日志文件
    LogFile,
    /// 回收站
    RecycleBin,
    /// 更新备份
    UpdateBackup,
    /// 浏览器缓存
    BrowserCache,
    /// 系统临时文件
    SystemTemp,
    /// 用户临时文件
    UserTemp,
    /// 其他
    Other,
}

/// 文件扫描器
pub struct Scanner {
    /// 扫描规则列表
    rules: Vec<ScanRule>,
    /// 排除路径列表
    exclude_paths: Vec<String>,
}

/// 扫描规则结构体
#[derive(Debug, Clone)]
pub struct ScanRule {
    /// 文件名模式
    pub pattern: String,
    /// 文件类型
    pub file_type: FileType,
    /// 最大文件年龄（天）
    pub max_age_days: Option<u64>,
}

impl Scanner {
    /// 创建新的扫描器
    ///
    /// 参数:
    ///   - rules: 扫描规则列表
    ///   - exclude_paths: 排除路径列表
    ///
    /// 返回值:
    ///   - 新的扫描器实例
    pub fn new(rules: Vec<ScanRule>, exclude_paths: Vec<String>) -> Self {
        Scanner {
            rules,
            exclude_paths,
        }
    }

    /// 扫描指定目录
    ///
    /// 参数:
    ///   - path: 要扫描的目录路径
    ///
    /// 返回值:
    ///   - Ok(Vec<FileInfo>): 扫描到的文件列表
    ///   - Err(String): 错误信息
    pub fn scan_directory(&self, path: &str) -> Result<Vec<FileInfo>, String> {
        let mut files = Vec::new();
        let path = Path::new(path);

        if !path.exists() {
            return Err(format!("路径不存在: {}", path.display()));
        }

        self.scan_recursive(path, &mut files)?;
        Ok(files)
    }

    /// 递归扫描目录
    ///
    /// 参数:
    ///   - path: 要扫描的目录路径
    ///   - files: 文件列表的引用，用于存储扫描结果
    ///
    /// 返回值:
    ///   - Ok(()): 扫描成功
    ///   - Err(String): 错误信息
    fn scan_recursive(&self, path: &Path, files: &mut Vec<FileInfo>) -> Result<(), String> {
        let entries = fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取文件失败: {}", e))?;
            let file_path = entry.path();

            // 检查是否在排除路径中
            if self.is_excluded(&file_path) {
                continue;
            }

            if file_path.is_dir() {
                self.scan_recursive(&file_path, files)?;
            } else if let Some(file_info) = self.check_file(&file_path) {
                files.push(file_info);
            }
        }

        Ok(())
    }

    /// 检查文件是否匹配扫描规则
    ///
    /// 参数:
    ///   - path: 文件路径
    ///
    /// 返回值:
    ///   - Some(FileInfo): 文件信息，如果文件匹配规则
    ///   - None: 文件不匹配规则
    fn check_file(&self, path: &Path) -> Option<FileInfo> {
        let file_name = path.file_name()?.to_str()?;
        let file_path = path.to_str()?;

        for rule in &self.rules {
            if self.match_pattern(file_name, &rule.pattern) {
                let metadata = fs::metadata(path).ok()?;
                let size = metadata.len();
                let modified = metadata.modified().ok()?;
                let last_modified = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();

                // 检查文件年龄
                if let Some(max_age) = rule.max_age_days {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()?
                        .as_secs();
                    let age_days = (current_time - last_modified) / 86400;
                    if age_days < max_age {
                        continue;
                    }
                }

                return Some(FileInfo {
                    path: file_path.to_string(),
                    size,
                    file_type: rule.file_type.clone(),
                    last_modified,
                });
            }
        }

        None
    }

    /// 匹配文件名模式
    ///
    /// 参数:
    ///   - file_name: 文件名
    ///   - pattern: 匹配模式
    ///
    /// 返回值:
    ///   - true: 匹配成功
    ///   - false: 匹配失败
    fn match_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }
        file_name == pattern
    }

    /// 检查路径是否在排除列表中
    ///
    /// 参数:
    ///   - path: 文件路径
    ///
    /// 返回值:
    ///   - true: 路径在排除列表中
    ///   - false: 路径不在排除列表中
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_str().unwrap_or("");
        for exclude_path in &self.exclude_paths {
            if path_str.contains(exclude_path) {
                return true;
            }
        }
        false
    }
}
