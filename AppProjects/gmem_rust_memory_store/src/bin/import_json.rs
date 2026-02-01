use std::fs;
use std::path::PathBuf;
use serde_json;
use gmem_rust_memory_store::record::MemoryRecord;
use gmem_rust_memory_store::config::{load_config, get_memory_path};

/// 读取JSON记忆文件
///
/// # 参数
/// * `file_path` - JSON文件路径
///
/// # 返回
/// 记忆记录列表
fn read_json_file(file_path: &str) -> Result<Vec<MemoryRecord>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    
    let records: Vec<MemoryRecord> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON解析失败: {}", e))?;
    
    Ok(records)
}

/// 保存记录到记忆目录
///
/// # 参数
/// * `records` - 记录记录列表
///
/// # 返回
/// 操作结果
fn save_to_memory_directory(records: &[MemoryRecord]) -> Result<(), String> {
    let config = load_config(None);
    let output_dir = get_memory_path(&config);
    
    let file_path = PathBuf::from(&output_dir).join("global-memory-recorder.json");
    
    let json = serde_json::to_string_pretty(records)
        .map_err(|e| format!("JSON序列化失败: {}", e))?;
    
    fs::write(&file_path, json)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    println!("已保存 {} 条记录到 {}", records.len(), file_path.display());
    
    Ok(())
}

/// 导入JSON记忆文件到记忆目录
///
/// # 参数
/// * `file_path` - JSON文件路径
///
/// # 返回
/// 操作结果
pub fn import_json_to_memory(file_path: &str) -> Result<(), String> {
    println!("读取JSON文件: {}", file_path);
    
    let records = read_json_file(file_path)?;
    
    println!("找到 {} 条记录", records.len());
    
    let active_count = records.iter().filter(|r| r.deleted_at.is_none()).count();
    let deleted_count = records.iter().filter(|r| r.deleted_at.is_some()).count();
    
    println!("活跃记录: {}", active_count);
    println!("已删除记录: {}", deleted_count);
    
    save_to_memory_directory(&records)?;
    
    println!("导入完成！请运行 direct_organize 工具进行分类整理。");
    
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("使用方法: cargo run --bin import_json -- <json_file>");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    
    match import_json_to_memory(file_path) {
        Ok(_) => {
            println!("\n导入成功！");
            println!("现在可以运行以下命令进行分类整理：");
            println!("  GmemoryStore.exe --direct-organize");
        }
        Err(e) => {
            println!("错误: {}", e);
            std::process::exit(1);
        }
    }
}
