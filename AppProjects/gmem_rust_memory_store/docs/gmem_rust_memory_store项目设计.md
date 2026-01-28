# GmemoryStore 项目设计文档

## 1. 项目概述

GmemoryStore 是一个使用 Rust 语言实现的高性能、类型安全的记忆存储服务。它支持记忆的增删改查、智能搜索、相关性评分、确定性压缩等功能，为 Copilot 等 AI 助手提供持久化的记忆管理能力。

**更新时间**：2026-01-28 19:36:00 UTC+8

## 2. 核心功能

### 2.1 记忆管理
- **添加记忆**：支持文本内容和用户标签
- **搜索记忆**：基于关键词、标签、文本内容的相关性评分
- **删除记忆**：支持软删除和硬删除
- **压缩记忆**：将相关记忆压缩为预算约束的 markdown 块
- **导出/导入**：支持 JSON 格式的记忆导出和导入

### 2.2 技术特性
- **高性能**：基于 Rust 编译型语言，性能优异
- **类型安全**：充分利用 Rust 的类型系统，避免运行时错误
- **并发安全**：通过文件锁机制保证多进程并发访问安全
- **原子操作**：使用临时文件+重命名模式保证数据一致性
- **上海时区**：所有时间戳使用 UTC+8 时区
- **完整 CLI**：提供交互式命令行接口
- **版本管理**：自动生成包含时间戳的版本号
- **图标嵌入**：支持 Windows 平台的图标嵌入

## 3. 项目结构

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
│   └── mcp_server.rs  # MCP 服务器（预留）
├── build.rs           # 构建脚本（版本号生成、图标编译）
├── build_and_rename.ps1 # 编译和重命名脚本
├── Cargo.toml         # 项目配置文件
├── devrom.ico         # 应用图标
├── icon.rc            # 图标资源配置
├── README.md          # 项目说明文档
├── test_regex.rs      # 测试文件
├── ver                # 版本号文件
└── .gitignore         # Git忽略文件
```

## 4. 核心数据结构

### 4.1 记忆记录

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

### 4.2 搜索结果

```rust
pub struct SearchResult {
    pub record: MemoryRecord,    // 记忆记录
    pub score: f64,              // 相关性评分
}
```

## 5. 核心算法

### 5.1 相关性评分算法

评分公式：
- 文本中每个关键词匹配：+5 分
- 每个标签匹配：+8 分
- 每个提取的关键词匹配：+6 分
- 时效性：+0-5 分（更新越近分数越高）

### 5.2 关键词提取算法

- 自动从文本中提取有意义的关键词
- 过滤常见英文停用词（如 "the", "is", "and" 等）
- 按频率排序，最多返回 10 个关键词

### 5.3 压缩算法

- 基于相关性评分排序记忆
- 计算每个记忆的字符数
- 在预算约束内选择最相关的记忆
- 生成 markdown 格式的压缩结果

## 6. 技术实现

### 6.1 版本管理

- **ver 文件**：存储大版本号（如 0.1.0）
- **build.rs**：自动生成完整版本号
  - 读取 ver 文件中的大版本号
  - 生成时间戳（YYYYMMDDHHSS）
  - 构建完整版本号格式：`v0.1.0-YYYYMMDDHHSS`
- **PowerShell 脚本**：自动编译并重命名可执行文件
  - 源文件：`GmemoryStore.exe`
  - 目标文件：`GmemoryStore_v0.1.0-YYYYMMDDHHSS.exe`

### 6.2 图标嵌入

- **Windows 平台**：使用 embed-resource 库嵌入图标
- **icon.rc**：定义图标资源文件
- **图标路径**：从 `E:\AI_Worker\exe_ico\devrom.ico` 复制到项目目录

### 6.3 文件锁定机制

- 使用原子文件创建（`create_new`）作为锁定机制
- 默认超时时间：2500ms
- 支持自定义超时时间

### 6.4 原子操作

- 使用临时文件 + 重命名模式保证数据一致性
- 避免写入过程中断导致数据损坏

### 6.5 时间戳处理

- 所有时间戳使用 UTC+8 时区（上海时区）
- 格式：`2026-01-28T12:34:56.789+08:00`

## 7. 命令行接口

### 7.1 启动方式

```bash
# 使用默认记忆文件路径
GmemoryStore

# 指定自定义记忆文件路径
GmemoryStore /path/to/memory.json
```

### 7.2 可用命令

| 命令 | 描述 | 参数 |
|------|------|------|
| add | 存储新记忆 | `[--tags a,b,c] <text>` |
| search | 搜索记忆 | `<query> [--limit N]` |
| delete | 软删除记忆 | `<id>` |
| purge | 硬删除记忆 | `[--id ID] [--tag TAG] [--text TEXT]` |
| compress | 压缩记忆 | `<query> [--budget N] [--limit N]` |
| stats | 显示记忆统计信息 | 无 |
| export | 导出所有记忆为 JSON | 无 |
| import | 从 JSON 文件导入记忆 | `<json_file>` |
| logs show | 显示最近日志 | 无 |
| logs clear | 清除所有日志 | 无 |
| logs status | 显示日志状态 | 无 |
| whereiscfg | 显示配置文件路径 | 无 |
| help | 显示帮助信息 | 无 |
| exit | 退出 CLI | 无 |

## 8. 依赖项

| 依赖项 | 版本 | 用途 |
|--------|------|------|
| chrono | 0.4 | 时间处理（上海时区支持） |
| serde | 1.0 | 序列化/反序列化 |
| serde_json | 1.0 | JSON 处理 |
| regex | 1.12 | 正则表达式（关键词提取） |
| getrandom | 0.2 | 随机数生成 |
| fastrand | 2.3 | 快速随机数 |
| dirs | 5.0 | 跨平台目录路径 |
| embed-resource | 1.8 | Windows 图标嵌入 |
| winres | 0.1.12 | Windows 资源管理 |

## 9. 构建和部署

### 9.1 从源码编译

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

### 9.2 直接下载

从 [Releases](https://github.com/aspnmy/copilot-memory-store/releases) 页面下载对应平台的可执行文件。

## 10. 测试和验证

### 10.1 功能测试

- **添加记忆**：测试记忆的添加功能
- **搜索记忆**：测试搜索功能和相关性评分
- **删除记忆**：测试软删除和硬删除功能
- **压缩记忆**：测试压缩功能和预算控制
- **导出/导入**：测试 JSON 格式的导出和导入
- **日志管理**：测试日志的显示、清除和状态查询

### 10.2 性能测试

- **并发访问**：测试多进程并发访问的安全性
- **大数据量**：测试处理大量记忆的性能
- **搜索性能**：测试搜索操作的响应时间

## 11. 版本历史

| 版本 | 日期 | 描述 |
|------|------|------|
| v0.1.0-20260128193206 | 2026-01-28 | 初始版本 |
| v0.1.0-20260128192739 | 2026-01-28 | 修复图标嵌入 |
| v0.1.0-20260128191116 | 2026-01-28 | 修复版本号显示 |

## 12. 未来计划

- [ ] 实现 MCP 服务器功能
- [ ] 添加更多平台的图标支持
- [ ] 优化搜索算法
- [ ] 添加更多的记忆管理功能
- [ ] 支持更多的导入/导出格式
- [ ] 添加 GUI 界面

## 13. 总结

GmemoryStore 项目成功实现了一个高性能、类型安全的记忆存储服务，支持记忆的增删改查、智能搜索、相关性评分、确定性压缩等功能。通过使用 Rust 语言的特性，项目实现了并发安全、原子操作等特性，保证了数据的一致性和可靠性。

项目还实现了版本管理、图标嵌入等辅助功能，提高了用户体验和项目的可维护性。未来，项目将继续完善 MCP 服务器功能，添加更多平台的支持，优化搜索算法，为用户提供更好的记忆管理服务。

**更新时间**：2026-01-28 19:36:00 UTC+8
