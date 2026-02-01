use crate::store::MemoryStore;
use crate::config::{load_config, get_memory_path};
use crate::lock::LockType;
use std::path::{Path, PathBuf};

/// MD文件处理选项
#[derive(Clone)]
pub struct MdProcessorOptions {
    /// 是否为临时文件（临时文件会添加temp前缀）
    pub is_temporary: bool,
    /// 记忆分类
    pub category: String,
    /// 额外标签
    pub additional_tags: Vec<String>,
}

impl Default for MdProcessorOptions {
    fn default() -> Self {
        Self {
            is_temporary: false,
            category: "default".to_string(),
            additional_tags: Vec::new(),
        }
    }
}

/// MD文件处理器
pub struct MdProcessor {
    store: MemoryStore,
}

impl MdProcessor {
    /// 创建新的MD文件处理器
    ///
    /// # 参数
    /// * `memory_path` - 记忆存储路径
    ///
    /// # 返回
    /// MD文件处理器实例
    pub fn new(memory_path: Option<&str>) -> Self {
        Self {
            store: MemoryStore::new(memory_path, Some(LockType::Cli)),
        }
    }

    /// 从MD文件读取内容并添加到记忆库
    ///
    /// # 参数
    /// * `file_path` - MD文件路径
    /// * `options` - 处理选项
    ///
    /// # 返回
    /// 操作结果
    pub fn add_md_to_memory(&self, file_path: &Path, options: MdProcessorOptions) -> std::io::Result<()> {
        println!("开始处理MD文件: {}", file_path.display());
        
        // 读取MD文件内容
        let content = std::fs::read_to_string(file_path)?;
        
        // 生成记忆文本
        let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
        let memory_text = format!("# {} 内容\n\n{}", file_name, content);
        
        // 生成标签
        let mut tags = vec!["markdown".to_string(), "file".to_string()];
        
        // 如果是临时文件，添加temp标签
        if options.is_temporary {
            tags.push("temp".to_string());
        }
        
        // 添加额外标签
        tags.extend(options.additional_tags);
        
        // 检查记忆库中是否已经存在相同的记忆
        let existing_records = self.store.load()?;
        let memory_exists = existing_records.iter().any(|record| {
            record.text == memory_text && record.deleted_at.is_none()
        });
        
        if memory_exists {
            println!("记忆已存在，跳过添加");
            println!("文件: {}", file_path.display());
            println!("分类: {}", options.category);
            return Ok(());
        }
        
        // 添加到记忆库
        self.store.add_memory(&memory_text, Some(tags))?;
        
        println!("成功将MD文件添加到记忆库！");
        println!("文件: {}", file_path.display());
        println!("分类: {}", options.category);
        println!("是否临时: {}", options.is_temporary);
        
        Ok(())
    }

    /// 批量处理目录中的MD文件
    ///
    /// # 参数
    /// * `directory` - 目录路径
    /// * `options` - 处理选项
    ///
    /// # 返回
    /// 处理的文件数量
    pub fn batch_process_md_files(&self, directory: &Path, options: MdProcessorOptions) -> std::io::Result<usize> {
        let mut processed_count = 0;
        
        // 遍历目录中的MD文件
        if let Ok(entries) = std::fs::read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().unwrap_or_default() == "md" {
                        // 处理每个MD文件
                        if self.add_md_to_memory(&path, options.clone()).is_ok() {
                            processed_count += 1;
                        }
                    }
                }
            }
        }
        
        println!("批量处理完成，共处理 {} 个MD文件", processed_count);
        
        Ok(processed_count)
    }
}

/// 便捷函数：处理单个MD文件
///
/// # 参数
/// * `file_path` - MD文件路径
/// * `memory_path` - 记忆存储路径
/// * `is_temporary` - 是否为临时文件
/// * `category` - 记忆分类
///
/// # 返回
/// 操作结果
pub fn process_single_md_file(
    file_path: &str,
    memory_path: Option<&str>,
    is_temporary: bool,
    category: &str,
) -> std::io::Result<()> {
    // 尝试使用普通方式处理
    let processor = MdProcessor::new(memory_path);
    let options = MdProcessorOptions {
        is_temporary,
        category: category.to_string(),
        additional_tags: Vec::new(),
    };
    
    match processor.add_md_to_memory(&PathBuf::from(file_path), options) {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("Timed out acquiring lock") => {
            // 如果是锁超时错误，使用直接处理方式
            println!("锁超时，尝试使用直接处理方式...");
            direct_process_single_md_file(file_path, memory_path, is_temporary, category)
        }
        Err(e) => Err(e),
    }
}

/// 直接处理单个MD文件，绕过锁文件机制
///
/// # 参数
/// * `file_path` - MD文件路径
/// * `memory_path` - 记忆存储路径
/// * `is_temporary` - 是否为临时文件
/// * `category` - 记忆分类
///
/// # 返回
/// 操作结果
fn direct_process_single_md_file(
    file_path: &str,
    _memory_path: Option<&str>,
    is_temporary: bool,
    category: &str,
) -> std::io::Result<()> {
    println!("开始直接处理MD文件: {}", file_path);
    
    // 读取MD文件内容
    let content = std::fs::read_to_string(file_path)?;
    
    // 生成记忆文本
    let file_name = std::path::Path::new(file_path).file_name().unwrap_or_default().to_string_lossy();
    let memory_text = format!("# {} 内容\n\n{}", file_name, content);
    
    // 生成标签
    let mut tags = vec!["markdown".to_string(), "file".to_string()];
    
    // 如果是临时文件，添加temp标签
    if is_temporary {
        tags.push("temp".to_string());
    }
    
    // 提取关键词
    let keywords = crate::keywords::extract_keywords(&memory_text);
    
    // 创建新记录
    let new_record = crate::record::MemoryRecord {
        id: crate::timestamp::make_id(),
        text: memory_text,
        tags,
        keywords,
        created_at: crate::timestamp::now_iso(),
        updated_at: crate::timestamp::now_iso(),
        deleted_at: None,
    };
    
    // 确定存储路径
    let config = load_config(None);
    let base_path = get_memory_path(&config);
    let output_dir = std::path::Path::new(&base_path);
    
    // 确保输出目录存在
    std::fs::create_dir_all(output_dir)?;
    
    // 确定分类文件路径
    let file_name = format!("{}-global-gmem-recoder.json", category);
    let output_file_path = output_dir.join(file_name);
    
    // 读取现有记录（使用明确的文件打开和关闭）
    let mut records: Vec<crate::record::MemoryRecord> = Vec::new();
    if output_file_path.exists() {
        // 尝试读取现有文件，如果失败则忽略
        match std::fs::read_to_string(&output_file_path) {
            Ok(raw) if !raw.trim().is_empty() => {
                match serde_json::from_str(&raw) {
                    Ok(parsed_records) => {
                        records = parsed_records;
                    }
                    Err(e) => {
                        println!("警告: 解析现有记录失败: {}, 将创建新文件", e);
                    }
                }
            }
            Err(e) => {
                println!("警告: 读取现有记录失败: {}, 将创建新文件", e);
            }
            _ => {
                // 文件为空，使用空向量
            }
        }
    }
    
    // 检查是否已经存在相同的记录
    let mut record_exists = false;
    for record in &records {
        if record.text == new_record.text {
            record_exists = true;
            println!("记忆已存在，跳过添加");
            break;
        }
    }
    
    // 如果记录不存在，添加新记录
    if !record_exists {
        records.push(new_record);
        
        // 保存文件（使用临时文件）
        let json = serde_json::to_string_pretty(&records)?;
        
        // 创建临时文件
        let temp_file_path = output_dir.join(format!("temp_{}.json", std::process::id()));
        
        // 写入临时文件
        std::fs::write(&temp_file_path, &json)?;
        
        // 尝试删除目标文件（如果存在）
        if output_file_path.exists() {
            match std::fs::remove_file(&output_file_path) {
                Ok(_) => {
                    println!("已删除现有文件: {}", output_file_path.display());
                }
                Err(e) => {
                    println!("警告: 删除现有文件失败: {}, 将尝试直接重命名", e);
                }
            }
        }
        
        // 尝试重命名临时文件到目标文件
        match std::fs::rename(&temp_file_path, &output_file_path) {
            Ok(_) => {
                println!("成功将MD文件添加到记忆库！");
                println!("文件: {}", file_path);
                println!("分类: {}", category);
                println!("是否临时: {}", is_temporary);
                println!("保存到: {}", output_file_path.display());
            }
            Err(e) => {
                println!("警告: 重命名临时文件失败: {}", e);
                // 尝试直接写入目标文件
                match std::fs::write(&output_file_path, &json) {
                    Ok(_) => {
                        println!("成功将MD文件添加到记忆库！");
                        println!("文件: {}", file_path);
                        println!("分类: {}", category);
                        println!("是否临时: {}", is_temporary);
                        println!("保存到: {}", output_file_path.display());
                    }
                    Err(e) => {
                        println!("错误: 直接写入目标文件失败: {}", e);
                        // 清理临时文件
                        let _ = std::fs::remove_file(&temp_file_path);
                        return Err(e);
                    }
                }
            }
        }
    } else {
        println!("记忆已存在，跳过添加");
    }
    
    Ok(())
}
