use gmem_rust_memory_store::{MemoryStore, run_repl, load_config, organize_memory, direct_organize, read_memory, process_single_md_file, LockType};
use gmem_rust_memory_store::logs::{init_global_logger, LogConfig, LogLevel};
use gmem_rust_memory_store::config;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut debug_mode = false;
    let mut memory_path: Option<&str> = None;
    let mut organize_mode = false;
    let mut direct_organize_mode = false;
    let mut read_mode = false;
    let mut md_mode = false;
    let mut md_file_path: Option<&str> = None;
    let mut md_temporary = false;
    let mut md_category = "default";
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" | "-d" => {
                debug_mode = true;
                i += 1;
            }
            "--organize" => {
                organize_mode = true;
                i += 1;
            }
            "--direct-organize" => {
                direct_organize_mode = true;
                i += 1;
            }
            "--read" => {
                read_mode = true;
                i += 1;
            }
            "--md" => {
                md_mode = true;
                i += 1;
                if i < args.len() {
                    md_file_path = Some(args[i].as_str());
                    i += 1;
                }
            }
            "--md-temporary" => {
                md_temporary = true;
                i += 1;
            }
            "--md-category" => {
                if i + 1 < args.len() {
                    md_category = args[i + 1].as_str();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--memory-path" => {
                i += 1;
                if i < args.len() {
                    memory_path = Some(args[i].as_str());
                    i += 1;
                }
            }
            _ => {
                // 非标志参数，不处理，留给后续的命令解析
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
    
    let logs_path = config::get_config_path(&config.logs_dir, "logs/debug", Some(exe_dir));
    
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
    
    // 处理记忆整理模式
    if organize_mode {
        if let Err(e) = organize_memory() {
            eprintln!("Error organizing memory: {}", e);
            std::process::exit(1);
        }
        return;
    }
    
    // 处理直接整理模式
    if direct_organize_mode {
        if let Err(e) = direct_organize() {
            eprintln!("Error direct organizing memory: {}", e);
            std::process::exit(1);
        }
        return;
    }
    
    // 处理记忆读取模式
    if read_mode {
        if let Err(e) = read_memory() {
            eprintln!("Error reading memory: {}", e);
            std::process::exit(1);
        }
        return;
    }
    
    // 处理MD文件模式
    if md_mode {
        if let Some(file_path) = md_file_path {
            let final_memory_path = if memory_path.is_some() {
                memory_path
            } else {
                config.memory_path.as_deref()
            };
            if let Err(e) = process_single_md_file(file_path, final_memory_path, md_temporary, md_category) {
                eprintln!("Error processing MD file: {}", e);
                std::process::exit(1);
            }
        } else {
            eprintln!("Error: --md requires a file path");
            std::process::exit(1);
        }
        return;
    }
    
    let final_memory_path = if memory_path.is_some() {
        memory_path
    } else {
        config.memory_path.as_deref()
    };

    // 检查是否有非标志命令行参数
    let has_command_args = args.iter().skip(1).any(|arg| !arg.starts_with("--"));

    // 确定锁类型：交互模式使用Interactive，命令行模式使用Cli
    let lock_type = if has_command_args {
        LockType::Cli
    } else {
        LockType::Interactive
    };

    // 计算锁文件路径
    let lock_path = if let Some(path) = final_memory_path {
        PathBuf::from(path).join(format!("lock{}", lock_type.suffix()))
    } else {
        PathBuf::from("./memory").join(format!("lock{}", lock_type.suffix()))
    };

    let store = MemoryStore::new(final_memory_path, Some(lock_type));
    let version = env!("APP_VERSION");
    
    // 对于交互模式，添加信号处理，在程序退出时删除锁文件
    if lock_type == LockType::Interactive {
        // 设置 Ctrl+C 处理
        ctrlc::set_handler(move || {
            println!("\n正在清理锁文件...");
            if lock_path.exists() {
                if let Err(e) = std::fs::remove_file(&lock_path) {
                    eprintln!("删除锁文件失败: {}", e);
                } else {
                    println!("锁文件已删除");
                }
            }
            std::process::exit(0);
        }).expect("设置信号处理失败");
        
        println!("提示: 按 Ctrl+C 退出程序（会自动清理锁文件）");
    }

    if has_command_args {
        // 构建命令字符串
        let command_str = args.iter().skip(1)
            .filter(|arg| !arg.starts_with("--"))
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(" ");

        if !command_str.is_empty() {
            // 解析并执行命令
            if let Some(parsed) = gmem_rust_memory_store::cli::parse(&command_str) {
                if let Err(e) = gmem_rust_memory_store::cli::execute_command(&store, &parsed) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Invalid command");
                std::process::exit(1);
            }
        } else {
            // 进入交互界面
            if let Err(e) = run_repl(store, debug_mode, version) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // 没有命令参数，进入交互界面
        if let Err(e) = run_repl(store, debug_mode, version) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
