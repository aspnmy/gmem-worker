# C盘无用文件清理工具

一个使用Rust语言开发的高效、安全的C盘清理工具，用于快速扫描和清理Windows系统C盘中的无用文件。

## 功能特性

- **快速扫描**：高效扫描C盘中的无用文件
- **安全清理**：只清理确认安全的文件，避免误删重要文件
- **详细报告**：生成详细的清理报告，包括文件类型、大小、数量等
- **自定义配置**：支持自定义清理规则和排除路径
- **批量处理**：支持批量删除文件，提高清理效率
- **预览模式**：默认使用预览模式，查看将要清理的文件

## 支持的文件类型

- 临时文件（*.tmp, *.temp）
- 缓存文件（*.cache, *.cach）
- 日志文件（*.log）
- 更新备份文件（*.old, *.bak）
- 浏览器缓存
- 系统临时文件

## 安装

### 前置要求

- Rust >= 1.70
- Windows 操作系统

### 编译

```bash
# 克隆项目
git clone https://github.com/aspnmy/rust_disk_cleaner.git
cd rust_disk_cleaner

# 编译项目
cargo build --release

# 编译后的可执行文件位于 target/release/disk_cleaner.exe
```

## 使用方法

### 基本使用

#### 预览模式（推荐）

```bash
# 预览模式，扫描默认路径
.\target\release\disk_cleaner.exe

# 预览模式，扫描指定路径
.\target\release\disk_cleaner.exe --scan "C:\Temp"

# 预览模式，安静模式
.\target\release\disk_cleaner.exe --quiet
```

#### 实际清理

```bash
# 执行实际清理（会提示确认）
.\target\release\disk_cleaner.exe --clean

# 扫描指定路径并清理
.\target\release\disk_cleaner.exe --scan "C:\Temp" --clean

# 设置文件最大年龄
.\target\release\disk_cleaner.exe --max-age 7 --clean
```

#### 高级选项

```bash
# 添加多个扫描路径
.\target\release\disk_cleaner.exe --scan "C:\Temp" --scan "C:\Users\Username\AppData\Local\Temp" --clean

# 添加排除路径
.\target\release\disk_cleaner.exe --exclude "C:\Important" --clean

# 组合使用
.\target\release\disk_cleaner.exe --scan "C:\Temp" --exclude "C:\Important" --max-age 7 --clean
```

### 命令行选项

| 选项 | 说明 |
|-------|------|
| `--clean` | 执行实际清理（默认为预览模式） |
| `--quiet` | 安静模式，不输出详细信息 |
| `--scan <路径>` | 添加扫描路径 |
| `--exclude <路径>` | 添加排除路径 |
| `--max-age <天数>` | 设置文件最大年龄（天） |
| `--help` | 显示帮助信息 |

## 清理策略

### 文件类型分类

| 文件类型 | 扩展名 | 清理策略 | 风险等级 |
|---------|---------|----------|---------|
| 临时文件 | *.tmp, *.temp | 清理7天前的文件 | 低 |
| 缓存文件 | *.cache, *.cach | 清理30天前的文件 | 低 |
| 日志文件 | *.log | 清理30天前的文件 | 中 |
| 更新备份 | *.old, *.bak | 清理90天前的文件 | 中 |
| 浏览器缓存 | Cache | 清理30天前的文件 | 低 |
| 系统临时文件 | ~* | 清理7天前的文件 | 低 |

### 默认排除路径

- Windows系统目录
- Program Files
- Program Files (x86)
- ProgramData
- $Recycle.Bin（回收站）
- System Volume Information

## 项目结构

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

## 核心模块

### scanner.rs - 文件扫描模块

负责扫描指定目录中的文件，识别无用文件。

### cleaner.rs - 文件清理模块

负责清理扫描到的无用文件。

### rules.rs - 清理规则模块

定义清理规则和排除路径。

### config.rs - 配置管理模块

管理配置文件的读取和写入。

### utils.rs - 工具函数模块

提供格式化文件大小、文件类型、时间戳等工具函数。

### report.rs - 报告生成模块

生成详细的清理报告。

## 安全注意事项

### 风险评估

| 文件类型 | 风险等级 | 说明 |
|---------|---------|------|
| 临时文件 | 低 | 通常可以安全删除 |
| 缓存文件 | 低 | 删除后程序会重新生成 |
| 日志文件 | 中 | 删除后影响问题排查 |
| 更新备份 | 中 | 删除后无法回滚 |
| 系统文件 | 高 | 不要删除系统文件 |
| 用户数据 | 高 | 不要删除用户数据 |

### 最佳实践

1. **预览模式**：首次使用建议使用预览模式，查看将要清理的文件
2. **确认清理**：实际清理前仔细确认将要清理的文件列表
3. **系统备份**：清理前建议备份重要数据
4. **定期清理**：建议每月清理一次

## 性能优化

- 使用多线程并行扫描，提高扫描速度
- 支持增量扫描，避免重复扫描
- 批量删除文件，提高清理效率

## 常见问题

### 清理后系统变慢

清理缓存文件后，程序需要重新生成缓存，可能会暂时变慢。等待一段时间，系统会自动恢复正常。

### 清理后程序无法启动

检查是否清理了该程序的配置文件。重新安装该程序或从备份中恢复配置文件。

### 扫描速度慢

减少扫描路径，使用多线程扫描，排除不需要扫描的目录。

## 许可证

MIT License

## 作者

aspnmy <support@e2bank.cn>

## 致谢

感谢所有为本项目做出贡献的开发者。
