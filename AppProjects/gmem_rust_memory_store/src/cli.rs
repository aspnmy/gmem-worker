use std::collections::HashMap;
use std::io::{self, Write};
use crate::store::MemoryStore;
use crate::compress::compress_deterministic;

/// 解析的命令结构
#[derive(Debug)]
pub struct Parsed {
    pub cmd: String,
    pub args: Vec<String>,
    pub opts: HashMap<String, String>,
}

/// 解析命令行字符串
///
/// # 参数
/// * `line` - 命令行字符串
///
/// # 返回
/// 解析后的命令结构，如果输入为空则返回 None
pub fn parse(line: &str) -> Option<Parsed> {
    let tokens = tokenize(line.trim());
    
    if tokens.is_empty() {
        return None;
    }

    let cmd = tokens[0].to_lowercase();
    let mut opts: HashMap<String, String> = HashMap::new();
    let mut args: Vec<String> = Vec::new();

    let mut i = 1;
    while i < tokens.len() {
        let t = &tokens[i];
        if let Some(key) = t.strip_prefix("--") {
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with("--") {
                opts.insert(key.to_string(), tokens[i + 1].clone());
                i += 2;
            } else {
                opts.insert(key.to_string(), String::new());
                i += 1;
            }
        } else {
            args.push(t.clone());
            i += 1;
        }
    }

    Some(Parsed { cmd, args, opts })
}

/// 分词命令行字符串，处理带引号的字符串
///
/// # 参数
/// * `line` - 命令行字符串
///
/// # 返回
/// 分词后的字符串数组
fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    let mut current = String::new();
    
    for c in line.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            '"' | '\'' if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    
    if !current.is_empty() {
        tokens.push(current);
    }
    
    tokens
}

/// 主命令行 REPL 循环
///
/// # 参数
/// * `store` - 记忆存储实例
/// * `debug_mode` - 是否启用debug模式
/// * `version` - 版本号
///
/// # 返回
/// IO 错误（如果有）
pub fn run_repl(store: MemoryStore, debug_mode: bool, version: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Copilot Memory Store CLI v{}", version);
    println!("Type 'help' for available commands, 'exit' to quit");
    if debug_mode {
        println!("Debug mode enabled");
        // 显示配置文件路径
        let config_path = crate::config::get_config_file_path(None);
        println!("Config file: {}", config_path);
        // 显示日志目录路径
        let exe_path = std::env::current_exe().unwrap_or_else(|_| std::env::current_dir().unwrap());
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let logs_dir = exe_dir.join("logs/debug");
        println!("Logs directory: {}", logs_dir.display());
    }
    println!();

    loop {
        print!(" > ");
        stdout.flush()?;

        let mut line = String::new();
        let bytes_read = stdin.read_line(&mut line)?;

        if bytes_read == 0 {
            break;
        }

        let line = line.trim();
        
        if line.is_empty() {
            continue;
        }

        if line == "exit" {
            println!("Goodbye!");
            break;
        }

        match parse(line) {
            Some(parsed) => {
                if let Err(e) = execute_command(&store, &parsed) {
                    println!("Error: {}", e);
                }
            }
            None => continue,
        }
    }

    Ok(())
}

/// 执行命令
///
/// # 参数
/// * `store` - 记忆存储实例
/// * `parsed` - 解析后的命令
///
/// # 返回
/// IO 错误（如果有）
pub fn execute_command(store: &MemoryStore, parsed: &Parsed) -> io::Result<()> {
    match parsed.cmd.as_str() {
        "add" => {
            let text = parsed.args.join(" ");
            let tags = parsed.opts.get("tags")
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

            let rec = store.add_memory(&text, tags)?;
            println!("✅ Added {}", rec.id);
        }
        "search" => {
            let query = parsed.args.join(" ");
            let limit = parsed.opts.get("limit")
                .and_then(|l| l.parse().ok());

            let hits = store.search(&query, limit)?;
            if hits.is_empty() {
                println!("No results found");
            } else {
                for (i, hit) in hits.iter().enumerate() {
                    let tag_str = if hit.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", hit.tags.join(", "))
                    };
                    println!("{}. {}{} (score: {:.1})", i + 1, hit.text, tag_str, hit.score);
                }
            }
        }
        "stats" => {
            let stats = store.compute_stats()?;
            println!("Total: {}, Active: {}, Deleted: {}", stats.total, stats.active, stats.deleted);
            if !stats.tags.is_empty() {
                println!("\nTags:");
                for (tag, count) in stats.tags.iter().take(10) {
                    println!("  - {}: {}", tag, count);
                }
            }
        }
        "logs" => {
            println!("Logs command:");
            println!("  logs show - 显示最近的日志");
            println!("  logs clear - 清除所有日志");
            println!("  logs status - 显示日志状态");
        }
        "logs show" => {
            println!("Showing recent logs...");
            // 这里可以实现显示最近日志的逻辑
        }
        "logs clear" => {
            println!("Clearing all logs...");
            // 这里可以实现清除日志的逻辑
        }
        "logs status" => {
            println!("Logs status:");
            println!("  Logs directory: logs/debug");
            println!("  Max size: 1MB per file");
            println!("  Rotation: Enabled");
        }
        "whereiscfg" => {
            let config_path = crate::config::get_config_file_path(None);
            println!("Current config file path:");
            println!("{}", config_path);
        }
        "delete" => {
            if parsed.args.is_empty() {
                println!("Usage: delete <id>");
                return Ok(());
            }
            let id = &parsed.args[0];
            if store.soft_delete(id)? {
                println!("✅ Deleted {}", id);
            } else {
                println!("❌ Memory not found: {}", id);
            }
        }
        "purge" => {
            let id = parsed.opts.get("id").map(|s| s.as_str());
            let tag = parsed.opts.get("tag").map(|s| s.as_str());
            let match_text = parsed.opts.get("text").map(|s| s.as_str());
            
            let purged = store.purge(id, tag, match_text)?;
            println!("✅ Purged {} memories", purged);
        }
        "compress" => {
            let query = parsed.args.join(" ");
            let budget = parsed.opts.get("budget")
                .and_then(|b| b.parse().ok())
                .unwrap_or(2000);
            let limit = parsed.opts.get("limit")
                .and_then(|l| l.parse().ok());

            let records = store.load()?;
            let result = compress_deterministic(&records, &query, budget, limit);
            
            println!("--- Compressed Output ({} / {} chars) ---", result.used, result.budget);
            println!("{}", result.markdown);
            println!("--- End ---");
        }
        "export" => {
            let json = store.export_json()?;
            println!("{}", json);
        }
        "import" => {
            if parsed.args.is_empty() {
                println!("Usage: import <json_file>");
                return Ok(());
            }
            let file_path = &parsed.args[0];
            let json_data = std::fs::read_to_string(file_path)?;
            let (success, skipped, failed) = store.import_json(&json_data)?;
            println!("✅ Imported: {}, Skipped: {}, Failed: {}", success, skipped, failed);
        }
        "help" => {
            println!("Available commands:");
            println!("  add [--tags a,b,c] <text>      - Store a new memory");
            println!("  search <query> [--limit N]     - Search memories");
            println!("  delete <id>                    - Soft delete a memory");
            println!("  purge [--id ID] [--tag TAG] [--text TEXT] - Hard delete memories");
            println!("  compress <query> [--budget N] [--limit N] - Compress memories");
            println!("  stats                          - Show memory statistics");
            println!("  export                         - Export all memories as JSON");
            println!("  import <json_file>             - Import memories from JSON file");
            println!("  logs show                       - Show recent logs");
            println!("  logs clear                      - Clear all logs");
            println!("  logs status                     - Show logs status");
            println!("  whereiscfg                      - Show config file path");
            println!("  help                           - Show this help");
            println!("  exit                           - Quit CLI");
        }
        _ => {
            println!("Unknown command: {}. Type 'help' for available commands.", parsed.cmd);
        }
    }
    Ok(())
}
