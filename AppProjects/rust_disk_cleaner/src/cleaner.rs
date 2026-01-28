use std::fs;
use std::path::Path;
use crate::scanner::FileInfo;

/// 文件清理器
pub struct Cleaner {
    /// 是否为预览模式（不实际删除文件）
    dry_run: bool,
    /// 是否输出详细信息
    verbose: bool,
}

impl Cleaner {
    /// 创建新的清理器
    ///
    /// 参数:
    ///   - dry_run: 是否为预览模式
    ///   - verbose: 是否输出详细信息
    ///
    /// 返回值:
    ///   - 新的清理器实例
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Cleaner {
            dry_run,
            verbose,
        }
    }

    /// 清理文件
    ///
    /// 参数:
    ///   - files: 要清理的文件列表
    ///
    /// 返回值:
    ///   - Ok(CleanResult): 清理结果
    ///   - Err(String): 错误信息
    pub fn clean_files(&self, files: &[FileInfo]) -> Result<CleanResult, String> {
        let mut cleaned_files = Vec::new();
        let mut failed_files = Vec::new();
        let mut total_size = 0u64;

        for file in files {
            match self.clean_file(file) {
                Ok(size) => {
                    cleaned_files.push(file.clone());
                    total_size += size;
                }
                Err(e) => {
                    failed_files.push((file.clone(), e));
                }
            }
        }

        Ok(CleanResult {
            cleaned_files,
            failed_files,
            total_size,
        })
    }

    /// 清理单个文件
    ///
    /// 参数:
    ///   - file: 文件信息
    ///
    /// 返回值:
    ///   - Ok(u64): 文件大小
    ///   - Err(String): 错误信息
    fn clean_file(&self, file: &FileInfo) -> Result<u64, String> {
        let path = Path::new(&file.path);

        if self.verbose {
            println!("清理文件: {} (大小: {} bytes)", file.path, file.size);
        }

        if self.dry_run {
            return Ok(file.size);
        }

        fs::remove_file(path)
            .map_err(|e| format!("删除文件失败: {} - {}", file.path, e))?;

        Ok(file.size)
    }

    /// 清空目录
    ///
    /// 参数:
    ///   - path: 目录路径
    ///
    /// 返回值:
    ///   - Ok(u64): 清理的总大小
    ///   - Err(String): 错误信息
    #[allow(dead_code)]
    pub fn clean_directory(&self, path: &str) -> Result<u64, String> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(format!("目录不存在: {}", path.display()));
        }

        let mut total_size = 0u64;
        let entries = fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取文件失败: {}", e))?;
            let file_path = entry.path();

            if file_path.is_dir() {
                total_size += self.clean_directory(file_path.to_str().unwrap())?;
            } else {
                let metadata = fs::metadata(&file_path)
                    .map_err(|e| format!("获取文件元数据失败: {} - {}", file_path.display(), e))?;
                let size = metadata.len();

                if self.verbose {
                    println!("清理文件: {} (大小: {} bytes)", file_path.display(), size);
                }

                if !self.dry_run {
                    fs::remove_file(&file_path)
                        .map_err(|e| format!("删除文件失败: {} - {}", file_path.display(), e))?;
                }

                total_size += size;
            }
        }

        Ok(total_size)
    }
}

/// 清理结果结构体
#[derive(Debug)]
pub struct CleanResult {
    /// 清理成功的文件列表
    pub cleaned_files: Vec<FileInfo>,
    /// 清理失败的文件列表（文件和错误信息）
    pub failed_files: Vec<(FileInfo, String)>,
    /// 清理的总大小（字节）
    pub total_size: u64,
}
