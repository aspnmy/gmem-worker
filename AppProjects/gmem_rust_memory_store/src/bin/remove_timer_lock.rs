use std::fs;
use std::path::{Path, PathBuf};

/// 删除定时器锁文件工具
/// 功能：删除 organize_timer 的锁文件

/// 获取当前可执行文件所在目录
///
/// # 返回
/// 可执行文件所在目录的绝对路径
fn get_exe_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap());
    exe_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
}

/// 获取锁文件路径
///
/// # 返回
/// 锁文件的绝对路径
fn get_lock_file() -> String {
    let exe_dir = get_exe_dir();
    
    // 尝试从配置文件读取记忆路径
    let config_path = exe_dir.join("config").join(".env.toml");
    let memory_path: Option<String> = if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            for line in content.lines() {
                if line.starts_with("memory_path") {
                    if let Some(path) = line.split('=').nth(1) {
                        let trimmed = path.trim().trim_matches('"').trim_matches('\'');
                        let resolved = expand_env_vars(trimmed);
                        if !resolved.is_empty() {
                            let mut result = PathBuf::from(&resolved);
                            result.push(".organize_timer.lock");
                            return result.to_str().unwrap_or(".organize_timer.lock").to_string();
                        }
                    }
                }
            }
            None
        } else {
            None
        }
    } else {
        None
    };
    
    // 如果配置文件中没有找到，使用默认路径
    match memory_path {
        Some(path) => {
            let mut result = PathBuf::from(&path);
            result.push(".organize_timer.lock");
            result.to_str().unwrap_or(".organize_timer.lock").to_string()
        }
        None => {
            // 尝试使用环境变量
            if let Ok(env_path) = std::env::var("GmemWorkerHome") {
                let mut result = PathBuf::from(&env_path);
                result.push(".organize_timer.lock");
                result.to_str().unwrap_or(".organize_timer.lock").to_string()
            } else {
                // 使用相对路径（相对于可执行文件目录）
                let mut result = exe_dir.join("..").join("..").join("GmemWorkerHome");
                result.push(".organize_timer.lock");
                result.to_str().unwrap_or(".organize_timer.lock").to_string()
            }
        }
    }
}

/// 展开环境变量
///
/// # 参数
/// * `input` - 输入字符串，可能包含环境变量
///
/// # 返回
/// 展开环境变量后的字符串
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    
    // 支持 %VAR% 格式（Windows）
    if let Some(start) = result.find('%') {
        if let Some(end) = result[start+1..].find('%') {
            let var_name = &result[start+1..start+1+end];
            if let Ok(var_value) = std::env::var(var_name) {
                result = result.replace(&format!("%{}%", var_name), &var_value);
            } else {
                return String::new();
            }
        }
    }
    
    result
}

fn main() {
    println!("========================================");
    println!("删除定时器锁文件工具");
    println!("========================================");
    
    let lock_file = get_lock_file();
    println!("锁文件路径: {}", lock_file);
    
    if Path::new(&lock_file).exists() {
        match fs::remove_file(&lock_file) {
            Ok(_) => {
                println!("========================================");
                println!("锁文件删除成功!");
                println!("========================================");
            }
            Err(e) => {
                println!("========================================");
                println!("错误: 删除锁文件失败: {}", e);
                println!("========================================");
                std::process::exit(1);
            }
        }
    } else {
        println!("========================================");
        println!("提示: 锁文件不存在");
        println!("========================================");
    }
}
