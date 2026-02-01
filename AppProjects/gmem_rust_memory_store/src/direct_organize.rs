use std::fs;
use std::path::PathBuf;
use serde_json;
use glob;
use crate::record::MemoryRecord;
use crate::config::{load_config, get_memory_path};

/// 从所有分类文件中加载记忆
///
/// # 返回
/// 所有记忆记录
fn load_all_records() -> std::io::Result<Vec<MemoryRecord>> {
    let config = load_config(None);
    let output_dir = get_memory_path(&config);
    let mut all_records: Vec<MemoryRecord> = Vec::new();
    let mut record_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    // 读取所有分类文件
    let pattern = format!("{}\\*-global-gmem-recoder.json", output_dir);
    if let Ok(entries) = glob::glob(&pattern) {
        for entry in entries {
            if let Ok(path) = entry {
                if path.is_file() {
                    if let Ok(raw) = fs::read_to_string(&path) {
                        if !raw.trim().is_empty() {
                            if let Ok(records) = serde_json::from_str::<Vec<MemoryRecord>>(&raw) {
                                for record in records {
                                    if !record_ids.contains(&record.id) {
                                        record_ids.insert(record.id.clone());
                                        all_records.push(record);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 读取原始的global-memory-recorder.json文件
    let input_path = format!("{}\\global-memory-recorder.json", output_dir);
    if std::path::Path::new(&input_path).exists() {
        let raw = fs::read_to_string(&input_path)?;
        if !raw.trim().is_empty() {
            if let Ok(records) = serde_json::from_str::<Vec<MemoryRecord>>(&raw) {
                for record in records {
                    if !record_ids.contains(&record.id) {
                        record_ids.insert(record.id.clone());
                        all_records.push(record);
                    }
                }
            }
        }
    }
    
    Ok(all_records)
}

/// 为放错的记忆添加正确的标签
///
/// # 参数
/// * `records` - 记忆记录列表
///
/// # 返回
/// 修正后的记录列表
fn add_correct_tags(records: Vec<MemoryRecord>) -> Vec<MemoryRecord> {
    let mut corrected_records: Vec<MemoryRecord> = Vec::new();
    
    for mut record in records {
        // 检查内容是否包含特定关键词，添加相应的标签
        let text_lower = record.text.to_lowercase();
        
        // 检查是否包含规则相关内容
        if text_lower.contains("规则") || text_lower.contains("规范") {
            if !record.tags.contains(&"rules".to_string()) {
                record.tags.push("rules".to_string());
            }
        }
        
        // 检查是否包含Rust相关内容
        if text_lower.contains("rust") {
            if !record.tags.contains(&"rust".to_string()) {
                record.tags.push("rust".to_string());
            }
        }
        
        // 检查是否包含工作流程相关内容
        if text_lower.contains("流程") || text_lower.contains("workflow") {
            if !record.tags.contains(&"workflow".to_string()) {
                record.tags.push("workflow".to_string());
            }
        }
        
        // 检查是否包含使用相关内容
        if text_lower.contains("使用") || text_lower.contains("usage") {
            if !record.tags.contains(&"usage".to_string()) {
                record.tags.push("usage".to_string());
            }
        }
        
        // 检查是否包含优先级相关内容
        if text_lower.contains("优先级") || text_lower.contains("high") || text_lower.contains("medium") {
            if !record.tags.contains(&"priority".to_string()) {
                record.tags.push("priority".to_string());
            }
        }
        
        corrected_records.push(record);
    }
    
    corrected_records
}

/// 直接整理记忆，按分类保存
///
/// # 返回
/// 操作结果
pub fn direct_organize() -> std::io::Result<()> {
    println!("开始直接整理全局记忆...");
    
    // 1. 读取所有分类文件中的记忆
    let records = load_all_records()?;
    
    // 2. 为放错的记忆添加正确的标签
    let corrected_records = add_correct_tags(records);
    
    println!("加载并修正了 {} 条记忆记录", corrected_records.len());
    
    // 3. 按分类分组
    let mut category_records: std::collections::HashMap<String, Vec<MemoryRecord>> = std::collections::HashMap::new();
    
    for record in corrected_records {
        // 跳过已删除的记录
        if record.deleted_at.is_some() {
            continue;
        }
        
        // 确定分类
        let config = load_config(None);
        let category = crate::config::get_category_for_tags(&config, &record.tags);
        
        // 添加到对应分类
        category_records.entry(category).or_insert(Vec::new()).push(record);
    }
    
    // 4. 保存到各个分类文件
    let config = load_config(None);
    let output_dir = get_memory_path(&config);
    
    for (category, records) in &category_records {
        let file_name = format!("{}-global-gmem-recoder.json", category);
        let file_path = PathBuf::from(&output_dir).join(file_name);
        
        // 保存文件
        let json = serde_json::to_string_pretty(records)?;
        fs::write(&file_path, json)?;
        
        println!("已保存 {} 条记忆到 {}", records.len(), file_path.display());
    }
    
    // 5. 显示整理结果
    println!("\n记忆整理完成！");
    println!("分类统计：");
    for (category, records) in &category_records {
        println!("- {}: {} 条", category, records.len());
    }
    
    println!("\n生成的分类文件：");
    for category in category_records.keys() {
        println!("{}\\{}-global-gmem-recoder.json", output_dir, category);
    }
    
    Ok(())
}
