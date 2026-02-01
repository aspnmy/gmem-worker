use crate::store::MemoryStore;
use crate::config::{load_config, get_memory_path};
use crate::lock::LockType;

/// 读取并显示所有记忆
pub fn read_memory() -> std::io::Result<()> {
    println!("开始读取全局记忆...");
    
    // 从配置文件读取记忆路径
    let config = load_config(None);
    let memory_path = get_memory_path(&config);
    
    // 创建记忆存储实例，使用配置文件中的路径
    let store = MemoryStore::new(Some(&memory_path), Some(LockType::Cli));
    
    // 加载所有记忆
    let records = store.load()?;
    
    println!("成功加载了 {} 条记忆记录", records.len());
    println!("========================================");
    
    // 显示每条记忆
    for (index, record) in records.iter().enumerate() {
        // 跳过已删除的记录
        if record.deleted_at.is_some() {
            continue;
        }
        
        println!("记忆 #{}:", index + 1);
        println!("ID: {}", record.id);
        println!("内容: {}", record.text);
        println!("标签: {:?}", record.tags);
        println!("关键词: {:?}", record.keywords);
        println!("创建时间: {}", record.created_at);
        println!("更新时间: {}", record.updated_at);
        println!("========================================");
    }
    
    println!("记忆读取完成！");
    Ok(())
}
