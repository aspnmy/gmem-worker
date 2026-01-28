mod scanner;
mod cleaner;
mod rules;
mod config;
mod utils;
mod report;

use std::env;
use std::path::PathBuf;
use std::time::Instant;
use scanner::Scanner;
use cleaner::Cleaner;
use rules::{get_default_rules, get_exclude_paths};
use config::Config;
use report::ReportGenerator;
use utils::{format_file_size, ensure_directory_exists, get_current_timestamp};

/// 主函数
fn main() {
    let args: Vec<String> = env::args().collect();

    // 解析命令行参数
    let config = parse_args(&args);

    println!("C盘无用文件清理工具");
    println!("====================");
    println!();

    // 创建扫描器
    let rules = get_default_rules();
    let exclude_paths = get_exclude_paths();
    let scanner = Scanner::new(rules, exclude_paths);

    // 扫描文件
    println!("开始扫描文件...");
    let scan_start = Instant::now();

    let mut all_files = Vec::new();
    for scan_path in &config.scan_paths {
        println!("扫描路径: {}", scan_path);
        match scanner.scan_directory(scan_path) {
            Ok(files) => {
                println!("找到 {} 个无用文件", files.len());
                all_files.extend(files);
            }
            Err(e) => {
                eprintln!("扫描失败: {}", e);
            }
        }
    }

    let scan_duration = scan_start.elapsed();
    println!("扫描完成，耗时: {:?}", scan_duration);
    println!("总共找到 {} 个无用文件", all_files.len());
    println!();

    // 计算总大小
    let total_size: u64 = all_files.iter().map(|f| f.size).sum();
    println!("总大小: {}", format_file_size(total_size));
    println!();

    // 询问是否清理
    if config.dry_run {
        println!("预览模式，不会实际删除文件");
        println!("使用 --clean 参数执行实际清理");
        println!();

        // 生成扫描报告
        if let Err(e) = generate_scan_report(&all_files) {
            eprintln!("生成扫描报告失败: {}", e);
        }
    } else {
        println!("警告：即将删除 {} 个文件，释放 {} 空间",
            all_files.len(),
            format_file_size(total_size)
        );
        println!("是否继续? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "y" {
            println!("取消清理");
            return;
        }

        // 清理文件
        println!("开始清理文件...");
        let clean_start = Instant::now();

        let cleaner = Cleaner::new(config.dry_run, config.verbose);
        match cleaner.clean_files(&all_files) {
            Ok(result) => {
                let clean_duration = clean_start.elapsed();
                println!("清理完成，耗时: {:?}", clean_duration);
                println!("清理文件数: {}", result.cleaned_files.len());
                println!("释放空间: {}", format_file_size(result.total_size));

                if !result.failed_files.is_empty() {
                    println!("失败文件数: {}", result.failed_files.len());
                }

                // 生成清理报告
                if let Err(e) = generate_clean_report(&result) {
                    eprintln!("生成清理报告失败: {}", e);
                } else {
                    println!("报告已生成");
                }
            }
            Err(e) => {
                eprintln!("清理失败: {}", e);
            }
        }
    }
}

/// 解析命令行参数
///
/// 参数:
///   - args: 命令行参数列表
///
/// 返回值:
///   - Config: 配置对象
fn parse_args(args: &[String]) -> Config {
    let mut config = Config::default();

    // 获取可执行文件所在目录
    let exe_path = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // 构建配置文件路径：可执行文件同级目录下的./config/default_config.toml
    let config_dir = exe_path.join("config");
    let config_file_path = config_dir.join("default_config.toml");
    let config_file_str = config_file_path.to_string_lossy().to_string();

    // 确保配置文件存在，如果不存在则创建默认配置
    if let Err(e) = Config::ensure_config_file(&config_file_str) {
        eprintln!("配置文件检查失败: {}", e);
    }

    // 尝试从配置文件加载默认配置
    if let Ok(loaded_config) = Config::load_from_file(&config_file_str) {
        config.scan_paths = loaded_config.scan_paths;
        config.exclude_paths = loaded_config.exclude_paths;
        config.max_age_days = loaded_config.max_age_days;
        config.min_file_size = loaded_config.min_file_size;
        config.dry_run = loaded_config.dry_run;
        config.verbose = loaded_config.verbose;
    }

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--clean" => {
                config.dry_run = false;
            }
            "--quiet" => {
                config.verbose = false;
            }
            "--scan" => {
                if i + 1 < args.len() {
                    config.scan_paths.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--exclude" => {
                if i + 1 < args.len() {
                    config.exclude_paths.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--max-age" => {
                if i + 1 < args.len() {
                    if let Ok(days) = args[i + 1].parse::<u64>() {
                        config.max_age_days = days;
                    }
                    i += 1;
                }
            }
            "--config" => {
                if i + 1 < args.len() {
                    if let Ok(loaded_config) = Config::load_from_file(&args[i + 1]) {
                        config.scan_paths = loaded_config.scan_paths;
                        config.exclude_paths = loaded_config.exclude_paths;
                        config.max_age_days = loaded_config.max_age_days;
                        config.min_file_size = loaded_config.min_file_size;
                        config.dry_run = loaded_config.dry_run;
                        config.verbose = loaded_config.verbose;
                    }
                    i += 1;
                }
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    config
}

/// 打印帮助信息
fn print_help() {
    println!("C盘无用文件清理工具");
    println!();
    println!("用法:");
    println!("  disk_cleaner [选项]");
    println!();
    println!("选项:");
    println!("  --clean          执行实际清理（默认为预览模式）");
    println!("  --quiet          安静模式，不输出详细信息");
    println!("  --scan <路径>    添加扫描路径");
    println!("  --exclude <路径>  添加排除路径");
    println!("  --max-age <天数>  设置文件最大年龄（天）");
    println!("  --config <文件>   指定配置文件（默认：config/default_config.toml）");
    println!("  --help           显示帮助信息");
    println!();
    println!("示例:");
    println!("  disk_cleaner                    # 预览模式，使用默认配置");
    println!("  disk_cleaner --clean           # 执行实际清理");
    println!("  disk_cleaner --scan \"C:\\Temp\" --clean  # 扫描指定路径并清理");
    println!("  disk_cleaner --config custom.toml --clean  # 使用自定义配置文件");
}

/// 生成扫描报告
///
/// 参数:
///   - files: 扫描到的文件列表
///
/// 返回值:
///   - Ok(()): 报告生成成功
///   - Err(String): 错误信息
fn generate_scan_report(files: &[scanner::FileInfo]) -> Result<(), String> {
    // 确保reports目录存在
    ensure_directory_exists("reports")?;

    let report_path = format!("reports/scan_report_{}.txt",
        get_current_timestamp()
    );
    let report_generator = ReportGenerator::new(report_path);
    report_generator.generate_scan_report(files)
}

/// 生成清理报告
///
/// 参数:
///   - result: 清理结果
///
/// 返回值:
///   - Ok(()): 报告生成成功
///   - Err(String): 错误信息
fn generate_clean_report(result: &cleaner::CleanResult) -> Result<(), String> {
    // 确保reports目录存在
    ensure_directory_exists("reports")?;

    let report_path = format!("reports/clean_report_{}.txt",
        get_current_timestamp()
    );
    let report_generator = ReportGenerator::new(report_path);
    report_generator.generate_report(result)
}
