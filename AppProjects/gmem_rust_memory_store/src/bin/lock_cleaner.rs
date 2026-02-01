use std::path::Path;
use std::thread;
use std::time::Duration;
use gmem_rust_memory_store::config::{load_config, get_memory_path};
use gmem_rust_memory_store::lock::cleanup_expired_locks;

/// 获取记忆路径
///
/// # 返回
/// 记忆路径
fn get_memory_path_from_config() -> String {
    let config = load_config(None);
    get_memory_path(&config)
}

/// 清理过期的锁文件
///
/// # 参数
/// * `memory_path` - 记忆路径
/// * `max_age_seconds` - 锁文件最大年龄（秒）
///
/// # 返回
/// 清理的锁文件数量
fn clean_locks(memory_path: &str, max_age_seconds: Option<u64>) -> usize {
    let memory_dir = Path::new(memory_path);
    
    if !memory_dir.exists() {
        println!("记忆目录不存在: {}", memory_path);
        return 0;
    }
    
    println!("检查记忆目录: {}", memory_path);
    println!("锁文件最大年龄: {} 秒", max_age_seconds.unwrap_or(300));
    
    let cleaned = cleanup_expired_locks(memory_dir, max_age_seconds);
    
    if cleaned > 0 {
        println!("清理了 {} 个过期锁文件", cleaned);
    } else {
        println!("没有发现过期锁文件");
    }
    
    cleaned
}

/// 定时清理锁文件
///
/// # 参数
/// * `interval_minutes` - 清理间隔（分钟）
/// * `max_age_seconds` - 锁文件最大年龄（秒）
fn run_periodic_cleanup(interval_minutes: Option<u64>, max_age_seconds: Option<u64>) {
    let interval = interval_minutes.unwrap_or(5);
    let memory_path = get_memory_path_from_config();
    
    println!("启动定时锁文件清理工具");
    println!("清理间隔: {} 分钟", interval);
    println!("记忆路径: {}", memory_path);
    println!("按 Ctrl+C 停止\n");
    
    loop {
        clean_locks(&memory_path, max_age_seconds);
        
        println!("等待 {} 分钟后再次检查...\n", interval);
        thread::sleep(Duration::from_secs(interval * 60));
    }
}

/// 主函数
fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let mut interval_minutes: Option<u64> = None;
    let mut max_age_seconds: Option<u64> = None;
    let mut once_mode = false;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--interval" => {
                i += 1;
                if i < args.len() {
                    interval_minutes = args[i].parse().ok();
                    i += 1;
                }
            }
            "--max-age" => {
                i += 1;
                if i < args.len() {
                    max_age_seconds = args[i].parse().ok();
                    i += 1;
                }
            }
            "--once" => {
                once_mode = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    if once_mode {
        // 单次清理模式
        let memory_path = get_memory_path_from_config();
        clean_locks(&memory_path, max_age_seconds);
    } else {
        // 定时清理模式
        run_periodic_cleanup(interval_minutes, max_age_seconds);
    }
}
