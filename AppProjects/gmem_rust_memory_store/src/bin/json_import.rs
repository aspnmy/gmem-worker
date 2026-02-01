use std::fs;
use std::process::{Command, Stdio};
use serde_json;

// JSON记忆导入工具
// 功能：读取JSON格式的记忆文件，批量导入到记忆系统中

#[derive(Debug, serde::Deserialize)]
struct MemoryRecord {
    id: String,
    text: String,
    tags: Vec<String>,
    #[serde(rename = "createdAt")]
    _created_at: String,
    #[serde(rename = "updatedAt")]
    _updated_at: String,
    #[serde(rename = "deletedAt")]
    deleted_at: Option<String>,
}

/// 读取JSON记忆文件
fn read_json_file(file_path: &str) -> Result<Vec<MemoryRecord>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    
    let records: Vec<MemoryRecord> = serde_json::from_str(&content)
        .map_err(|e| format!("JSON解析失败: {}", e))?;
    
    Ok(records)
}

/// 转换标签为逗号分隔的字符串
pub fn tags_to_string(tags: &[String]) -> String {
    tags.join(", ")
}

/// 导入记忆到系统
pub fn import_memory(text: &str, tags: &[String]) -> Result<(), String> {
    // 构建JSON请求
    let tags_str = tags_to_string(tags);
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "add_memory",
            "arguments": {
                "text": text,
                "tags": tags_str
            }
        }
    });
    
    let json_payload = serde_json::to_string(&request)
        .map_err(|e| format!("JSON序列化失败: {}", e))?;
    
    // 调用gmemory_mcp_server.exe
    let mcp_server_path = "V:/git_data/GmemWorker/GmemWorker/bin/gmemory_mcp_server.exe";
    let bin_dir = "V:/git_data/GmemWorker/GmemWorker/bin";
    
    let mut cmd = Command::new(mcp_server_path)
        .current_dir(bin_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("执行命令失败: {}", e))?;
    
    // 写入输入
    if let Some(stdin) = &mut cmd.stdin {
        std::io::Write::write_all(stdin, json_payload.as_bytes())
            .map_err(|e| format!("写入输入失败: {}", e))?;
    }
    
    // 等待命令执行完成
    let output = cmd.wait_with_output()
        .map_err(|e| format!("等待命令执行失败: {}", e))?;
    
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("导入失败: {}", stderr))
    }
}

/// 批量导入记忆
fn import_memories(records: &[MemoryRecord]) -> (usize, usize, usize) {
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut skip_count = 0;
    
    for (index, record) in records.iter().enumerate() {
        println!("-------------------------------------");
        println!("导入记忆 {} / {}", index + 1, records.len());
        
        // 检查是否已删除
        if record.deleted_at.is_some() {
            println!("跳过已删除的记忆: {}", record.id);
            skip_count += 1;
            continue;
        }
        
        // 导入记忆
        match import_memory(&record.text, &record.tags) {
            Ok(_) => success_count += 1,
            Err(e) => {
                println!("导入失败: {}", e);
                fail_count += 1;
            }
        }
    }
    
    (success_count, fail_count, skip_count)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("使用方法: cargo run --bin json_import -- <json_file>");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    
    println!("读取JSON文件: {}", file_path);
    
    let records = match read_json_file(file_path) {
        Ok(records) => records,
        Err(e) => {
            println!("错误: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("找到 {} 条记忆", records.len());
    println!("=====================================");
    
    let (success, fail, skip) = import_memories(&records);
    
    println!("=====================================");
    println!("导入完成!");
    println!("成功: {}", success);
    println!("失败: {}", fail);
    println!("跳过: {}", skip);
    println!("总计: {}", records.len());
}