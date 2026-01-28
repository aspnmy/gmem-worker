# gmem_rust_memory_store 版本记忆存储服务设计文档
## 项目名
- gmem_rust_memory_store

## 项目git路径
- git clone -b gmem_rust_memory_store https://github.com/aspnmy/copilot-memory-store.git

## 项目概述

本文档描述如何使用 Rust 语言重新实现 Copilot Memory Store 的核心功能，提供高性能、类型安全的记忆存储服务。

## 核心功能

### 1. 记忆记录结构

```rust
/// 记忆记录结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// 唯一标识符（格式：m_YYYYMMDDTHHMMSSZ_randomhex）
    pub id: String,
    /// 记忆内容文本
    pub text: String,
    /// 用户提供的标签用于分类
    pub tags: Vec<String>,
    /// 自动提取的关键词用于改进搜索
    pub keywords: Vec<String>,
    /// 记忆创建时的 ISO 时间戳
    pub created_at: String,
    /// 记忆最后修改时的 ISO 时间戳
    pub updated_at: String,
    /// 如果软删除则为 ISO 时间戳，否则为 null
    pub deleted_at: Option<String>,
}

/// 记忆存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    /// 总记录数（包括已删除的）
    pub total: usize,
    /// 活跃记录数（未删除的）
    pub active: usize,
    /// 软删除记录数
    pub deleted: usize,
    /// 标签频率映射
    pub tags: HashMap<String, usize>,
}

/// 带相关性分数的搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub text: String,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    /// 相关性分数（越高越相关）
    pub score: f64,
}

/// 确定性压缩的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    /// 压缩后的 markdown 输出
    pub markdown: String,
    /// 包含的搜索命中
    pub included: Vec<SearchHit>,
    /// 请求的字符预算
    pub budget: usize,
    /// 实际使用的字符数
    pub used: usize,
}
```

### 2. 时间戳处理（上海时区 UTC+8）

```rust
use chrono::{DateTime, Utc, Timelike, Local};

/// 返回当前时间作为上海时区的 ISO 字符串
pub fn now_iso() -> String {
    let now = Utc::now();
    let shanghai_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let local_time = now.with_timezone(&shanghai_offset);
    local_time.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
}

/// 生成带时间戳和随机后缀的唯一记忆 ID（上海时区）
pub fn make_id() -> String {
    let now = Utc::now();
    let shanghai_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let local_time = now.with_timezone(&shanghai_offset);
    
    let ts = local_time.format("%Y%m%dT%H%M%S%fZ").to_string();
    let rand: String = (0..3)
        .map(|_| {
            let mut buf = [0u8; 1];
            getrandom::getrandom(&mut buf).unwrap();
            format!("{:02x}", buf[0])
        })
        .collect();
    
    format!("m_{}_{}", ts, rand)
}
```

### 3. 关键词提取

```rust
use regex::Regex;
use std::collections::HashMap;

/// 常见英文停用词，在关键词提取时过滤掉
/// 这些词太常见，对搜索相关性没有帮助
const STOP_WORDS: &[&str] = &[
    "i", "me", "my", "we", "our", "you", "your", "he", "she", "it", "they", "them",
    "a", "an", "the", "this", "that", "these", "those",
    "is", "am", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
    "can", "may", "might", "must", "shall",
    "and", "or", "but", "if", "then", "else", "when", "where", "why", "how",
    "all", "each", "every", "both", "few", "more", "most", "some", "any", "no",
    "not", "only", "own", "same", "so", "than", "too", "very",
    "just", "also", "now", "here", "there", "about", "after", "before",
    "to", "from", "up", "down", "in", "out", "on", "off", "over", "under",
    "with", "without", "for", "of", "at", "by", "as", "into", "through",
    "like", "want", "use", "using", "used", "prefer", "always", "never",
];

/// 从文本中提取有意义的关键词用于搜索索引
/// 过滤停用词并按频率返回前 10 个
///
/// # 参数
/// * `text` - 要提取关键词的文本
///
/// # 返回
/// 最多 10 个关键词的数组，按频率排序
pub fn extract_keywords(text: &str) -> Vec<String> {
    let word_re = Regex::new(r"[a-z0-9]+").unwrap();
    let words: Vec<String> = word_re
        .find_iter(text.to_lowercase().as_str())
        .map(|m| m.as_str().to_string())
        .filter(|w| w.len() > 2 && !STOP_WORDS.contains(&w.as_str()))
        .collect();

    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in &words {
        *freq.entry(word.clone()).or_insert(0) += 1;
    }

    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter()
        .take(10)
        .map(|(word, _)| word)
        .collect()
}
```

### 4. 文件锁定机制

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

/// 获取文件锁以实现并发访问安全
/// 使用原子文件创建（wx 标志）作为锁定机制
///
/// # 参数
/// * `lock_path` - 锁文件路径
/// * `timeout_ms` - 等待锁的最大时间（默认 2500ms）
///
/// # 错误
/// 如果在超时时间内无法获取锁则返回错误
pub fn acquire_lock(lock_path: &Path, timeout_ms: Option<u64>) -> io::Result<File> {
    let timeout = timeout_ms.unwrap_or(2500);
    let start = std::time::Instant::now();

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(file) => {
                writeln!(file, "{} {}", std::process::id(), now_iso())?;
                return Ok(file);
            }
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                if start.elapsed().as_millis() > timeout {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        format!("Timed out acquiring lock: {}", lock_path.display()),
                    ));
                }
                thread::sleep(Duration::from_millis(50 + fastrand::u64(..50)));
            }
            Err(e) => return Err(e),
        }
    }
}

/// 通过删除锁文件来释放文件锁
pub fn release_lock(lock_path: &Path) {
    let _ = fs::remove_file(lock_path);
}
```

### 5. 记忆存储核心

```rust
use std::fs;
use std::path::{Path, PathBuf};
use serde_json;

const DEFAULT_MEMORY_PATH: &str = ".copilot-memory.json";
const DEFAULT_LOCK_NAME: &str = ".copilot-memory.lock";

/// 记忆存储结构
pub struct MemoryStore {
    memory_path: PathBuf,
    lock_path: PathBuf,
}

impl MemoryStore {
    /// 创建新的记忆存储实例
    pub fn new(memory_path: Option<&str>) -> Self {
        let mp = resolve_memory_path(memory_path);
        let lock = resolve_lock_path(&mp);
        Self {
            memory_path: mp,
            lock_path: lock,
        }
    }

    /// 从磁盘加载记忆存储
    ///
    /// # 返回
    /// 包含所有记录的向量
    pub fn load(&self) -> io::Result<Vec<MemoryRecord>> {
        if !self.memory_path.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(&self.memory_path)?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        let data: Vec<MemoryRecord> = serde_json::from_str(&raw)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(data)
    }

    /// 添加新记忆到存储
    /// 自动从文本中提取关键词以改进搜索
    ///
    /// # 参数
    /// * `text` - 记忆内容（必需）
    /// * `tags` - 用于分类的可选标签
    ///
    /// # 返回
    /// 创建的记忆记录
    ///
    /// # 错误
    /// 如果文本为空则返回错误
    pub fn add_memory(&self, text: &str, tags: Option<Vec<String>>) -> io::Result<MemoryRecord> {
        let _lock = acquire_lock(&self.lock_path, None)?;
        let records = self.load()?;

        let t = text.trim();
        if t.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot add an empty memory."));
        }

        let keywords = extract_keywords(t);
        let rec = MemoryRecord {
            id: make_id(),
            text: t.to_string(),
            tags: normalize_tags(tags),
            keywords,
            created_at: now_iso(),
            updated_at: now_iso(),
            deleted_at: None,
        };

        let mut new_records = records;
        new_records.push(rec.clone());
        atomic_write(&self.memory_path, &new_records)?;

        Ok(rec)
    }

    /// 搜索记忆并按相关性排序
    ///
    /// # 参数
    /// * `query` - 搜索查询（空格分隔的关键词）
    /// * `limit` - 返回的最大结果数（默认 10）
    ///
    /// # 返回
    /// 按分数降序排列的搜索命中数组
    pub fn search(&self, query: &str, limit: Option<usize>) -> io::Result<Vec<SearchHit>> {
        let records = self.load()?;
        let limit = limit.unwrap_or(10);

        let mut hits: Vec<SearchHit> = Vec::new();
        for r in &records {
            if r.deleted_at.is_some() {
                continue;
            }
            let score = score_record(r, query);
            if score <= 0.0 {
                continue;
            }
            hits.push(SearchHit {
                id: r.id.clone(),
                text: r.text.clone(),
                tags: r.tags.clone(),
                keywords: r.keywords.clone(),
                created_at: r.created_at.clone(),
                updated_at: r.updated_at.clone(),
                score,
            });
        }

        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(hits.into_iter().take(std::cmp::max(1, limit)).collect())
    }

    /// 计算记忆存储的统计信息
    ///
    /// # 参数
    /// * `records` - 记忆记录数组
    ///
    /// # 返回
    /// 包括计数和标签频率的统计信息
    pub fn compute_stats(&self) -> io::Result<StoreStats> {
        let records = self.load()?;

        let mut tags: HashMap<String, usize> = HashMap::new();
        let mut deleted = 0;

        for r in &records {
            if r.deleted_at.is_some() {
                deleted += 1;
            }
            for t in &r.tags {
                *tags.entry(t.clone()).or_insert(0) += 1;
            }
        }

        Ok(StoreStats {
            total: records.len(),
            active: records.len() - deleted,
            deleted,
            tags,
        })
    }
}

/// 规范化标签为小写、修剪、唯一值
fn normalize_tags(tags: Option<Vec<String>>) -> Vec<String> {
    match tags {
        Some(tags) => {
            let mut out = std::collections::HashSet::new();
            for t in tags {
                let cleaned = t.trim().to_lowercase();
                if !cleaned.is_empty() {
                    out.insert(cleaned);
                }
            }
            out.into_iter().collect()
        }
        None => Vec::new(),
    }
}

/// 解析记忆文件路径
fn resolve_memory_path(p: Option<&str>) -> PathBuf {
    let raw = p.unwrap_or(DEFAULT_MEMORY_PATH).trim();
    let path = Path::new(raw);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    }
}

/// 解析锁文件路径
fn resolve_lock_path(memory_path: &Path) -> PathBuf {
    memory_path.parent().unwrap().join(DEFAULT_LOCK_NAME)
}

/// 使用临时文件 + 重命名模式原子性写入
fn atomic_write(path: &Path, data: &Vec<MemoryRecord>) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = format!("{}.tmp.{}.tmp", path.display(), std::process::id());
    let tmp = Path::new(&tmp_path);

    let json = serde_json::to_string_pretty(data, Default::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;

    Ok(())
}
```

### 6. 相关性评分

```rust
use regex::Regex;

/// 计算记录相对于查询的相关性分数
///
/// 评分公式：
/// - 文本中每个关键词匹配 +5 分
/// - 每个标签匹配 +8 分
/// - 每个提取的关键词匹配 +6 分
/// - 时效性 +0-5 分（更新 = 更高）
///
/// # 参数
/// * `r` - 要评分的记忆记录
/// * `query` - 搜索查询
///
/// # 返回
/// 数值相关性分数（0 = 无匹配）
fn score_record(r: &MemoryRecord, query: &str) -> f64 {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0.0;
    }

    let text = r.text.to_lowercase();
    let mut score = 0.0;

    for token in q.split_whitespace() {
        let re = Regex::new(&format!(r"(?i){}", regex::escape(token))).unwrap();
        let hits = re.find_iter(&text).count();
        score += hits as f64 * 5.0;

        // 标签匹配奖励
        if r.tags.iter().any(|t| t.to_lowercase() == token) {
            score += 8.0;
        }

        // 关键词匹配奖励（预索引的关键词）
        if r.keywords.iter().any(|k| k == token) {
            score += 6.0;
        }
    }

    // 时效性评分
    let age_ms = chrono::Utc::now()
        .signed_duration_since(
            DateTime::parse_from_rfc3339(&r.updated_at)
                .unwrap_or_else(|_| DateTime::parse_from_rfc3339(&r.created_at).unwrap())
                .with_timezone(&chrono::Utc)
        )
        .num_milliseconds()
        .abs();

    let days = age_ms as f64 / (1000.0 * 60.0 * 60.0 * 24.0);
    let recency = (5.0 - (days / 30.0).min(5.0)).max(0.0);
    score += recency;

    score
}
```

### 7. 确定性压缩

```rust
/// 确定性地将相关记忆压缩为预算约束的 markdown 块
/// 使用确定性截断（无 LLM）- 包含记忆直到预算耗尽
///
/// # 参数
/// * `records` - 记忆记录数组
/// * `query` - 查找相关记忆的搜索查询
/// * `budget` - 输出的最大字符数（最小 200）
/// * `limit` - 考虑的最大记忆数（默认 25）
///
/// # 返回
/// 带有 markdown 和元数据的 CompressResult
pub fn compress_deterministic(
    records: &Vec<MemoryRecord>,
    query: &str,
    budget: usize,
    limit: Option<usize>,
) -> CompressResult {
    let budget = budget.max(200);
    let limit = limit.unwrap_or(25);

    let hits = search_records(records, query, Some(limit));

    let mut lines: Vec<String> = Vec::new();
    lines.push("# Copilot Context (auto)".to_string());
    lines.push(String::new());
    lines.push("## Relevant memory".to_string());

    for h in &hits {
        let tag_str = if h.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", h.tags.join(", "))
        };
        lines.push(format!("- ({}){} {}", h.id, tag_str, h.text));
    }

    let md = lines.join("\n") + "\n";
    if md.len() <= budget {
        return CompressResult {
            markdown: md,
            included: hits,
            budget,
            used: md.len(),
        };
    }

    let mut out: Vec<String> = Vec::new();
    let mut size = 0;
    for line in &lines {
        if size + line.len() + 1 > budget {
            break;
        }
        out.push(line.clone());
        size += line.len() + 1;
    }

    let md2 = out.join("\n") + "\n";
    CompressResult {
        markdown: md2,
        included: hits,
        budget,
        used: md2.len(),
    }
}

/// 搜索记录（内部辅助函数）
fn search_records(
    records: &Vec<MemoryRecord>,
    query: &str,
    limit: Option<usize>,
) -> Vec<SearchHit> {
    let mut hits: Vec<SearchHit> = Vec::new();
    let limit = limit.unwrap_or(10);

    for r in records {
        if r.deleted_at.is_some() {
            continue;
        }
        let score = score_record(r, query);
        if score <= 0.0 {
            continue;
        }
        hits.push(SearchHit {
            id: r.id.clone(),
            text: r.text.clone(),
            tags: r.tags.clone(),
            keywords: r.keywords.clone(),
            created_at: r.created_at.clone(),
            updated_at: r.updated_at.clone(),
            score,
        });
    }

    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hits.into_iter().take(std::cmp::max(1, limit)).collect()
}
```

### 8. 命令行接口

```rust
use std::io::{self, Write};

/// 解析的命令结构
#[derive(Debug)]
pub struct Parsed {
    pub cmd: String,
    pub args: Vec<String>,
    pub opts: HashMap<String, String>,
}

/// 解析命令行字符串
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
        if t.starts_with("--") {
            let key = &t[2..];
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
fn tokenize(line: &str) -> Vec<String> {
    let re = Regex::new(r#""([^"\\]*(?:\\.[^"\\]*)*"|'([^'\\]*(?:\\.[^'\\]*)*)'|(\S+)"#).unwrap();
    re.find_iter(line)
        .map(|m| {
            let token = m.as_str();
            token.replace(r#"\""#, "\"").replace(r#"\'"#, "'")
        })
        .collect()
}

/// 主命令行 REPL 循环
pub fn run_repl(store: MemoryStore) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Copilot Memory Store CLI");
    println!("Type 'help' for available commands, 'exit' to quit\n");

    loop {
        print!("> ");
        stdout.flush()?;

        let mut line = String::new();
        stdin.read_line(&mut line)?;

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
fn execute_command(store: &MemoryStore, parsed: &Parsed) -> io::Result<()> {
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
            for (i, hit) in hits.iter().enumerate() {
                println!("{}. {} (score: {:.1})", i + 1, hit.text, hit.score);
            }
        }
        "stats" => {
            let stats = store.compute_stats()?;
            println!("Total: {}, Active: {}, Deleted: {}", stats.total, stats.active, stats.deleted);
        }
        "help" => {
            println!("Available commands:");
            println!("  add [--tags a,b,c] <text>  - Store a new memory");
            println!("  search <query> [--limit N]  - Search memories");
            println!("  stats                       - Show memory statistics");
            println!("  help                        - Show this help");
            println!("  exit                        - Quit CLI");
        }
        _ => {
            println!("Unknown command: {}", parsed.cmd);
        }
    }

    Ok(())
}
```

## Cargo.toml 依赖配置

```toml
[package]
name = "rust-memory-store"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
getrandom = "0.2"
fastrand = "2.0"

[dev-dependencies]
criterion = "0.5"

[[bin]]
name = "memory-store"
path = "src/main.rs"
```

## 项目结构

```
rust-memory-store/
├── Cargo.toml
├── src/
│   ├── main.rs           # 命令行入口
│   ├── lib.rs           # 库入口
│   ├── store.rs         # 记忆存储核心
│   ├── record.rs        # 记录结构定义
│   ├── search.rs        # 搜索功能
│   ├── compress.rs      # 压缩功能
│   ├── lock.rs         # 文件锁定
│   ├── timestamp.rs     # 时间戳处理
│   ├── keywords.rs      # 关键词提取
│   └── cli.rs          # 命令行接口
└── tests/
    ├── store_tests.rs
    ├── search_tests.rs
    └── integration_tests.rs
```

## 编译和运行

### 编译

```bash
cargo build --release
```

### 运行

```bash
# 开发模式
cargo run

# 发布模式
cargo run --release

# 命令行参数
cargo run -- add "测试记忆" --tags gmem,test
cargo run -- search "测试"
cargo run -- stats
```

## 性能优化

1. **内存效率**：使用 `Vec` 和 `HashMap` 进行高效内存管理
2. **并发安全**：使用文件锁定机制防止并发写入冲突
3. **原子操作**：使用临时文件 + 重命名模式确保写入原子性
4. **搜索优化**：预提取关键词并使用正则表达式快速匹配
5. **类型安全**：利用 Rust 的类型系统在编译时捕获错误

## 与 TypeScript 版本的对比

| 特性 | TypeScript 版本 | Rust 版本 |
|------|---------------|-----------|
| 类型安全 | 运行时检查 | 编译时检查 |
| 内存安全 | 依赖 GC | 所有权系统 |
| 性能 | V8 引擎优化 | 零成本抽象 |
| 并发 | 事件循环 | 原生线程 + 锁 |
| 部署 | 需要 Node.js | 单一二进制文件 |
| 依赖管理 | npm | Cargo |

## 后续开发计划

1. **MCP 服务器**：实现 Rust 版本的 MCP 服务器（包括 7 个工具、2 个资源、3 个提示）
2. **异步 I/O**：使用 `tokio` 实现异步文件操作
3. **持久化优化**：考虑使用 SQLite 或其他嵌入式数据库
4. **网络服务**：添加 HTTP/REST API 支持
5. **加密支持**：为敏感记忆添加加密功能
6. **自动备份**：实现定时自动备份和恢复机制
7. **数据迁移**：实现记忆数据的迁移和版本升级
8. **DeepSeek LLM 集成**：实现智能上下文压缩和重塑功能

## 总结

本文档提供了 Rust 版本记忆存储服务的完整设计，包括：

- ✅ 核心数据结构定义
- ✅ 时间戳处理（上海时区）
- ✅ 关键词提取算法
- ✅ 文件锁定机制
- ✅ 记忆存储核心功能
- ✅ 搜索和相关性评分
- ✅ 确定性压缩
- ✅ 命令行接口
- ✅ 项目结构和依赖配置
- ✅ 软删除和硬删除功能
- ✅ 导出和导入功能
- ✅ 自动备份功能
- ✅ 配置管理功能
- ✅ DeepSeek LLM 集成
- ✅ MCP 服务器设计

## 附录：遗漏功能补充

### A. 软删除和硬删除

```rust
impl MemoryStore {
    /// 软删除记忆（标记为已删除）
    ///
    /// # 参数
    /// * `id` - 记忆 ID
    ///
    /// # 返回
    /// 是否找到并删除了记忆
    pub fn soft_delete(&self, id: &str) -> io::Result<bool> {
        let _lock = acquire_lock(&self.lock_path, None)?;
        let mut records = self.load()?;
        
        let found = records.iter_mut().any(|r| {
            if r.id == id && r.deleted_at.is_none() {
                r.deleted_at = Some(now_iso());
                r.updated_at = now_iso();
                true
            } else {
                false
            }
        });
        
        if found {
            atomic_write(&self.memory_path, &records)?;
        }
        
        Ok(found)
    }

    /// 硬删除记忆（永久删除）
    ///
    /// # 参数
    /// * `id` - 记忆 ID（可选）
    /// * `tag` - 标签匹配（可选）
    /// * `match_text` - 文本匹配（可选）
    ///
    /// # 返回
    /// 删除的记忆数量
    pub fn purge(&self, id: Option<&str>, tag: Option<&str>, match_text: Option<&str>) -> io::Result<usize> {
        let _lock = acquire_lock(&self.lock_path, None)?;
        let mut records = self.load()?;
        
        let initial_len = records.len();
        records.retain(|r| {
            if let Some(id_val) = id {
                if r.id == id_val {
                    return false;
                }
            }
            if let Some(tag_val) = tag {
                if r.tags.iter().any(|t| t == tag_val) {
                    return false;
                }
            }
            if let Some(text_val) = match_text {
                if r.text.contains(text_val) {
                    return false;
                }
            }
            true
        });
        
        let purged = initial_len - records.len();
        if purged > 0 {
            atomic_write(&self.memory_path, &records)?;
        }
        
        Ok(purged)
    }
}
```

### B. 导出和导入功能

```rust
impl MemoryStore {
    /// 导出所有记忆为 JSON 字符串
    ///
    /// # 返回
    /// JSON 格式的记忆数据
    pub fn export_json(&self) -> io::Result<String> {
        let records = self.load()?;
        serde_json::to_string_pretty(&records, Default::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    /// 从 JSON 导入记忆
    ///
    /// # 参数
    /// * `json_data` - JSON 格式的记忆数据
    ///
    /// # 返回
    /// (成功数量, 跳过数量, 失败数量)
    pub fn import_json(&self, json_data: &str) -> io::Result<(usize, usize, usize)> {
        let _lock = acquire_lock(&self.lock_path, None)?;
        let mut records = self.load()?;
        
        let imported: Vec<MemoryRecord> = serde_json::from_str(json_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let existing_ids: std::collections::HashSet<String> = 
            records.iter().map(|r| r.id.clone()).collect();
        
        let mut success = 0;
        let mut skipped = 0;
        let mut failed = 0;
        
        for mut rec in imported {
            if existing_ids.contains(&rec.id) {
                skipped += 1;
                continue;
            }
            
            rec.created_at = now_iso();
            rec.updated_at = now_iso();
            records.push(rec);
            success += 1;
        }
        
        atomic_write(&self.memory_path, &records)?;
        Ok((success, skipped, failed))
    }
}
```

### C. 自动备份功能

```rust
use std::fs;
use std::path::PathBuf;

/// 备份配置
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// 备份间隔（毫秒）
    pub interval_ms: u64,
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 最大备份数量
    pub max_backups: usize,
    /// 是否压缩
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            interval_ms: 3600000, // 1 小时
            backup_dir: PathBuf::from("backups"),
            max_backups: 10,
            compress: false,
        }
    }
}

/// 执行备份
///
/// # 参数
/// * `store` - 记忆存储实例
/// * `config` - 备份配置
///
/// # 返回
/// 备份文件路径
pub fn perform_backup(store: &MemoryStore, config: &BackupConfig) -> io::Result<PathBuf> {
    let json_data = store.export_json()?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("memories_backup_{}.json", timestamp);
    let backup_path = config.backup_dir.join(filename);
    
    fs::create_dir_all(&config.backup_dir)?;
    fs::write(&backup_path, json_data)?;
    
    // 清理旧备份
    cleanup_old_backups(config)?;
    
    Ok(backup_path)
}

/// 清理旧备份，保留最新的 max_backups 个
fn cleanup_old_backups(config: &BackupConfig) -> io::Result<()> {
    if !config.backup_dir.exists() {
        return Ok(());
    }
    
    let mut backups: Vec<_> = fs::read_dir(&config.backup_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();
    
    backups.sort_by_key(|e| e.path());
    
    while backups.len() > config.max_backups {
        if let Some(oldest) = backups.remove(0) {
            fs::remove_file(oldest.path())?;
        }
    }
    
    Ok(())
}
```

### D. 配置管理

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 记忆文件路径
    pub memory_path: PathBuf,
    /// 备份目录
    pub backup_dir: PathBuf,
    /// DeepSeek API 配置
    pub deepseek_api_key: Option<String>,
    pub deepseek_base_url: Option<String>,
    pub deepseek_model: Option<String>,
    /// 备份配置
    pub backup_interval_ms: u64,
    pub max_backups: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            memory_path: PathBuf::from(".copilot-memory.json"),
            backup_dir: PathBuf::from("backups"),
            deepseek_api_key: None,
            deepseek_base_url: Some("https://api.deepseek.com".to_string()),
            deepseek_model: Some("deepseek-chat".to_string()),
            backup_interval_ms: 3600000,
            max_backups: 10,
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: Config,
}

impl ConfigManager {
    /// 创建配置管理器
    pub fn new(config_path: Option<&Path>) -> io::Result<Self> {
        let config_path = config_path.unwrap_or(&Path::new(".copilot-memory-config.json")).to_path_buf();
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            Config::default()
        };
        
        Ok(Self { config_path, config })
    }

    /// 获取配置
    pub fn get(&self) -> &Config {
        &self.config
    }

    /// 保存配置
    pub fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.config, Default::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    /// 更新配置
    pub fn update<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut Config),
    {
        f(&mut self.config);
        self.save()
    }
}
```

### E. DeepSeek LLM 集成

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// DeepSeek API 配置
#[derive(Debug, Clone)]
pub struct DeepSeekConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

/// DeepSeek API 请求
#[derive(Debug, Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

/// DeepSeek API 响应
#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessage,
}

/// 使用 DeepSeek LLM 进行智能上下文压缩
///
/// # 参数
/// * `config` - DeepSeek 配置
/// * `query` - 用户查询
/// * `markdown_context` - 预压缩的 markdown 上下文
/// * `budget_chars` - 字符预算
///
/// # 返回
/// 压缩后的 markdown
pub async fn deepseek_compress(
    config: &DeepSeekConfig,
    query: &str,
    markdown_context: &str,
    budget_chars: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    
    let system_prompt = format!(
        "You compress user memory context for an LLM coding assistant. \
        Output must be Markdown. Be concise. Keep only information relevant to the user's query. \
        Hard limit: {} characters. Do not include any secrets.",
        budget_chars
    );
    
    let request = DeepSeekRequest {
        model: config.model.clone(),
        messages: vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: format!("Query:\n{}\n\nContext:\n{}", query, markdown_context),
            },
        ],
        temperature: 0.2,
    };
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("DeepSeek API error: {}", error_text).into());
    }
    
    let data: DeepSeekResponse = response.json().await?;
    let content = data.choices.get(0)
        .ok_or("No choices in response")?
        .message.content.clone();
    
    if content.len() > budget_chars {
        Ok(content.chars().take(budget_chars).collect())
    } else {
        Ok(content)
    }
}

/// 使用 DeepSeek LLM 进行智能上下文重塑
///
/// # 参数
/// * `config` - DeepSeek 配置
/// * `task` - 任务描述
/// * `markdown_context` - 预压缩的 markdown 上下文
/// * `budget_chars` - 字符预算
///
/// # 返回
/// 重塑后的上下文
pub async fn deepseek_shape(
    config: &DeepSeekConfig,
    task: &str,
    markdown_context: &str,
    budget_chars: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    
    let system_prompt = format!(
        "You transform user memory context into actionable guidance for a coding task. \
        Output must be Markdown. Be specific and practical. \
        Hard limit: {} characters. Do not include any secrets.",
        budget_chars
    );
    
    let request = DeepSeekRequest {
        model: config.model.clone(),
        messages: vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: format!(
                    "Task:\n{}\n\nContext:\n{}\n\nTransform the context into actionable guidance for this task.",
                    task, markdown_context
                ),
            },
        ],
        temperature: 0.3,
    };
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("DeepSeek API error: {}", error_text).into());
    }
    
    let data: DeepSeekResponse = response.json().await?;
    let content = data.choices.get(0)
        .ok_or("No choices in response")?
        .message.content.clone();
    
    if content.len() > budget_chars {
        Ok(content.chars().take(budget_chars).collect())
    } else {
        Ok(content)
    }
}
```

### F. MCP 服务器设计

MCP (Model Context Protocol) 服务器提供 7 个工具、2 个资源和 3 个提示：

#### 工具

1. **memory_write** - 添加记忆（支持标签建议）
2. **memory_search** - 搜索记忆（支持原始 JSON 输出）
3. **memory_compress** - 压缩上下文（支持 LLM 增强）
4. **memory_delete** - 软删除记忆
5. **memory_purge** - 硬删除记忆（支持确认）
6. **memory_export** - 导出所有记忆
7. **inject_context** - 注入任务上下文（支持 LLM 重塑）

#### 资源

1. **memory://stats** - 实时统计信息
2. **memory://recent** - 最近 10 条记忆

#### 提示

1. **summarize-memories** - 总结主题记忆
2. **remember-decision** - 记录架构决策（ADR 格式）
3. **inject-context** - 注入上下文提示

#### Elicitation 支持

- `memory_purge`: 请求用户确认永久删除
- `memory_write`: 提供现有标签选择

### G. 更新的 Cargo.toml 依赖

```toml
[package]
name = "rust-memory-store"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
getrandom = "0.2"
fastrand = "2.0"
tokio = { version = "1.0", features = ["full"], optional = true }
reqwest = { version = "0.11", features = ["json"], optional = true }
dirs = "5.0",  # 用于获取系统配置目录

[features]
default = []
async = ["tokio"]
llm = ["reqwest"]
full = ["async", "llm"]

[dev-dependencies]
criterion = "0.5"

[[bin]]
name = "memory-store"
path = "src/main.rs"

[[bin]]
name = "memory-mcp-server"
path = "src/mcp_server.rs"
required-features = ["full"]
```

这个设计遵循 Rust 最佳实践，提供高性能、类型安全的记忆存储解决方案，可以作为 TypeScript 版本的替代实现。
