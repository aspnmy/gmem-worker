# GmemoryStore

Rust 实现的高性能、类型安全的 Copilot Memory Store 记忆存储服务。

## 项目简介

本项目使用 Rust 语言重新实现了 Copilot Memory Store 的核心功能，提供了高性能、类型安全的记忆存储服务。支持记忆的增删改查、智能搜索、相关性评分、确定性压缩等功能。

## 核心特性

- **高性能**：基于 Rust 编译型语言，性能优异
- **类型安全**：充分利用 Rust 的类型系统，避免运行时错误
- **并发安全**：通过文件锁机制保证多进程并发访问安全
- **原子操作**：使用临时文件+重命名模式保证数据一致性
- **智能搜索**：基于关键词、标签、文本内容的相关性评分
- **确定性压缩**：将相关记忆压缩为预算约束的 markdown 块
- **上海时区**：所有时间戳使用 UTC+8 时区
- **完整 CLI**：提供交互式命令行接口
- **版本管理**：自动生成包含时间戳的版本号
- **图标嵌入**：支持 Windows 平台的图标嵌入
- **日志管理**：提供日志的显示、清除和状态查询功能

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone -b gmem_rust_memory_store https://github.com/aspnmy/copilot-memory-store.git
cd copilot-memory-store

# 编译 release 版本
cargo build --release

# 或使用 PowerShell 脚本（自动编译并重命名）
powershell -ExecutionPolicy Bypass -File build_and_rename.ps1

# 编译后的可执行文件位于 target/release/GmemoryStore_v0.1.0-YYYYMMDDHHSS.exe
```

### 直接下载

从 [Releases](https://github.com/aspnmy/copilot-memory-store/releases) 页面下载对应平台的可执行文件。

## 使用方法

### 启动 CLI

```bash
# 使用默认记忆文件路径
GmemoryStore

# 指定自定义记忆文件路径
GmemoryStore /path/to/memory.json
```

### 命令列表

#### 添加记忆

```bash
> add 这是一个测试记忆
✅ Added m_20260128T123456789Z_abc

> add --tags 工作,重要 完成项目重构任务
✅ Added m_20260128T123456789Z_def
```

#### 搜索记忆

```bash
> search 测试
1. 这是一个测试记忆 (score: 25.0)

> search 工作 --limit 5
1. 完成项目重构任务 [工作, 重要] (score: 33.0)
```

#### 查看统计

```bash
> stats
Total: 10, Active: 8, Deleted: 2

Tags:
  - 工作: 5
  - 重要: 3
  - 学习: 2
```

#### 软删除记忆

```bash
> delete m_20260128T123456789Z_abc
✅ Deleted m_20260128T123456789Z_abc
```

#### 硬删除记忆

```bash
# 按 ID 删除
> purge --id m_20260128T123456789Z_abc
✅ Purged 1 memories

# 按标签删除
> purge --tag 工作
✅ Purged 5 memories

# 按文本内容删除
> purge --text 测试
✅ Purged 2 memories
```

#### 压缩记忆

```bash
> compress 项目 --budget 1000 --limit 10
--- Compressed Output (856 / 1000 chars) ---
# Copilot Context (auto)

## Relevant memory
- (m_20260128T123456789Z_def) [工作, 重要] 完成项目重构任务
- (m_20260128T123456789Z_ghi) [学习] 学习 Rust 语言
--- End ---
```

#### 导出记忆

```bash
> export
[
  {
    "id": "m_20260128T123456789Z_def",
    "text": "完成项目重构任务",
    "tags": ["工作", "重要"],
    "keywords": ["项目", "重构", "任务"],
    "created_at": "2026-01-28T12:34:56.789+08:00",
    "updated_at": "2026-01-28T12:34:56.789+08:00",
    "deleted_at": null
  }
]
```

#### 导入记忆

```bash
> import memories.json
✅ Imported: 5, Skipped: 2, Failed: 0
```

#### 日志管理

```bash
# 显示最近日志
> logs show

# 清除所有日志
> logs clear
✅ Logs cleared

# 显示日志状态
> logs status
Logs file size: 1.2 KB
```

#### 配置管理

```bash
# 显示配置文件路径
> whereiscfg
Config file: C:\Users\username\.config\GmemoryStore\config.toml
```

#### 帮助

```bash
> help
Available commands:
  add [--tags a,b,c] <text>      - Store a new memory
  search <query> [--limit N]     - Search memories
  delete <id>                    - Soft delete a memory
  purge [--id ID] [--tag TAG] [--text TEXT] - Hard delete memories
  compress <query> [--budget N] [--limit N] - Compress memories
  stats                          - Show memory statistics
  export                         - Export all memories as JSON
  import <json_file>             - Import memories from JSON file
  logs show                      - Show recent logs
  logs clear                     - Clear all logs
  logs status                    - Show logs status
  whereiscfg                     - Show config file path
  help                           - Show this help
  exit                           - Quit CLI
```

## 核心功能

### MCP 服务器

本项目实现了 MCP (Model Context Protocol) 服务器，用于与 AI 模型进行交互，提供记忆管理功能。

#### 启动 MCP 服务器

```bash
# 使用默认配置
cargo run --release --bin gmemory_mcp_server

# 或使用编译后的可执行文件
target/release/gmemory_mcp_server
```

#### MCP 服务器工具

MCP 服务器实现了以下工具：

- `add_memory` - 添加新记忆
- `search_memory` - 搜索记忆
- `compress_memory` - 压缩记忆
- `delete_memory` - 删除记忆
- `get_stats` - 获取记忆存储统计信息

### 记忆记录结构

```rust
pub struct MemoryRecord {
    pub id: String,              // 唯一标识符
    pub text: String,            // 记忆内容
    pub tags: Vec<String>,       // 用户标签
    pub keywords: Vec<String>,    // 自动提取的关键词
    pub created_at: String,      // 创建时间（上海时区）
    pub updated_at: String,      // 更新时间（上海时区）
    pub deleted_at: Option<String>, // 删除时间（软删除）
}
```

### 相关性评分算法

评分公式：
- 文本中每个关键词匹配：+5 分
- 每个标签匹配：+8 分
- 每个提取的关键词匹配：+6 分
- 时效性：+0-5 分（更新越近分数越高）

### 关键词提取

- 自动从文本中提取有意义的关键词
- 过滤常见英文停用词（如 "the", "is", "and" 等）
- 按频率排序，最多返回 10 个关键词

### 文件锁定机制

- 使用原子文件创建（`create_new`）作为锁定机制
- 默认超时时间：2500ms
- 支持自定义超时时间

### 原子操作

- 使用临时文件 + 重命名模式
- 确保数据一致性，避免写入过程中断导致数据损坏

### 版本管理

- **ver 文件**：存储大版本号（如 0.1.0）
- **build.rs**：自动生成完整版本号
  - 读取 ver 文件中的大版本号
  - 生成时间戳（YYYYMMDDHHSS）
  - 构建完整版本号格式：`v0.1.0-YYYYMMDDHHSS`
- **PowerShell 脚本**：自动编译并重命名可执行文件
  - 源文件：`GmemoryStore.exe`
  - 目标文件：`GmemoryStore_v0.1.0-YYYYMMDDHHSS.exe`

### 图标嵌入

- **Windows 平台**：使用 embed-resource 库嵌入图标
- **icon.rc**：定义图标资源文件
- **图标路径**：从 `E:\AI_Worker\exe_ico\devrom.ico` 复制到项目目录

## 开发指南

### 项目结构

```
gmem_rust_memory_store/
├── src/
│   ├── record.rs      # 核心数据结构
│   ├── timestamp.rs   # 时间戳处理（上海时区）
│   ├── keywords.rs    # 关键词提取
│   ├── lock.rs        # 文件锁定机制
│   ├── store.rs       # 记忆存储核心
│   ├── compress.rs    # 确定性压缩
│   ├── logs.rs        # 日志管理
│   ├── config.rs      # 配置管理
│   ├── cli.rs         # 命令行接口
│   ├── lib.rs         # 库入口
│   ├── main.rs        # CLI 可执行文件
│   ├── mcp_server.rs  # MCP 服务器
│   ├── mcp_serialization.rs # MCP 序列化
├── src/bin/
│   ├── organize_timer.rs    # 定时整理内存
│   ├── organize_once.rs     # 单次整理内存
│   ├── remove_timer_lock.rs # 移除定时锁
│   ├── lock_cleaner.rs      # 锁清理工具
│   ├── import_json.rs       # JSON 导入工具
│   └── ...
├── build.rs           # 构建脚本（版本号生成、图标编译）
├── build_and_rename.ps1 # 编译和重命名脚本
├── build_all.ps1      # 一次性编译所有版本脚本
├── Cargo.toml         # 项目配置文件
├── devrom.ico         # 应用图标
├── icon.rc            # 图标资源配置
├── README.md          # 项目说明文档
├── test_regex.rs      # 测试文件
├── ver                # 版本号文件
└── .gitignore         # Git忽略文件
```

### 依赖项

- `chrono` - 时间处理（上海时区支持）
- `serde` / `serde_json` - 序列化/反序列化
- `regex` - 正则表达式（关键词提取）
- `getrandom` - 随机数生成
- `fastrand` - 快速随机数
- `dirs` - 跨平台目录路径
- `embed-resource` - Windows 图标嵌入
- `winres` - Windows 资源管理

### 编译和测试

```bash
# 开发版本编译
cargo build

# Release 版本编译
cargo build --release

# 或使用 PowerShell 脚本（自动编译并重命名）
powershell -ExecutionPolicy Bypass -File build_and_rename.ps1

# 一次性编译所有版本（推荐）
powershell -ExecutionPolicy Bypass -File build_all.ps1

# 运行测试
cargo test

# 代码检查
cargo clippy --all-targets --all-features -- -D warnings

# 格式化代码
cargo fmt
```

### 特性标志

- `default` - 默认特性（无额外功能）
- `async` - 启用异步支持（需要 tokio）
- `llm` - 启用 LLM 压缩功能（需要 reqwest）
- `full` - 启用所有特性（`async` + `llm`）

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 作者

aspnmy <support@e2bank.cn>

## 相关链接

- [GitHub 仓库](https://github.com/aspnmy/copilot-memory-store)
- [设计文档](https://github.com/aspnmy/copilot-memory-store/blob/gmem_rust_memory_store/docs/gmem_rust_memory_store项目设计.md)