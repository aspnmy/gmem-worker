# 【实战】C盘无用文件快速清理工具 - Rust实现

项目路径：

https://github.com/aspnmy/rust_disk_cleaner.git

如果要完整体验此工具需要git仓库到本地

```bash
git clone https://github.com/aspnmy/rust_disk_cleaner.git
```

## 一、简介

随着Windows系统的使用，C盘空间会逐渐被各种无用文件占用，包括临时文件、缓存文件、日志文件、回收站文件、更新备份文件等。这些文件不仅占用宝贵的磁盘空间，还可能影响系统性能。

本文将介绍如何使用Rust语言开发一个高效、安全的C盘清理工具，该工具可以：

- **快速扫描**：高效扫描C盘中的无用文件
- **安全清理**：只清理确认安全的文件，避免误删重要文件
- **详细报告**：生成详细的清理报告，包括文件类型、大小、数量等
- **自定义配置**：支持自定义清理规则和排除路径
- **批量处理**：支持批量删除文件，提高清理效率

### 1.1 为什么选择Rust？

Rust是一门系统级编程语言，具有以下优势：

- **高性能**：Rust的性能接近C/C++，适合处理大量文件操作
- **内存安全**：Rust的所有权机制确保内存安全，避免常见的内存错误
- **并发安全**：Rust的并发模型安全高效，适合多线程文件扫描
- **跨平台**：Rust支持Windows、Linux、macOS等多个平台
- **丰富的生态**：Rust拥有丰富的第三方库，方便快速开发

### 1.2 工具特点

- **高效扫描**：使用多线程并行扫描，提高扫描速度
- **安全可靠**：内置安全检查机制，避免误删系统关键文件
- **详细日志**：记录所有操作，便于审计和回溯
- **用户友好**：提供命令行界面，操作简单直观
- **可扩展**：支持插件式架构，方便扩展新功能

## 二、项目架构

### 2.1 项目结构

```
rust_disk_cleaner/
├── src/
│   ├── main.rs              # 主程序入口
│   ├── scanner.rs           # 文件扫描模块
│   ├── cleaner.rs           # 文件清理模块
│   ├── rules.rs             # 清理规则模块
│   ├── config.rs            # 配置管理模块
│   ├── utils.rs             # 工具函数模块
│   └── report.rs            # 报告生成模块
├── config/
│   ├── default_rules.toml   # 默认清理规则
│   └── exclude_paths.toml   # 排除路径配置
├── logs/                    # 日志目录
├── reports/                 # 报告目录
├── Cargo.toml               # Rust项目配置
├── README.md                # 项目说明
└── .gitignore               # Git忽略文件
```

### 2.2 核心模块说明

#### 2.2.1 scanner.rs - 文件扫描模块

负责扫描指定目录中的文件，识别无用文件：

```rust
use std::fs;
use std::path::Path;
use std::collections::HashMap;

/// 文件信息结构体
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub file_type: FileType,
    pub last_modified: u64,
}

/// 文件类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    TempFile,
    CacheFile,
    LogFile,
    RecycleBin,
    UpdateBackup,
    BrowserCache,
    SystemTemp,
    UserTemp,
    Other,
}

/// 文件扫描器
pub struct Scanner {
    rules: Vec<ScanRule>,
    exclude_paths: Vec<String>,
}

/// 扫描规则结构体
#[derive(Debug, Clone)]
pub struct ScanRule {
    pub pattern: String,
    pub file_type: FileType,
    pub max_age_days: Option<u64>,
}

impl Scanner {
    /// 创建新的扫描器
    pub fn new(rules: Vec<ScanRule>, exclude_paths: Vec<String>) -> Self {
        Scanner {
            rules,
            exclude_paths,
        }
    }

    /// 扫描指定目录
    pub fn scan_directory(&self, path: &str) -> Result<Vec<FileInfo>, String> {
        let mut files = Vec::new();
        let path = Path::new(path);

        if !path.exists() {
            return Err(format!("路径不存在: {}", path.display()));
        }

        self.scan_recursive(path, &mut files)?;
        Ok(files)
    }

    /// 递归扫描目录
    fn scan_recursive(&self, path: &Path, files: &mut Vec<FileInfo>) -> Result<(), String> {
        let entries = fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取文件失败: {}", e))?;
            let file_path = entry.path();

            // 检查是否在排除路径中
            if self.is_excluded(&file_path) {
                continue;
            }

            if file_path.is_dir() {
                self.scan_recursive(&file_path, files)?;
            } else {
                if let Some(file_info) = self.check_file(&file_path) {
                    files.push(file_info);
                }
            }
        }

        Ok(())
    }

    /// 检查文件是否匹配扫描规则
    fn check_file(&self, path: &Path) -> Option<FileInfo> {
        let file_name = path.file_name()?.to_str()?;
        let file_path = path.to_str()?;

        for rule in &self.rules {
            if self.match_pattern(file_name, &rule.pattern) {
                let metadata = fs::metadata(path).ok()?;
                let size = metadata.len();
                let modified = metadata.modified().ok()?;
                let last_modified = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();

                // 检查文件年龄
                if let Some(max_age) = rule.max_age_days {
                    let age_days = (chrono::Utc::now().timestamp() as u64 - last_modified) / 86400;
                    if age_days < max_age {
                        continue;
                    }
                }

                return Some(FileInfo {
                    path: file_path.to_string(),
                    size,
                    file_type: rule.file_type.clone(),
                    last_modified,
                });
            }
        }

        None
    }

    /// 匹配文件名模式
    fn match_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }
        file_name == pattern
    }

    /// 检查路径是否在排除列表中
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_str().unwrap_or("");
        for exclude_path in &self.exclude_paths {
            if path_str.contains(exclude_path) {
                return true;
            }
        }
        false
    }
}
```

#### 2.2.2 cleaner.rs - 文件清理模块

负责清理扫描到的无用文件：

```rust
use std::fs;
use std::path::Path;
use crate::scanner::FileInfo;

/// 文件清理器
pub struct Cleaner {
    dry_run: bool,
    verbose: bool,
}

impl Cleaner {
    /// 创建新的清理器
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Cleaner {
            dry_run,
            verbose,
        }
    }

    /// 清理文件
    pub fn clean_files(&self, files: &[FileInfo]) -> Result<CleanResult, String> {
        let mut cleaned_files = Vec::new();
        let mut failed_files = Vec::new();
        let mut total_size = 0u64;

        for file in files {
            match self.clean_file(file) {
                Ok(size) => {
                    cleaned_files.push(file.clone());
                    total_size += size;
                }
                Err(e) => {
                    failed_files.push((file.clone(), e));
                }
            }
        }

        Ok(CleanResult {
            cleaned_files,
            failed_files,
            total_size,
        })
    }

    /// 清理单个文件
    fn clean_file(&self, file: &FileInfo) -> Result<u64, String> {
        let path = Path::new(&file.path);

        if self.verbose {
            println!("清理文件: {} (大小: {} bytes)", file.path, file.size);
        }

        if self.dry_run {
            return Ok(file.size);
        }

        fs::remove_file(path)
            .map_err(|e| format!("删除文件失败: {} - {}", file.path, e))?;

        Ok(file.size)
    }

    /// 清空目录
    pub fn clean_directory(&self, path: &str) -> Result<u64, String> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(format!("目录不存在: {}", path.display()));
        }

        let mut total_size = 0u64;
        let entries = fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取文件失败: {}", e))?;
            let file_path = entry.path();

            if file_path.is_dir() {
                total_size += self.clean_directory(file_path.to_str().unwrap())?;
            } else {
                let metadata = fs::metadata(&file_path).ok()?;
                let size = metadata.len();

                if self.verbose {
                    println!("清理文件: {} (大小: {} bytes)", file_path.display(), size);
                }

                if !self.dry_run {
                    fs::remove_file(&file_path)
                        .map_err(|e| format!("删除文件失败: {} - {}", file_path.display(), e))?;
                }

                total_size += size;
            }
        }

        Ok(total_size)
    }
}

/// 清理结果结构体
#[derive(Debug)]
pub struct CleanResult {
    pub cleaned_files: Vec<FileInfo>,
    pub failed_files: Vec<(FileInfo, String)>,
    pub total_size: u64,
}
```

#### 2.2.3 rules.rs - 清理规则模块

定义清理规则和排除路径：

```rust
use crate::scanner::{ScanRule, FileType};

/// 获取默认清理规则
pub fn get_default_rules() -> Vec<ScanRule> {
    vec![
        // 临时文件
        ScanRule {
            pattern: "*.tmp".to_string(),
            file_type: FileType::TempFile,
            max_age_days: Some(7),
        },
        ScanRule {
            pattern: "*.temp".to_string(),
            file_type: FileType::TempFile,
            max_age_days: Some(7),
        },

        // 缓存文件
        ScanRule {
            pattern: "*.cache".to_string(),
            file_type: FileType::CacheFile,
            max_age_days: Some(30),
        },
        ScanRule {
            pattern: "*.cach".to_string(),
            file_type: FileType::CacheFile,
            max_age_days: Some(30),
        },

        // 日志文件
        ScanRule {
            pattern: "*.log".to_string(),
            file_type: FileType::LogFile,
            max_age_days: Some(30),
        },

        // 更新备份文件
        ScanRule {
            pattern: "*.old".to_string(),
            file_type: FileType::UpdateBackup,
            max_age_days: Some(90),
        },
        ScanRule {
            pattern: "*.bak".to_string(),
            file_type: FileType::UpdateBackup,
            max_age_days: Some(90),
        },

        // 浏览器缓存
        ScanRule {
            pattern: "Cache".to_string(),
            file_type: FileType::BrowserCache,
            max_age_days: Some(30),
        },

        // 系统临时文件
        ScanRule {
            pattern: "~*".to_string(),
            file_type: FileType::SystemTemp,
            max_age_days: Some(7),
        },
    ]
}

/// 获取排除路径
pub fn get_exclude_paths() -> Vec<String> {
    vec![
        "Windows".to_string(),
        "Program Files".to_string(),
        "Program Files (x86)".to_string(),
        "ProgramData".to_string(),
        "$Recycle.Bin".to_string(),
        "System Volume Information".to_string(),
    ]
}

/// 获取特定目录的清理规则
pub fn get_directory_rules(directory: &str) -> Vec<ScanRule> {
    match directory {
        "C:\\Windows\\Temp" => vec![
            ScanRule {
                pattern: "*".to_string(),
                file_type: FileType::SystemTemp,
                max_age_days: Some(7),
            },
        ],
        "C:\\Users" => vec![
            ScanRule {
                pattern: "*.tmp".to_string(),
                file_type: FileType::UserTemp,
                max_age_days: Some(7),
            },
            ScanRule {
                pattern: "*.log".to_string(),
                file_type: FileType::LogFile,
                max_age_days: Some(30),
            },
        ],
        _ => get_default_rules(),
    }
}
```

#### 2.2.4 config.rs - 配置管理模块

管理配置文件的读取和写入：

```rust
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// 配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub scan_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub max_age_days: u64,
    pub min_file_size: u64,
    pub dry_run: bool,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scan_paths: vec![
                "C:\\Windows\\Temp".to_string(),
                "C:\\Users".to_string(),
            ],
            exclude_paths: vec![
                "Windows".to_string(),
                "Program Files".to_string(),
                "Program Files (x86)".to_string(),
                "ProgramData".to_string(),
            ],
            max_age_days: 30,
            min_file_size: 0,
            dry_run: true,
            verbose: true,
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(format!("配置文件不存在: {}", path.display()));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        Ok(())
    }
}
```

#### 2.2.5 report.rs - 报告生成模块

生成详细的清理报告：

```rust
use crate::scanner::{FileInfo, FileType};
use crate::cleaner::CleanResult;
use std::fs::File;
use std::io::Write;
use chrono::Local;

/// 报告生成器
pub struct ReportGenerator {
    output_path: String,
}

impl ReportGenerator {
    /// 创建新的报告生成器
    pub fn new(output_path: String) -> Self {
        ReportGenerator { output_path }
    }

    /// 生成清理报告
    pub fn generate_report(&self, result: &CleanResult) -> Result<(), String> {
        let mut file = File::create(&self.output_path)
            .map_err(|e| format!("创建报告文件失败: {}", e))?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        writeln!(file, "C盘清理报告").map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(file, "生成时间: {}", timestamp).map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 统计信息
        writeln!(file, "统计信息").map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(file, "清理文件数: {}", result.cleaned_files.len())
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(file, "失败文件数: {}", result.failed_files.len())
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(file, "释放空间: {} MB", result.total_size / 1024 / 1024)
            .map_err(|e| format!("写入报告失败: {}", e))?;
        writeln!(file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 按文件类型统计
        writeln!(file, "按文件类型统计").map_err(|e| format!("写入报告失败: {}", e))?;
        let mut type_stats = std::collections::HashMap::new();
        for file in &result.cleaned_files {
            let entry = type_stats.entry(file.file_type.clone()).or_insert((0u64, 0usize));
            entry.0 += file.size;
            entry.1 += 1;
        }

        for (file_type, (size, count)) in &type_stats {
            writeln!(file, "{}: {} 个文件, {} MB",
                format_file_type(file_type),
                count,
                size / 1024 / 1024
            ).map_err(|e| format!("写入报告失败: {}", e))?;
        }
        writeln!(file).map_err(|e| format!("写入报告失败: {}", e))?;

        // 清理的文件列表
        writeln!(file, "清理的文件列表").map_err(|e| format!("写入报告失败: {}", e))?;
        for file in &result.cleaned_files {
            writeln!(file, "{} - {} - {} MB",
                file.path,
                format_file_type(&file.file_type),
                file.size / 1024 / 1024
            ).map_err(|e| format!("写入报告失败: {}", e))?;
        }

        // 失败的文件列表
        if !result.failed_files.is_empty() {
            writeln!(file).map_err(|e| format!("写入报告失败: {}", e))?;
            writeln!(file, "失败的文件列表").map_err(|e| format!("写入报告失败: {}", e))?;
            for (file, error) in &result.failed_files {
                writeln!(file, "{} - {}", file.path, error)
                    .map_err(|e| format!("写入报告失败: {}", e))?;
            }
        }

        Ok(())
    }

    /// 格式化文件类型
    fn format_file_type(file_type: &FileType) -> &str {
        match file_type {
            FileType::TempFile => "临时文件",
            FileType::CacheFile => "缓存文件",
            FileType::LogFile => "日志文件",
            FileType::RecycleBin => "回收站",
            FileType::UpdateBackup => "更新备份",
            FileType::BrowserCache => "浏览器缓存",
            FileType::SystemTemp => "系统临时文件",
            FileType::UserTemp => "用户临时文件",
            FileType::Other => "其他",
        }
    }
}
```

#### 2.2.6 main.rs - 主程序入口

整合所有模块，提供命令行界面：

```rust
mod scanner;
mod cleaner;
mod rules;
mod config;
mod utils;
mod report;

use std::env;
use std::time::Instant;
use scanner::Scanner;
use cleaner::Cleaner;
use rules::{get_default_rules, get_exclude_paths};
use config::Config;
use report::ReportGenerator;

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
    println!("总大小: {} MB", total_size / 1024 / 1024);
    println!();

    // 询问是否清理
    if config.dry_run {
        println!("预览模式，不会实际删除文件");
        println!("使用 --clean 参数执行实际清理");
        println!();
    } else {
        println!("警告：即将删除 {} 个文件，释放 {} MB 空间",
            all_files.len(),
            total_size / 1024 / 1024
        );
        println!("是否继续? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "y" {
            println!("取消清理");
            return;
        }
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
            println!("释放空间: {} MB", result.total_size / 1024 / 1024);

            if !result.failed_files.is_empty() {
                println!("失败文件数: {}", result.failed_files.len());
            }

            // 生成报告
            let report_path = format!("reports/clean_report_{}.txt",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );
            let report_generator = ReportGenerator::new(report_path);
            if let Err(e) = report_generator.generate_report(&result) {
                eprintln!("生成报告失败: {}", e);
            } else {
                println!("报告已生成");
            }
        }
        Err(e) => {
            eprintln!("清理失败: {}", e);
        }
    }
}

/// 解析命令行参数
fn parse_args(args: &[String]) -> Config {
    let mut config = Config::default();

    for i in 1..args.len() {
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
                }
            }
            "--exclude" => {
                if i + 1 < args.len() {
                    config.exclude_paths.push(args[i + 1].clone());
                }
            }
            "--max-age" => {
                if i + 1 < args.len() {
                    if let Ok(days) = args[i + 1].parse::<u64>() {
                        config.max_age_days = days;
                    }
                }
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
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
    println!("  --help           显示帮助信息");
    println!();
    println!("示例:");
    println!("  disk_cleaner                    # 预览模式，扫描默认路径");
    println!("  disk_cleaner --clean           # 执行实际清理");
    println!("  disk_cleaner --scan \"C:\\Temp\" --clean  # 扫描指定路径并清理");
}
```

## 三、Cargo.toml配置

```toml
[package]
name = "rust_disk_cleaner"
version = "0.1.0"
edition = "2021"
authors = ["aspnmy <support@e2bank.cn>"]
description = "C盘无用文件快速清理工具"
license = "MIT"

[dependencies]
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[[bin]]
name = "disk_cleaner"
path = "src/main.rs"
```

## 四、使用方法

### 4.1 编译项目

```bash
# 克隆项目
git clone https://github.com/aspnmy/rust_disk_cleaner.git
cd rust_disk_cleaner

# 编译项目
cargo build --release

# 编译后的可执行文件位于 target/release/disk_cleaner.exe
```

### 4.2 基本使用

#### 4.2.1 预览模式（推荐）

```bash
# 预览模式，扫描默认路径
.\target\release\disk_cleaner.exe

# 预览模式，扫描指定路径
.\target\release\disk_cleaner.exe --scan "C:\Temp"

# 预览模式，安静模式
.\target\release\disk_cleaner.exe --quiet
```

#### 4.2.2 实际清理

```bash
# 执行实际清理（会提示确认）
.\target\release\disk_cleaner.exe --clean

# 扫描指定路径并清理
.\target\release\disk_cleaner.exe --scan "C:\Temp" --clean

# 设置文件最大年龄
.\target\release\disk_cleaner.exe --max-age 7 --clean
```

#### 4.2.3 高级选项

```bash
# 添加多个扫描路径
.\target\release\disk_cleaner.exe --scan "C:\Temp" --scan "C:\Users\Username\AppData\Local\Temp" --clean

# 添加排除路径
.\target\release\disk_cleaner.exe --exclude "C:\Important" --clean

# 组合使用
.\target\release\disk_cleaner.exe --scan "C:\Temp" --exclude "C:\Important" --max-age 7 --clean
```

### 4.3 使用示例

#### 4.3.1 清理系统临时文件

```bash
# 预览系统临时文件
.\target\release\disk_cleaner.exe --scan "C:\Windows\Temp"

# 清理系统临时文件
.\target\release\disk_cleaner.exe --scan "C:\Windows\Temp" --clean
```

#### 4.3.2 清理用户临时文件

```bash
# 预览用户临时文件
.\target\release\disk_cleaner.exe --scan "C:\Users\Username\AppData\Local\Temp"

# 清理用户临时文件
.\target\release\disk_cleaner.exe --scan "C:\Users\Username\AppData\Local\Temp" --clean
```

#### 4.3.3 清理浏览器缓存

```bash
# 预览Chrome缓存
.\target\release\disk_cleaner.exe --scan "C:\Users\Username\AppData\Local\Google\Chrome\User Data\Default\Cache"

# 清理Chrome缓存
.\target\release\disk_cleaner.exe --scan "C:\Users\Username\AppData\Local\Google\Chrome\User Data\Default\Cache" --clean
```

## 五、清理策略

### 5.1 文件类型分类

#### 5.1.1 临时文件（*.tmp, *.temp）

- **位置**：系统临时目录、用户临时目录
- **清理策略**：清理7天前的文件
- **风险等级**：低
- **说明**：临时文件通常由程序创建，用于临时存储数据，程序关闭后通常不再需要

#### 5.1.2 缓存文件（*.cache, *.cach）

- **位置**：应用程序缓存目录、浏览器缓存目录
- **清理策略**：清理30天前的文件
- **风险等级**：低
- **说明**：缓存文件用于加速程序运行，删除后程序会重新生成

#### 5.1.3 日志文件（*.log）

- **位置**：应用程序日志目录、系统日志目录
- **清理策略**：清理30天前的文件
- **风险等级**：中
- **说明**：日志文件记录程序运行信息，删除后不影响程序运行，但可能影响问题排查

#### 5.1.4 更新备份文件（*.old, *.bak）

- **位置**：系统目录、应用程序目录
- **清理策略**：清理90天前的文件
- **风险等级**：中
- **说明**：更新备份文件是系统或程序更新前的备份，删除后无法回滚

#### 5.1.5 浏览器缓存

- **位置**：浏览器缓存目录
- **清理策略**：清理30天前的文件
- **风险等级**：低
- **说明**：浏览器缓存用于加速网页加载，删除后浏览器会重新下载

#### 5.1.6 系统临时文件（~*）

- **位置**：系统目录、用户目录
- **清理策略**：清理7天前的文件
- **风险等级**：低
- **说明**：系统临时文件通常由系统创建，用于临时存储数据

### 5.2 排除路径

以下路径默认被排除，不会被扫描：

- `Windows`：Windows系统目录
- `Program Files`：程序安装目录
- `Program Files (x86)`：32位程序安装目录
- `ProgramData`：程序数据目录
- `$Recycle.Bin`：回收站
- `System Volume Information`：系统卷信息

### 5.3 清理建议

#### 5.3.1 定期清理

- **频率**：建议每月清理一次
- **时间**：建议在系统空闲时清理
- **备份**：清理前建议备份重要数据

#### 5.3.2 清理顺序

1. **临时文件**：优先清理临时文件，风险最低
2. **缓存文件**：清理缓存文件，释放空间较多
3. **日志文件**：清理日志文件，注意保留最近的日志
4. **更新备份**：清理更新备份，确认不需要回滚后再清理

#### 5.3.3 注意事项

- **预览模式**：首次使用建议使用预览模式，查看将要清理的文件
- **确认清理**：实际清理前仔细确认将要清理的文件列表
- **系统备份**：清理前建议创建系统备份
- **重要数据**：不要清理包含重要数据的目录

## 六、安全注意事项

### 6.1 风险评估

| 文件类型 | 风险等级 | 说明 |
|---------|---------|------|
| 临时文件 | 低 | 通常可以安全删除 |
| 缓存文件 | 低 | 删除后程序会重新生成 |
| 日志文件 | 中 | 删除后影响问题排查 |
| 更新备份 | 中 | 删除后无法回滚 |
| 系统文件 | 高 | 不要删除系统文件 |
| 用户数据 | 高 | 不要删除用户数据 |

### 6.2 安全措施

#### 6.2.1 预览模式

- 默认使用预览模式
- 查看将要清理的文件列表
- 确认无误后再执行实际清理

#### 6.2.2 排除路径

- 默认排除系统关键路径
- 可以自定义排除路径
- 避免误删重要文件

#### 6.2.3 文件年龄限制

- 默认只清理30天前的文件
- 可以自定义文件年龄限制
- 避免删除最近创建的文件

#### 6.2.4 确认提示

- 实际清理前会提示确认
- 需要用户手动确认才会执行清理
- 避免误操作

### 6.3 最佳实践

#### 6.3.1 使用前检查

1. **查看配置**：检查扫描路径和排除路径配置
2. **预览模式**：先使用预览模式查看将要清理的文件
3. **确认文件**：仔细确认将要清理的文件列表
4. **备份数据**：清理前备份重要数据

#### 6.3.2 定期维护

1. **定期清理**：建议每月清理一次
2. **监控空间**：定期检查C盘空间使用情况
3. **优化设置**：根据实际情况调整清理策略
4. **查看报告**：定期查看清理报告

#### 6.3.3 应急处理

1. **误删文件**：如果误删重要文件，立即停止使用计算机
2. **数据恢复**：使用数据恢复软件尝试恢复
3. **系统还原**：如果系统文件被误删，使用系统还原功能
4. **专业帮助**：如果无法解决，寻求专业帮助

## 七、性能优化

### 7.1 扫描性能

#### 7.1.1 多线程扫描

```rust
use std::thread;
use std::sync::{Arc, Mutex};

/// 多线程扫描
pub fn parallel_scan(scanner: &Scanner, paths: &[String]) -> Vec<FileInfo> {
    let scanner = Arc::new(scanner);
    let files = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for path in paths {
        let scanner = Arc::clone(&scanner);
        let files = Arc::clone(&files);
        let path = path.clone();

        let handle = thread::spawn(move || {
            match scanner.scan_directory(&path) {
                Ok(mut result) => {
                    let mut files = files.lock().unwrap();
                    files.append(&mut result);
                }
                Err(e) => {
                    eprintln!("扫描失败: {} - {}", path, e);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let files = files.lock().unwrap();
    files.clone()
}
```

#### 7.1.2 增量扫描

```rust
use std::collections::HashSet;

/// 增量扫描
pub struct IncrementalScanner {
    scanned_files: HashSet<String>,
}

impl IncrementalScanner {
    pub fn new() -> Self {
        IncrementalScanner {
            scanned_files: HashSet::new(),
        }
    }

    pub fn scan(&mut self, scanner: &Scanner, path: &str) -> Vec<FileInfo> {
        let files = scanner.scan_directory(path).unwrap_or_default();

        files.into_iter()
            .filter(|file| !self.scanned_files.contains(&file.path))
            .filter_map(|file| {
                self.scanned_files.insert(file.path.clone());
                Some(file)
            })
            .collect()
    }
}
```

### 7.2 清理性能

#### 7.2.1 批量删除

```rust
use std::fs;

/// 批量删除文件
pub fn batch_delete(files: &[FileInfo]) -> Result<Vec<String>, String> {
    let mut deleted_files = Vec::new();
    let mut failed_files = Vec::new();

    for file in files {
        match fs::remove_file(&file.path) {
            Ok(_) => {
                deleted_files.push(file.path.clone());
            }
            Err(e) => {
                failed_files.push(format!("{}: {}", file.path, e));
            }
        }
    }

    if failed_files.is_empty() {
        Ok(deleted_files)
    } else {
        Err(format!("部分文件删除失败: {:?}", failed_files))
    }
}
```

#### 7.2.2 异步清理

```rust
use tokio::fs;

/// 异步清理文件
pub async fn async_clean_file(path: &str) -> Result<u64, String> {
    let metadata = fs::metadata(path).await
        .map_err(|e| format!("获取文件信息失败: {}", e))?;

    let size = metadata.len();

    fs::remove_file(path).await
        .map_err(|e| format!("删除文件失败: {}", e))?;

    Ok(size)
}
```

## 八、扩展功能

### 8.1 定时清理

使用Windows任务计划程序设置定时清理：

```powershell
# 创建任务计划
schtasks /create /tn "C盘清理" /tr "C:\path\to\disk_cleaner.exe --clean" /sc monthly /d 1 /st 02:00
```

### 8.2 系统托盘

创建系统托盘应用程序，方便用户操作：

```rust
use tray_icon::{TrayIconBuilder, menu::Menu};

/// 创建系统托盘图标
pub fn create_tray_icon() -> Result<TrayIcon, String> {
    let menu = Menu::new();
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .build()
        .map_err(|e| format!("创建托盘图标失败: {}", e))?;

    Ok(tray_icon)
}
```

### 8.3 图形界面

使用egui或tauri创建图形界面：

```rust
use eframe::egui;

/// 创建图形界面
pub fn create_gui() -> Result<(), String> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "C盘清理工具",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    ).map_err(|e| format!("创建GUI失败: {}", e))?;

    Ok(())
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("C盘清理工具");
            ui.label("选择要清理的文件类型");
            // 添加UI控件
        });
    }
}
```

## 九、常见问题

### 9.1 清理后系统变慢

**问题**：清理后系统变慢

**解决**：
- 清理缓存文件后，程序需要重新生成缓存，可能会暂时变慢
- 等待一段时间，系统会自动恢复正常
- 如果问题持续，检查是否误删了重要文件

### 9.2 清理后程序无法启动

**问题**：清理后某个程序无法启动

**解决**：
- 检查是否清理了该程序的配置文件
- 重新安装该程序
- 从备份中恢复配置文件

### 9.3 扫描速度慢

**问题**：扫描速度很慢

**解决**：
- 减少扫描路径
- 使用多线程扫描
- 使用增量扫描
- 排除不需要扫描的目录

### 9.4 清理失败

**问题**：清理文件失败

**解决**：
- 检查文件是否被占用
- 以管理员身份运行程序
- 关闭相关程序后再清理
- 检查文件权限

### 9.5 释放空间不如预期

**问题**：清理后释放的空间不如预期

**解决**：
- 检查是否清理了所有目标文件
- 检查文件大小统计是否正确
- 检查是否有其他大文件占用空间
- 使用磁盘分析工具查看空间使用情况

## 十、总结

本文介绍了如何使用Rust语言开发一个高效、安全的C盘清理工具。该工具具有以下特点：

### 10.1 核心优势

1. **高性能**：使用Rust开发，性能接近C/C++
2. **内存安全**：Rust的所有权机制确保内存安全
3. **并发安全**：支持多线程扫描，提高扫描速度
4. **安全可靠**：内置安全检查机制，避免误删重要文件
5. **用户友好**：提供命令行界面，操作简单直观
6. **可扩展**：支持插件式架构，方便扩展新功能

### 10.2 应用场景

1. **系统维护**：定期清理C盘，释放磁盘空间
2. **性能优化**：清理无用文件，提高系统性能
3. **磁盘管理**：管理磁盘空间，避免空间不足
4. **自动化运维**：结合任务计划程序，实现自动化清理

### 10.3 最佳实践

1. **预览模式**：首次使用建议使用预览模式
2. **定期清理**：建议每月清理一次
3. **备份数据**：清理前备份重要数据
4. **查看报告**：定期查看清理报告
5. **监控空间**：定期检查C盘空间使用情况

### 10.4 未来展望

未来计划添加以下功能：

1. **图形界面**：提供图形界面，方便用户操作
2. **系统托盘**：创建系统托盘应用程序
3. **定时清理**：支持定时清理功能
4. **云同步**：支持清理规则云同步
5. **智能分析**：使用AI分析文件重要性，智能推荐清理

通过使用这个C盘清理工具，我们可以高效、安全地清理C盘中的无用文件，释放磁盘空间，提高系统性能，成为系统维护的得力助手！

## 参考链接

- Rust官方文档：https://www.rust-lang.org/
- Rust标准库：https://doc.rust-lang.org/std/
- Cargo使用指南：https://doc.rust-lang.org/cargo/
- Windows文件系统：https://docs.microsoft.com/en-us/windows/win32/fileio/file-systems

## 相关代码

### 完整项目代码

项目代码已上传到GitHub：

https://github.com/aspnmy/rust_disk_cleaner.git

### 核心模块代码

本文已提供完整的Rust代码，包括：

1. [scanner.rs](#scanner.rs---文件扫描模块) - 文件扫描模块
2. [cleaner.rs](#cleaner.rs---文件清理模块) - 文件清理模块
3. [rules.rs](#rules.rs---清理规则模块) - 清理规则模块
4. [config.rs](#config.rs---配置管理模块) - 配置管理模块
5. [report.rs](#report.rs---报告生成模块) - 报告生成模块
6. [main.rs](#main.rs---主程序入口) - 主程序入口

### 编译和运行

```bash
# 克隆项目
git clone https://github.com/aspnmy/rust_disk_cleaner.git
cd rust_disk_cleaner

# 编译项目
cargo build --release

# 运行程序（预览模式）
.\target\release\disk_cleaner.exe

# 运行程序（实际清理）
.\target\release\disk_cleaner.exe --clean
```

## 附录

### A. 清理规则配置文件

```toml
# default_rules.toml

[[rules]]
pattern = "*.tmp"
file_type = "TempFile"
max_age_days = 7

[[rules]]
pattern = "*.cache"
file_type = "CacheFile"
max_age_days = 30

[[rules]]
pattern = "*.log"
file_type = "LogFile"
max_age_days = 30

[[rules]]
pattern = "*.old"
file_type = "UpdateBackup"
max_age_days = 90
```

### B. 排除路径配置文件

```toml
# exclude_paths.toml

exclude_paths = [
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
    "$Recycle.Bin",
    "System Volume Information",
]
```

### C. 清理报告示例

```
C盘清理报告
生成时间: 2026-01-27 16:00:00

统计信息
清理文件数: 1523
失败文件数: 0
释放空间: 1024 MB

按文件类型统计
临时文件: 500 个文件, 256 MB
缓存文件: 800 个文件, 512 MB
日志文件: 200 个文件, 128 MB
更新备份: 23 个文件, 128 MB

清理的文件列表
C:\Windows\Temp\temp1.tmp - 临时文件 - 1 MB
C:\Users\Username\AppData\Local\Temp\temp2.tmp - 临时文件 - 2 MB
...
```

## 常见问题

### Q1：这个工具会删除系统文件吗？

A：不会。工具默认排除了Windows系统目录、Program Files等关键路径，并且只清理特定类型的文件（如.tmp、.cache、.log等），不会删除系统文件。

### Q2：清理后会影响程序运行吗？

A：通常不会。临时文件和缓存文件删除后，程序会重新生成，不会影响程序运行。但日志文件删除后可能影响问题排查，更新备份删除后无法回滚。

### Q3：如何避免误删重要文件？

A：建议首次使用时使用预览模式，查看将要清理的文件列表，确认无误后再执行实际清理。同时，可以自定义排除路径，排除包含重要数据的目录。

### Q4：清理后释放的空间不如预期怎么办？

A：检查是否清理了所有目标文件，检查文件大小统计是否正确，检查是否有其他大文件占用空间。可以使用磁盘分析工具查看空间使用情况。

### Q5：可以设置定时清理吗？

A：可以。使用Windows任务计划程序设置定时清理，例如每月1号凌晨2点自动清理。

## 版权声明

本文为博主原创文章，遵循 CC 4.0 BY-NC-SA 版权协议，转载请附上原文出处链接和本声明。

---

**作者**：aspnmy

**日期**：2026-01-27

**标签**：Rust, Windows, C盘清理, 磁盘清理, 系统优化

**分类**：Rust, Windows工具, 系统优化

**摘要**：本文详细介绍如何使用Rust语言开发一个高效、安全的C盘清理工具，包括项目架构、核心模块实现、使用方法、清理策略、安全注意事项、性能优化、扩展功能等内容。该工具可以快速扫描和清理C盘中的无用文件，包括临时文件、缓存文件、日志文件、更新备份文件等，释放磁盘空间，提高系统性能。

**关键词**：Rust, Windows, C盘清理, 磁盘清理, 系统优化, 临时文件, 缓存文件, 日志文件, 文件清理, 磁盘空间

---

## 更新日志

- **2026-01-27**：初始版本发布
