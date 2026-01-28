use gmem_rust_memory_store::{MemoryStore, run_repl, load_config};
use gmem_rust_memory_store::logs::{init_global_logger, LogConfig, LogLevel};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut debug_mode = false;
    let mut memory_path: Option<&str> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" | "-d" => {
                debug_mode = true;
                i += 1;
            }
            _ => {
                if memory_path.is_none() {
                    memory_path = Some(args[i].as_str());
                }
                i += 1;
            }
        }
    }
    
    let config = load_config(None);
    
    // 从配置文件中读取debug_enabled参数，如果命令行没有指定
    if !debug_mode {
        debug_mode = config.debug_enabled.unwrap_or(false);
    }
    
    // 初始化日志系统
    let exe_path = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
    
    let logs_dir = config.logs_dir.as_deref()
        .unwrap_or("logs/debug")
        .to_string();
    let logs_path = if Path::new(&logs_dir).is_absolute() {
        PathBuf::from(logs_dir)
    } else {
        exe_dir.join(logs_dir)
    };
    
    let log_config = LogConfig {
        enabled: config.logs_enabled.unwrap_or(false),
        logs_dir: logs_path,
        max_size: config.logs_max_size.unwrap_or(1048576), // 1MB
        level: LogLevel::from(config.logs_level.as_deref().unwrap_or("info")),
        debug_mode: debug_mode,
    };
    
    if let Err(e) = init_global_logger(log_config) {
        eprintln!("Failed to initialize logger: {}", e);
    }
    
    let final_memory_path = if memory_path.is_some() {
        memory_path
    } else {
        config.memory_path.as_deref()
    };

    let store = MemoryStore::new(final_memory_path);
    let version = env!("APP_VERSION");

    if let Err(e) = run_repl(store, debug_mode, version) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
