use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 记忆整理定时器工具（常驻版本）
/// 功能：常驻系统，按指定间隔自动运行记忆整理工具，确保没有重复定义的规则，每条规则都在正确分类下
/// 使用方法：
///   - organize_timer.exe <间隔小时数>  : 常驻模式，每30分钟检查一次，间隔小时数后执行整理
///   - organize_timer.exe once        : 单次执行模式，执行一次整理后退出（可与常驻模式共存）

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
                    if let Some(path) = line.split('=').nth(1) {
                        let trimmed = path.trim().trim_matches('"').trim_matches('\'');
                        let resolved = expand_env_vars(trimmed);
                        if !resolved.is_empty() {
                            let mut result = PathBuf::from(&resolved);
                            result.push(".organize_timestamp");
                            return result.to_str().unwrap_or_else(|| ".organize_timestamp").to_string();
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
                // 使用相对路径（相对于可执行文件目录）
                let mut result = exe_dir.join("..").join("..").join("GmemWorkerHome");
                result.push(".organize_timestamp");
                result.to_str().unwrap_or_else(|| ".organize_timestamp").to_string()
            }
        }
    }
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
                            return result.to_str().unwrap_or_else(|| ".organize_timer.lock").to_string();
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
            result.to_str().unwrap_or_else(|| ".organize_timer.lock").to_string()
        }
        None => {
            // 尝试使用环境变量
            if let Ok(env_path) = std::env::var("GmemWorkerHome") {
                let mut result = PathBuf::from(&env_path);
                result.push(".organize_timer.lock");
                result.to_str().unwrap_or_else(|| ".organize_timer.lock").to_string()
            } else {
                // 使用相对路径（相对于可执行文件目录）
                let mut result = exe_dir.join("..").join("..").join("GmemWorkerHome");
                result.push(".organize_timer.lock");
                result.to_str().unwrap_or_else(|| ".organize_timer.lock").to_string()
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

/// 检查进程锁文件是否存在
///
/// # 返回
/// 是否存在锁文件
fn lock_file_exists() -> bool {
    let lock_file = get_lock_file();
    Path::new(&lock_file).exists()
}

/// 创建进程锁文件
///
/// # 返回
/// 操作结果
fn create_lock_file() -> Result<(), String> {
    let lock_file = get_lock_file();
    fs::write(&lock_file, std::process::id().to_string())
        .map_err(|e| format!("创建锁文件失败: {}", e))
}

/// 获取上次运行时间
///
/// # 参数
/// * `timestamp_file` - 时间戳文件路径
///
/// # 返回
/// 上次运行时间（Unix时间戳）
fn get_last_run_time(timestamp_file: &str) -> Option<u64> {
    if !Path::new(timestamp_file).exists() {
        return None;
    }
    
    let content = match fs::read_to_string(timestamp_file) {
        Ok(content) => content,
        Err(_) => return None,
    };
    
    content.trim().parse().ok()
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

/// 检查是否需要运行整理
///
/// # 参数
/// * `last_run_time` - 上次运行时间
/// * `interval_hours` - 间隔小时数
///
/// # 返回
/// 是否需要运行
fn should_run_organize(last_run_time: Option<u64>, interval_hours: u64) -> bool {
    match last_run_time {
        Some(last_time) => {
            let current_time = get_current_timestamp();
            let elapsed_hours = (current_time - last_time) / 3600;
            elapsed_hours >= interval_hours
        }
        None => true,
    }
}

/// 运行记忆整理工具
///
/// # 返回
/// 操作结果
fn run_organize_tool() -> Result<(), String> {
    let tool_path = get_gmemory_store_path();
    let exe_dir = get_exe_dir();
    
    println!("\n[{}] 运行记忆整理工具...", get_formatted_time());
    println!("工具路径: {}", tool_path.display());
    println!("工作目录: {}", exe_dir.display());
    
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
    
    println!("{}", stdout);
    
    Ok(())
}

/// 获取格式化的当前时间
///
/// # 返回
/// 格式化的时间字符串
fn get_formatted_time() -> String {
    let timestamp = get_current_timestamp();
    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 单次执行模式
///
/// # 返回
/// 操作结果
fn run_once() -> Result<(), String> {
    println!("========================================");
    println!("记忆整理工具（单次执行模式）");
    println!("========================================");
    
    if lock_file_exists() {
        println!("\n[{}] 检测到常驻定时器正在运行", get_formatted_time());
        println!("将触发立即整理...\n");
    } else {
        println!("\n[{}] 未检测到常驻定时器", get_formatted_time());
        println!("执行单次整理...\n");
    }
    
    match run_organize_tool() {
        Ok(_) => {
            let timestamp_file = get_timestamp_file();
            let current_time = get_current_timestamp();
            if let Err(e) = save_current_time(&timestamp_file, current_time) {
                println!("警告: 保存运行时间失败: {}", e);
            } else {
                println!("========================================");
                println!("[{}] 记忆整理完成!", get_formatted_time());
                println!("时间戳文件: {}", timestamp_file);
                println!("========================================");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// 常驻模式
///
/// # 参数
/// * `interval_hours` - 整理间隔小时数
fn run_daemon(interval_hours: u64) {
    let check_interval_minutes = 30u64;
    let timestamp_file = get_timestamp_file();
    let lock_file = get_lock_file();
    
    if lock_file_exists() {
        println!("警告: 检测到另一个定时器进程正在运行");
        println!("请先停止现有进程，或使用 'once' 模式");
        println!("锁文件路径: {}", lock_file);
        std::process::exit(1);
    }
    
    if let Err(e) = create_lock_file() {
        println!("错误: {}", e);
        std::process::exit(1);
    }
    
    println!("========================================");
    println!("记忆整理定时器工具（常驻版本）");
    println!("========================================");
    println!("整理间隔: {} 小时", interval_hours);
    println!("检查间隔: {} 分钟", check_interval_minutes);
    println!("时间戳文件: {}", timestamp_file);
    println!("锁文件: {}", lock_file);
    println!("========================================");
    println!("按 Ctrl+C 退出程序");
    println!("========================================\n");
    
    loop {
        let last_run_time = get_last_run_time(&timestamp_file);
        
        if should_run_organize(last_run_time, interval_hours) {
            println!("\n[{}] 检查结果: 需要运行记忆整理", get_formatted_time());
            println!("========================================");
            
            match run_organize_tool() {
                Ok(_) => {
                    let current_time = get_current_timestamp();
                    if let Err(e) = save_current_time(&timestamp_file, current_time) {
                        println!("警告: 保存运行时间失败: {}", e);
                    } else {
                        println!("========================================");
                        println!("[{}] 记忆整理完成!", get_formatted_time());
                        println!("下次整理时间: {} 小时后", interval_hours);
                        println!("========================================\n");
                    }
                }
                Err(e) => {
                    println!("[{}] 错误: {}", get_formatted_time(), e);
                    println!("将在下次检查时重试...\n");
                }
            }
        } else {
            let last_time = last_run_time.unwrap();
            let current_time = get_current_timestamp();
            let elapsed_hours = (current_time - last_time) / 3600;
            let remaining_hours = interval_hours - elapsed_hours;
            
            println!("[{}] 检查结果: 暂时不需要运行记忆整理", get_formatted_time());
            println!("上次整理: {} 小时前", elapsed_hours);
            println!("距离下次整理: {} 小时", remaining_hours);
        }
        
        println!("[{}] 等待 {} 分钟后再次检查...\n", get_formatted_time(), check_interval_minutes);
        
        thread::sleep(Duration::from_secs(check_interval_minutes * 60));
    }
}

/// 主函数
fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("========================================");
        println!("记忆整理定时器工具");
        println!("========================================");
        println!("使用方法:");
        println!("  1. 单次执行模式: organize_timer.exe once");
        println!("  2. 常驻模式:   organize_timer.exe <间隔小时数>");
        println!("");
        println!("示例:");
        println!("  organize_timer.exe once          # 执行一次整理后退出");
        println!("  organize_timer.exe 24            # 常驻运行，每24小时整理一次");
        println!("  organize_timer.exe 12            # 常驻运行，每12小时整理一次");
        println!("");
        println!("说明:");
        println!("  - 'once' 模式可以在常驻定时器运行时使用");
        println!("  - 常驻模式会创建锁文件防止重复运行");
        println!("========================================");
        std::process::exit(1);
    }
    
    let first_arg = &args[1];
    
    if first_arg.to_lowercase() == "once" {
        if let Err(e) = run_once() {
            println!("错误: {}", e);
            std::process::exit(1);
        }
    } else {
        let interval_hours: u64 = match first_arg.parse() {
            Ok(h) if h > 0 => h,
            _ => {
                println!("错误: 间隔时间必须为正整数");
                println!("使用方法: organize_timer.exe <间隔小时数>");
                println!("示例: organize_timer.exe 24");
                std::process::exit(1);
            }
        };
        
        run_daemon(interval_hours);
    }
}
