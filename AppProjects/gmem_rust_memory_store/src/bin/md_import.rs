use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use serde_json;

// MD文件解析工具
// 功能：读取MD文件，解析标题层级，提取内容，批量导入为记忆

#[derive(Debug)]
pub struct MdSection {
    pub level: usize,      // 标题级别（1-6）
    pub title: String,     // 标题文本
    pub content: String,   // 标题下的内容
    pub parent: Option<usize>, // 父标题索引
    pub children: Vec<usize>, // 子标题索引
}

impl MdSection {
    pub fn new(level: usize, title: String) -> Self {
        Self {
            level,
            title,
            content: String::new(),
            parent: None,
            children: Vec::new(),
        }
    }
}

/// 解析MD文件为章节结构
pub fn parse_md_file(file_path: &str) -> Result<Vec<MdSection>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    
    let mut sections = Vec::new();
    let mut stack = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // 检查是否是标题行
        if line.starts_with('#') {
            // 计算标题级别
            let level = line.chars().take_while(|&c| c == '#').count();
            if level > 0 && level <= 6 {
                // 提取标题文本
                let title = line[level..].trim().to_string();
                
                // 创建新章节
                let section = MdSection::new(level, title);
                let section_index = sections.len();
                
                // 处理层级关系
                while let Some(&last_level) = stack.last() {
                    if last_level >= level {
                        stack.pop();
                    } else {
                        break;
                    }
                }
                
                // 设置父标题
                if let Some(&parent_index) = stack.last() {
                    let parent: &mut MdSection = &mut sections[parent_index];
                    let children: &mut Vec<usize> = &mut parent.children;
                    children.push(section_index);
                    sections.push(section);
                    sections.last_mut().unwrap().parent = Some(parent_index);
                } else {
                    sections.push(section);
                }
                
                stack.push(section_index);
            }
        } else if !sections.is_empty() {
            // 非标题行，添加到当前章节的内容
            let current_index = stack.last().copied().unwrap_or(sections.len() - 1);
            sections[current_index].content.push_str(line);
            sections[current_index].content.push(' ');
        }
    }
    
    Ok(sections)
}

/// 生成记忆文本
pub fn generate_memory_text(section: &MdSection, sections: &[MdSection]) -> String {
    let mut text = String::new();
    
    // 构建完整标题路径
    let mut path = Vec::new();
    let mut current = Some(sections.iter().position(|s| s.title == section.title && s.level == section.level).unwrap());
    
    while let Some(idx) = current {
        path.push(sections[idx].title.clone());
        current = sections[idx].parent;
    }
    
    path.reverse();
    let full_title = path.join(" - ");
    
    // 生成记忆文本
    text.push_str(&full_title);
    text.push_str(": ");
    
    // 添加内容
    let content = section.content.trim();
    if !content.is_empty() {
        text.push_str(content);
    }
    
    text
}

/// 生成标签
pub fn generate_tags(section: &MdSection, sections: &[MdSection]) -> Vec<String> {
    let mut tags = vec!["rules", "md", "import", "gmem"]
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    
    // 检查整个标题路径（包括父章节）是否包含关键词
    let mut full_title = String::new();
    let mut current = Some(sections.iter().position(|s| s.title == section.title && s.level == section.level).unwrap());
    
    while let Some(idx) = current {
        full_title.push_str(&sections[idx].title);
        full_title.push_str(" ");
        current = sections[idx].parent;
    }
    
    let full_title_lower = full_title.to_lowercase();
    let keywords = vec! [
        "rust", "development", "quality", "audit", "cross-platform",
        "reuse", "backup", "gmem", "memory", "libraries", "ci",
        "syntax", "format", "security", "performance", "logic",
        "readability", "extensibility", "clippy", "semver", "documentation",
        "unsafe", "fuzz", "workflow", "critical", "high", "medium", "low",
        "platform", "isolation", "compilation", "testing", "deployment",
        "path", "encoding", "text", "signal", "criteria", "function",
        "trait", "validation", "compatibility", "scope", "frequency",
        "storage", "recovery", "data-type", "metadata", "write", "read",
        "delete", "capacity", "concurrency", "persistence", "error",
        "monitoring", "selection", "serde", "ndarray", "tokio", "rayon",
        "regex", "dicom", "embedded", "dependencies", "version", "custom",
        "review"
    ];
    
    for keyword in keywords {
        if full_title_lower.contains(keyword) {
            tags.push(keyword.to_string());
        }
    }
    
    tags
}

/// 导入记忆到系统
pub fn import_memory(text: &str, tags: &[String]) -> Result<(), String> {
    // 构建JSON请求
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

fn main() {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("用法: md_import <md文件路径>");
        return;
    }
    
    let md_file = &args[1];
    
    // 检查文件是否存在
    if !Path::new(md_file).exists() {
        println!("错误: 文件不存在: {}", md_file);
        return;
    }
    
    // 调用remove_lock工具删除锁文件
    println!("删除锁文件...");
    let remove_lock_path = "V:/git_data/GmemWorker/AppProjects/gmem_rust_memory_store/target/debug/remove_lock.exe";
    
    if Path::new(remove_lock_path).exists() {
        let output = std::process::Command::new(remove_lock_path)
            .output()
            .expect("执行remove_lock工具失败");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        println!("{}", stdout);
        if !stderr.is_empty() {
            println!("警告: {}", stderr);
        }
    } else {
        println!("警告: remove_lock工具不存在，跳过锁文件删除");
        // 尝试直接删除锁文件作为备选方案
        let lock_file = "E:/GmemWorkerHome/.copilot-memory.lock";
        if Path::new(lock_file).exists() {
            println!("发现锁文件,尝试删除...");
            if let Err(e) = std::fs::remove_file(lock_file) {
                println!("警告: 删除锁文件失败: {}", e);
            } else {
                println!("锁文件删除成功!");
            }
        }
    }
    
    // 解析MD文件
    println!("解析MD文件: {}", md_file);
    let sections = match parse_md_file(md_file) {
        Ok(sections) => sections,
        Err(e) => {
            println!("解析失败: {}", e);
            return;
        }
    };
    
    println!("解析完成,发现 {} 章节", sections.len());
    println!("=====================================");
    
    // 导入记忆
    let mut success_count = 0;
    let mut fail_count = 0;
    let remove_lock_path = "V:/git_data/GmemWorker/AppProjects/gmem_rust_memory_store/target/debug/remove_lock.exe";
    
    for (i, section) in sections.iter().enumerate() {
        println!("导入章节 {} / {}: {}", i + 1, sections.len(), section.title);
        
        // 每次导入前删除锁文件
        println!("删除锁文件...");
        if Path::new(remove_lock_path).exists() {
            let output = std::process::Command::new(remove_lock_path)
                .output()
                .expect("执行remove_lock工具失败");
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stdout.is_empty() {
                println!("{}", stdout.trim());
            }
            if !stderr.is_empty() {
                println!("警告: {}", stderr);
            }
        } else {
            // 备选方案：直接删除锁文件
            let lock_file = "E:/GmemWorkerHome/.copilot-memory.lock";
            if Path::new(lock_file).exists() {
                if let Err(e) = std::fs::remove_file(lock_file) {
                    println!("警告: 删除锁文件失败: {}", e);
                } else {
                    println!("锁文件删除成功!");
                }
            }
        }
        
        // 生成记忆文本
        let memory_text = generate_memory_text(section, &sections);
        
        // 生成标签
        let tags = generate_tags(section, &sections);
        
        // 导入记忆
        match import_memory(&memory_text, &tags) {
            Ok(_) => {
                println!("导入成功");
                success_count += 1;
            }
            Err(e) => {
                println!("导入失败: {}", e);
                fail_count += 1;
            }
        }
        
        println!("-------------------------------------");
    }
    
    // 统计结果
    println!("=====================================");
    println!("导入完成!");
    println!("成功: {}", success_count);
    println!("失败: {}", fail_count);
    println!("总章节: {}", sections.len());
}