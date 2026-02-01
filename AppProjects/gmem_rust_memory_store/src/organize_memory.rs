use crate::store::MemoryStore;
use crate::config::{load_config, get_memory_path};
use crate::lock::LockType;

/// 整理记忆，按分类保存
pub fn organize_memory() -> std::io::Result<()> {
    println!("开始整理全局记忆...");
    
    // 从配置文件读取记忆路径
    let config = load_config(None);
    let memory_path = get_memory_path(&config);
    
    // 1. 首先加载当前的global-memory-recorder.json文件
    let single_file_path = format!("{}\\global-memory-recorder.json", memory_path);
    let single_file_store = MemoryStore::new(Some(&single_file_path), Some(LockType::Cli));
    let records = single_file_store.load()?;
    
    println!("加载了 {} 条记忆记录", records.len());
    
    // 2. 创建目录存储的store实例
    let directory_store = MemoryStore::new(Some(&memory_path), Some(LockType::Cli));
    
    // 3. 按分类重新保存
    let mut category_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for record in records {
        // 跳过已删除的记录
        if record.deleted_at.is_some() {
            continue;
        }
        
        // 确定分类
        let category = crate::config::get_category_for_tags(&config, &record.tags);
        
        // 保存到对应分类
        directory_store.add_memory(&record.text, Some(record.tags.clone()))?;
        
        // 更新分类计数
        *category_counts.entry(category).or_insert(0) += 1;
    }
    
    // 4. 显示整理结果
    println!("\n记忆整理完成！");
    println!("分类统计：");
    for (category, count) in &category_counts {
        println!("- {}: {} 条", category, count);
    }
    
    println!("\n记忆已按分类保存到以下文件：");
    println!("{}\\[category]-global-gmem-recoder.json", memory_path);
    println!("其中 [category] 为对应的分类名称");
    
    // 显示生成的分类文件列表
    println!("\n生成的分类文件：");
    for category in category_counts.keys() {
        println!("- {}\\{}-global-gmem-recoder.json", memory_path, category);
    }
    
    Ok(())
}
