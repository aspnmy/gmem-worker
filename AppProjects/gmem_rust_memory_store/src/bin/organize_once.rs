use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Serialize;
use serde_json;

/// 记忆整理结果
#[derive(Debug, Serialize)]
struct OrganizeResult {
    success: bool,
    message: String,
    timestamp: u64,
    output: String,
}

/// 记忆整理工具（单次执行版本）
/// 功能：执行一次记忆整理后退出，专门给AI助手使用
/// 不涉及任何锁文件检测，直接执行整理并退出

/// 获取当前可执行文件所在目录
///
/// # 返回
/// 可执行文件所在目录的绝对路径
fn get_exe_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap());
    exe_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
}

/// 获取GmemoryStore.exe的路径
///
/// # 返回
/// GmemoryStore.exe的绝对路径
fn get_gmemory_store_path() -> PathBuf {
    get_exe_dir().join("GmemoryStore.exe")
}

/// 获取时间戳文件路径
///
/// # 返回
/// 时间戳文件的绝对路径
fn get_timestamp_file() -> String {
    let exe_dir = get_exe_dir();
    
    // 尝试从配置文件读取记忆路径
    let config_path = exe_dir.join("config").join(".env.toml");
    let memory_path: Option<String> = if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            for line in content.lines() {
                if line.starts_with("memory_path") {
                    if let Some(rest) = line.split('=').nth(1) {
                        let trimmed = rest.trim();
                        // 处理 "path1" | "path2" 格式
                        let paths: Vec<&str> = trimmed.split('|').map(|p| p.trim().trim_matches('"').trim_matches('\'')).collect();
                        
                        // 尝试第一个路径
                        for path in paths {
                            let resolved = expand_env_vars(path);
                            if !resolved.is_empty() {
                                let mut result = PathBuf::from(&resolved);
                                result.push(".organize_timestamp");
                                return result.to_str().unwrap_or_else(|| ".organize_timestamp").to_string();
                            }
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
            result.push(".organize_timestamp");
            result.to_str().unwrap_or_else(|| ".organize_timestamp").to_string()
        }
        None => {
            // 尝试使用环境变量
            if let Ok(env_path) = std::env::var("GmemWorkerHome") {
                let mut result = PathBuf::from(&env_path);
                result.push(".organize_timestamp");
                result.to_str().unwrap_or_else(|| ".organize_timestamp").to_string()
            } else {
                // 使用可执行文件目录
                let mut result = exe_dir;
                result.push(".organize_timestamp");
                result.to_str().unwrap_or_else(|| ".organize_timestamp").to_string()
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

/// 保存当前运行时间
///
/// # 参数
/// * `timestamp_file` - 时间戳文件路径
/// * `timestamp` - 当前时间戳
///
/// # 返回
/// 操作结果
fn save_current_time(timestamp_file: &str, timestamp: u64) -> Result<(), String> {
    fs::write(timestamp_file, timestamp.to_string())
        .map_err(|e| format!("写入时间戳文件失败: {}", e))
}

/// 获取当前时间戳（秒）
///
/// # 返回
/// 当前时间戳
fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 运行记忆整理工具
///
/// # 返回
/// 操作结果
fn run_organize_tool() -> Result<String, String> {
    let tool_path = get_gmemory_store_path();
    let exe_dir = get_exe_dir();
    
    let output = Command::new(&tool_path)
        .current_dir(&exe_dir)
        .arg("--direct-organize")
        .output()
        .map_err(|e| format!("执行记忆整理工具失败: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        return Err(format!("记忆整理工具执行失败: {}", stderr));
    }
    
    Ok(stdout.to_string())
}

/// 主函数
fn main() {
    match run_organize_tool() {
        Ok(output) => {
            let timestamp_file = get_timestamp_file();
            let current_time = get_current_timestamp();
            if let Err(e) = save_current_time(&timestamp_file, current_time) {
                let result = OrganizeResult {
                    success: false,
                    message: format!("保存运行时间失败: {}", e),
                    timestamp: current_time,
                    output: String::new(),
                };
                println!("{}", serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()));
                std::process::exit(1);
            } else {
                let result = OrganizeResult {
                    success: true,
                    message: "记忆整理完成".to_string(),
                    timestamp: current_time,
                    output: output,
                };
                println!("{}", serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()));
            }
        }
        Err(e) => {
            let result = OrganizeResult {
                success: false,
                message: e,
                timestamp: get_current_timestamp(),
                output: String::new(),
            };
            println!("{}", serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()));
            std::process::exit(1);
        }
    }
}
