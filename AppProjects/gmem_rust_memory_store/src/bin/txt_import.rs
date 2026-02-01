use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use serde_json;

/// TXT文件导入工具
/// 功能：读取TXT格式的规则文件，按章节导入为记忆

/// 章节结构
#[derive(Debug, Clone)]
struct Section {
    title: String,
    content: String,
    level: usize,
}

/// 解析TXT文件为章节列表
///
/// # 参数
/// * `content` - TXT文件内容
///
/// # 返回
/// 章节列表
fn parse_txt_file(content: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_section = Section {
        title: String::new(),
        content: String::new(),
        level: 0,
    };
    let mut has_content = false;

    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('#') {
            if has_content {
                sections.push(current_section.clone());
            }
            
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let title = trimmed[level..].trim().to_string();
            
            current_section = Section {
                title,
                content: String::new(),
                level,
            };
            has_content = false;
        } else if !trimmed.is_empty() {
            if !current_section.content.is_empty() {
                current_section.content.push('\n');
            }
            current_section.content.push_str(line);
            has_content = true;
        }
    }

    if has_content {
        sections.push(current_section);
    }

    sections
}

/// 生成记忆文本
///
/// # 参数
/// * `section` - 章节结构
///
/// # 返回
/// 记忆文本
fn generate_memory_text(section: &Section) -> String {
    if section.level == 0 {
        section.content.clone()
    } else {
        format!("{} - {}", section.title, section.content)
    }
}

/// 生成标签
///
/// # 参数
/// * `section` - 章节结构
/// * `file_name` - 文件名
///
/// # 返回
/// 标签列表
fn generate_tags(section: &Section, _file_name: &str) -> Vec<String> {
    let mut tags = vec![
        "gmem".to_string(),
        "txt".to_string(),
        "import".to_string(),
        "files".to_string(),
    ];

    let title_lower = section.title.to_lowercase();
    let content_lower = section.content.to_lowercase();

    if title_lower.contains("规则") || content_lower.contains("规则") {
        tags.push("rules".to_string());
    }

    if title_lower.contains("生产") || content_lower.contains("生产") {
        tags.push("production".to_string());
    }

    if title_lower.contains("测试") || content_lower.contains("测试") {
        tags.push("test".to_string());
    }

    if title_lower.contains("临时") || content_lower.contains("临时") {
        tags.push("temp".to_string());
    }

    if title_lower.contains("文件") || content_lower.contains("文件") {
        tags.push("files".to_string());
    }

    if title_lower.contains("说明") || content_lower.contains("说明") {
        tags.push("docs".to_string());
    }

    if title_lower.contains("配置") || content_lower.contains("配置") {
        tags.push("config".to_string());
    }

    if title_lower.contains("容器") || content_lower.contains("容器") || content_lower.contains("docker") {
        tags.push("docker".to_string());
    }

    if title_lower.contains("wsl") || content_lower.contains("wsl") {
        tags.push("wsl".to_string());
    }

    tags
}

/// 导入记忆到系统
///
/// # 参数
/// * `text` - 记忆文本
/// * `tags` - 标签列表
///
/// # 返回
/// 操作结果
pub fn import_memory(text: &str, tags: &[String]) -> Result<(), String> {
    let tags_str = tags.join(", ");
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
    
    let mcp_server_path = "V:/git_data/GmemWorker/GmemWorker/bin/gmemory_mcp_server.exe";
    let bin_dir = "V:/git_data/GmemWorker/GmemWorker/bin";
    
    let mut cmd = Command::new(mcp_server_path)
        .current_dir(bin_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("执行命令失败: {}", e))?;
    
    if let Some(stdin) = &mut cmd.stdin {
        std::io::Write::write_all(stdin, json_payload.as_bytes())
            .map_err(|e| format!("写入输入失败: {}", e))?;
    }
    
    let output = cmd.wait_with_output()
        .map_err(|e| format!("等待命令执行失败: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        return Err(format!("命令执行失败: {}", stderr));
    }
    
    if stdout.contains("error") {
        return Err(format!("添加记忆失败: {}", stdout));
    }
    
    Ok(())
}

/// 批量导入章节
///
/// # 参数
/// * `sections` - 章节列表
/// * `file_name` - 文件名
///
/// # 返回
/// (成功数, 失败数)
fn import_sections(sections: &[Section], file_name: &str) -> (usize, usize) {
    let mut success_count = 0;
    let mut fail_count = 0;
    let _remove_lock_path = "V:/git_data/GmemWorker/AppProjects/gmem_rust_memory_store/target/release/remove_lock.exe";
    
    for (index, section) in sections.iter().enumerate() {
        println!("-------------------------------------");
        println!("导入章节 {} / {}", index + 1, sections.len());
        println!("标题: {}", section.title);
        
        let text = generate_memory_text(section);
        let tags = generate_tags(section, file_name);
        
        println!("标签: {}", tags.join(", "));
        
        let lock_file = "E:/GmemWorkerHome/.copilot-memory.lock";
        
        if Path::new(lock_file).exists() {
            println!("发现锁文件,尝试删除...");
            if let Err(e) = std::fs::remove_file(lock_file) {
                println!("警告: 删除锁文件失败: {}", e);
            } else {
                println!("锁文件删除成功!");
            }
        }
        
        match import_memory(&text, &tags) {
            Ok(_) => {
                println!("✓ 导入成功");
                success_count += 1;
            }
            Err(e) => {
                println!("✗ 导入失败: {}", e);
                fail_count += 1;
            }
        }
    }
    
    (success_count, fail_count)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("使用方法: cargo run --bin txt_import -- <txt_file>");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.txt");
    
    println!("读取TXT文件: {}", file_path);
    
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("错误: 无法读取文件: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("解析章节...");
    let sections = parse_txt_file(&content);
    
    println!("找到 {} 个章节", sections.len());
    println!("=====================================");
    
    let (success, fail) = import_sections(&sections, file_name);
    
    println!("=====================================");
    println!("导入完成!");
    println!("成功: {}", success);
    println!("失败: {}", fail);
    println!("总计: {}", sections.len());
}
