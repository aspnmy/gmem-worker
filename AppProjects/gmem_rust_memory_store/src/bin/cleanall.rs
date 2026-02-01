use std::fs;
use std::path::Path;
use std::env;

/// 从.env.toml文件中读取memory_path配置
fn read_memory_path() -> String {
    // 尝试从多个相对位置读取配置文件
    let config_paths = [
        "./config/.env.toml",
        "./bin/config/.env.toml",
        "../config/.env.toml",
        "../../config/.env.toml",
        "../../../config/.env.toml",
        "../../../../config/.env.toml",
        "./GmemWorker/bin/config/.env.toml",
        "../GmemWorker/bin/config/.env.toml",
        "../../GmemWorker/bin/config/.env.toml"
    ];
    
    for config_path in &config_paths {
        if Path::new(config_path).exists() {
            println!("读取配置文件: {}", config_path);
            
            // 读取配置文件内容
            if let Ok(content) = fs::read_to_string(config_path) {
                // 简单解析memory_path行
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("memory_path = ") {
                        // 提取配置值
                        let value = line.strip_prefix("memory_path = ").unwrap_or("");
                        // 处理引号和环境变量
                        let value = value.trim_matches('"');
                        
                        // 处理环境变量或默认值格式
                        if value.contains("|") {
                            let parts: Vec<&str> = value.split('|').collect();
                            for part in parts {
                                let part = part.trim();
                                if part.starts_with("%") && part.ends_with("%") {
                                    // 尝试获取环境变量
                                    let env_var = part.trim_matches('%');
                                    if let Ok(env_value) = env::var(env_var) {
                                        return env_value;
                                    }
                                } else if !part.is_empty() {
                                    // 使用默认值
                                    let default_value = part.trim_matches('"');
                                    // 检查默认值是否也是环境变量格式
                                    if default_value.starts_with("%") && default_value.ends_with("%") {
                                        let env_var = default_value.trim_matches('%');
                                        if let Ok(env_value) = env::var(env_var) {
                                            return env_value;
                                        }
                                    }
                                    return default_value.to_string();
                                }
                            }
                        } else if value.starts_with("%") && value.ends_with("%") {
                            // 尝试获取环境变量
                            let env_var = value.trim_matches('%');
                            if let Ok(env_value) = env::var(env_var) {
                                return env_value;
                            }
                        } else {
                            return value.to_string();
                        }
                    }
                }
            }
        }
    }
    
    // 默认路径
    println!("未找到配置文件，使用默认路径");
    "E:\\GmemWorkerHome".to_string()
}

/// 清理所有记忆文件
fn clean_all_memories(memory_path: &str) -> std::io::Result<()> {
    let path = Path::new(memory_path);
    
    if !path.exists() {
        println!("记忆路径不存在: {}", memory_path);
        return Ok(());
    }
    
    if path.is_dir() {
        // 清理目录中的所有记忆文件
        println!("清理目录中的记忆文件: {}", memory_path);
        
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();
            
            // 检查是否是记忆文件
            if file_path.is_file() {
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                if file_name.contains("-global-gmem-recoder.json") || file_name == "default-global-gmem-recoder.json" {
                    println!("删除记忆文件: {}", file_path.display());
                    fs::remove_file(file_path)?;
                }
            }
        }
    } else {
        // 清理单个记忆文件
        println!("清理记忆文件: {}", memory_path);
        fs::remove_file(path)?;
    }
    
    println!("清理完成!");
    Ok(())
}

fn main() {
    // 读取memory_path配置
    let memory_path = read_memory_path();
    
    // 清理所有记忆
    match clean_all_memories(&memory_path) {
        Ok(_) => println!("记忆清理成功!"),
        Err(e) => println!("记忆清理失败: {}", e),
    }
    
    println!("操作完成!");
}
