use crate::scanner::{FileInfo, FileType};
use crate::cleaner::CleanResult;
use crate::utils::{format_file_type, format_file_size, format_timestamp};
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

/// 报告生成器
pub struct ReportGenerator {
    /// 输出文件路径
    output_path: String,
}

impl ReportGenerator {
    /// 创建新的报告生成器
    ///
    /// 参数:
    ///   - output_path: 输出文件路径
    ///
    /// 返回值:
    ///   - 新的报告生成器实例
    pub fn new(output_path: String) -> Self {
        ReportGenerator { output_path }
    }

    /// 生成清理报告
    ///
    /// 参数:
    ///   - result: 清理结果
    ///
    /// 返回值:
    ///   - Ok(()): 报告生成成功
    ///   - Err(String): 错误信息
    pub fn generate_report(&self, result: &CleanResult) -> Result<(), String> {
        let mut output_file = File::create(&self.output_path)
            .map_err(|e| format!("创建报告文件失败: {}", e))?;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

        writeln!(output_file, "C盘清理报告").map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file, "生成时间: {}", timestamp).map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 统计信息
        writeln!(output_file, "统计信息").map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file, "清理文件数: {}", result.cleaned_files.len())
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file, "失败文件数: {}", result.failed_files.len())
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file, "释放空间: {}", format_file_size(result.total_size))
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 按文件类型统计
        writeln!(output_file, "按文件类型统计").map_err(|e| format!("写入报告失败: {}", e))?;
        let mut type_stats: HashMap<FileType, (u64, usize)> = HashMap::new();
        for file_info in &result.cleaned_files {
            let entry = type_stats.entry(file_info.file_type.clone()).or_insert((0u64, 0usize));
            entry.0 += file_info.size;
            entry.1 += 1;
        }

        for (file_type, (size, count)) in &type_stats {
            writeln!(output_file, "{}: {} 个文件, {}",
                format_file_type(file_type),
                count,
                format_file_size(*size)
            ).map_err(|e| format!("写入报告失败: {}", e))?;
        }
        writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 清理的文件列表
        writeln!(output_file, "清理的文件列表").map_err(|e| format!("写入报告失败: {}", e))?;
        for file_info in &result.cleaned_files {
            writeln!(output_file, "{} - {} - {}",
                file_info.path,
                format_file_type(&file_info.file_type),
                format_file_size(file_info.size)
            ).map_err(|e| format!("写入报告失败: {}", e))?;
        }

        // 失败的文件列表
        if !result.failed_files.is_empty() {
            writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;
            writeln!(output_file, "失败的文件列表").map_err(|e| format!("写入报告失败: {}", e))?;
            for (file_info, error) in &result.failed_files {
                writeln!(output_file, "{} - {}", file_info.path, error)
                    .map_err(|e| format!("写入报告失败: {}", e))?;
            }
        }

        Ok(())
    }

    /// 生成扫描报告
    ///
    /// 参数:
    ///   - files: 扫描到的文件列表
    ///
    /// 返回值:
    ///   - Ok(()): 报告生成成功
    ///   - Err(String): 错误信息
    pub fn generate_scan_report(&self, files: &[FileInfo]) -> Result<(), String> {
        let mut output_file = File::create(&self.output_path)
            .map_err(|e| format!("创建报告文件失败: {}", e))?;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

        writeln!(output_file, "C盘扫描报告").map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file, "生成时间: {}", timestamp).map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 统计信息
        writeln!(output_file, "统计信息").map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file, "扫描文件数: {}", files.len())
            .map_err(|e| format!("写入报告失败: {}", e))?;

        let total_size: u64 = files.iter().map(|f| f.size).sum();
        writeln!(output_file, "总大小: {}", format_file_size(total_size))
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 按文件类型统计
        writeln!(output_file, "按文件类型统计").map_err(|e| format!("写入报告失败: {}", e))?;
        let mut type_stats: HashMap<FileType, (u64, usize)> = HashMap::new();
        for file_info in files {
            let entry = type_stats.entry(file_info.file_type.clone()).or_insert((0u64, 0usize));
            entry.0 += file_info.size;
            entry.1 += 1;
        }

        for (file_type, (size, count)) in &type_stats {
            writeln!(output_file, "{}: {} 个文件, {}",
                format_file_type(file_type),
                count,
                format_file_size(*size)
            ).map_err(|e| format!("写入报告失败: {}", e))?;
        }
        writeln!(output_file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 文件列表
        writeln!(output_file, "文件列表").map_err(|e| format!("写入报告失败: {}", e))?;
        for file_info in files {
            writeln!(output_file, "{} - {} - {} - {}",
                file_info.path,
                format_file_type(&file_info.file_type),
                format_file_size(file_info.size),
                format_timestamp(file_info.last_modified)
            ).map_err(|e| format!("写入报告失败: {}", e))?;
        }

        Ok(())
    }
}
