# 锁文件开发方案

## 设计目标

实现多锁文件机制，支持不同使用场景的并发访问，同时保证数据一致性。

## 锁文件类型

1. **交互模式锁** (`.copilot-memory.interactive.lock`)
   - 用于 GmemoryStore 交互模式
   - 程序持续运行，不退出
   - 防止多个交互模式实例同时运行

2. **命令行模式锁** (`.copilot-memory.cli.lock`)
   - 用于 GmemoryStore 命令行模式
   - 运行一次就退出
   - 防止多个命令行工具同时写入同一个记忆文件
   - 允许多个命令行工具并发运行（使用不同的锁）

3. **MCP服务器锁** (`.copilot-memory.mcp.lock`)
   - 用于 gmemory_mcp_server
   - 作为服务器持续运行
   - 防止多个MCP服务器实例同时运行
   - 防止与GmemoryStore交互模式冲突

## 核心功能

### 1. 多锁文件支持
- [x] 定义 LockType 枚举（Interactive, Cli, Mcp）
- [x] 每种锁类型使用不同的锁文件后缀
- [x] MemoryStore 支持指定锁类型
- [x] 根据运行模式自动选择锁类型

### 2. 锁文件年龄检查
- [x] 实现 `get_lock_file_age()` 函数，获取锁文件年龄
- [x] 实现 `cleanup_expired_locks()` 函数，清理过期锁文件
- [x] 实现 `acquire_lock_with_cleanup()` 函数，获取锁前自动清理过期锁
- [x] 创建 lock_cleaner.rs 定时清理工具

### 3. 正常退出处理
- [ ] 程序正常退出时删除锁文件
- [ ] 需要在各个程序中添加退出处理逻辑

### 4. 定时清理工具
- [x] 创建 lock_cleaner.rs 工具
- [x] 支持单次清理模式 (`--once`)
- [x] 支持定时清理模式（默认每5分钟）
- [x] 支持自定义清理间隔 (`--interval`)
- [x] 支持自定义锁文件最大年龄 (`--max-age`)

## 各程序锁文件使用

### GmemoryStore
- [x] 交互模式：使用 LockType::Interactive
- [x] 命令行模式：使用 LockType::Cli
- [ ] 正常退出时删除锁文件

### gmemory_mcp_server
- [x] 使用 LockType::Mcp
- [ ] 正常退出时删除锁文件

### 命令行工具
- [x] organize_once：使用 LockType::Cli
- [x] organize_timer：使用 LockType::Cli
- [x] md_import：使用 LockType::Cli
- [x] json_import：使用 LockType::Cli
- [x] txt_import：使用 LockType::Cli
- [x] cleanall：使用 LockType::Cli
- [x] remove_lock：使用 LockType::Cli
- [x] remove_timer_lock：使用 LockType::Cli
- [ ] 正常退出时删除锁文件

## 待完成功能

### 高优先级
1. **程序正常退出时删除锁文件**
   - 在 GmemoryStore 交互模式中添加退出处理
   - 在 gmemory_mcp_server 中添加退出处理
   - 在各个命令行工具中添加退出处理

2. **测试多锁文件方案**
   - 测试交互模式和命令行模式并发运行
   - 测试MCP服务器与其他模式并发运行
   - 测试锁文件过期自动清理

### 中优先级
3. **优化锁文件获取逻辑**
   - 在 MemoryStore 中使用 `acquire_lock_with_cleanup` 替代 `acquire_lock`
   - 这样可以在获取锁时自动清理过期锁文件

4. **文档更新**
   - 更新 README.md，说明多锁文件机制
   - 添加 lock_cleaner 工具使用说明

## 配置说明

### 锁文件最大年龄
- 默认：300秒（5分钟）
- 可通过 `--max-age` 参数自定义

### 清理间隔
- 默认：5分钟
- 可通过 `--interval` 参数自定义

## 注意事项

1. **锁文件位置**
   - 所有锁文件都放在记忆文件所在目录
   - 例如：`E:\GmemWorkerHome\.copilot-memory.interactive.lock`

2. **锁文件内容**
   - 格式：`进程ID 时间戳`
   - 用于调试和追踪

3. **并发控制**
   - 交互模式和MCP服务器互斥（不能同时运行）
   - 命令行工具可以与交互模式并发运行（使用不同的锁）
   - 多个命令行工具可以并发运行（使用相同的锁，串行执行）

4. **异常处理**
   - 程序异常退出时，锁文件可能残留
   - 使用 lock_cleaner 定时清理过期锁文件
   - 或手动运行 `lock_cleaner --once` 清理
