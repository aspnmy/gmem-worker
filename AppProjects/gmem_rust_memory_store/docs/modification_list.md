# 代码修改清单

## 已完成的功能

### 1. lock.rs - 锁文件核心功能 ✅
- [x] 定义 LockType 枚举（Interactive, Cli, Mcp）
- [x] 实现 `acquire_lock()` - 基本锁获取功能
- [x] 实现 `acquire_lock_with_cleanup()` - 带过期检查的锁获取
- [x] 实现 `get_lock_file_age()` - 获取锁文件年龄
- [x] 实现 `cleanup_expired_locks()` - 清理过期锁文件
- [x] 实现 `release_lock()` - 释放锁文件

### 2. store.rs - 多锁文件支持 ✅
- [x] MemoryStore 结构体添加 `lock_type` 字段
- [x] `new()` 方法支持 `lock_type` 参数
- [x] `resolve_lock_path()` 支持不同锁类型

### 3. lib.rs - 导出新功能 ✅
- [x] 导出 LockType
- [x] 导出 acquire_lock_with_cleanup
- [x] 导出 cleanup_expired_locks

### 4. main.rs - 根据运行模式选择锁类型 ✅
- [x] 交互模式使用 LockType::Interactive
- [x] 命令行模式使用 LockType::Cli

### 5. mcp_server.rs - MCP服务器使用Mcp锁 ✅
- [x] 使用 LockType::Mcp

### 6. md_processor.rs - MD处理器使用Cli锁 ✅
- [x] 使用 LockType::Cli

### 7. organize_memory.rs - 记忆整理使用Cli锁 ✅
- [x] 使用 LockType::Cli

### 8. read_memory.rs - 记忆读取使用Cli锁 ✅
- [x] 使用 LockType::Cli

### 9. lock_cleaner.rs - 定时清理工具 ✅
- [x] 支持单次清理模式
- [x] 支持定时清理模式
- [x] 支持自定义参数

### 10. Cargo.toml - 添加新二进制文件 ✅
- [x] 添加 lock_cleaner 二进制文件

## 需要修改的功能

### 1. store.rs - 使用带清理功能的锁获取 🔴 高优先级
**问题**：当前使用 `acquire_lock()`，不会自动清理过期锁文件
**修改**：将所有 `acquire_lock(&self.lock_path, None)` 改为 `acquire_lock_with_cleanup(&self.lock_path, None, Some(300))`

**位置**：
- 第73行：`add_memory()` 方法
- 第170行：`delete_memory()` 方法
- 第200行：`compress()` 方法
- 第249行：`import_json()` 方法

### 2. main.rs - 交互模式退出处理 🔴 高优先级
**问题**：交互模式正常退出时没有删除锁文件
**修改**：添加信号处理，在程序退出时删除锁文件

**需要添加**：
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// 在 main() 函数中
let lock_path = store.lock_path.clone();
let running = Arc::new(AtomicBool::new(true));

// 设置 Ctrl+C 处理
ctrlc::set_handler(move || {
    running.store(false, Ordering::SeqCst);
    let _ = std::fs::remove_file(&lock_path);
}).expect("Error setting Ctrl-C handler");

// 在 REPL 循环中检查 running 状态
while running.load(Ordering::SeqCst) {
    // REPL 逻辑
}
```

### 3. mcp_server.rs - MCP服务器退出处理 🔴 高优先级
**问题**：MCP服务器正常退出时没有删除锁文件
**修改**：添加信号处理，在程序退出时删除锁文件

**需要添加**：
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// 在 main() 函数中
let lock_path = store.lock_path.clone();
let running = Arc::new(AtomicBool::new(true));

// 设置 Ctrl+C 处理
ctrlc::set_handler(move || {
    running.store(false, Ordering::SeqCst);
    let _ = std::fs::remove_file(&lock_path);
}).expect("Error setting Ctrl-C handler");

// 在主循环中检查 running 状态
while running.load(Ordering::SeqCst) {
    // 服务器逻辑
}
```

### 4. 各命令行工具 - 正常退出处理 🟡 中优先级
**问题**：命令行工具正常退出时没有删除锁文件
**修改**：使用 RAII 模式，在程序退出时自动删除锁文件

**需要修改的文件**：
- src/bin/organize_once.rs
- src/bin/organize_timer.rs
- src/bin/md_import.rs
- src/bin/json_import.rs
- src/bin/txt_import.rs
- src/bin/cleanall.rs
- src/bin/remove_lock.rs
- src/bin/remove_timer_lock.rs

**修改方案**：创建一个 LockGuard 结构体，在 Drop 时自动删除锁文件

### 5. Cargo.toml - 添加 ctrlc 依赖 🟡 中优先级
**问题**：需要添加信号处理库
**修改**：在 [dependencies] 中添加 `ctrlc = "3.4"`

## 编译错误修复

### 当前编译错误
1. organize_memory.rs:36 - `add_memory()` 调用使用了3个参数，但方法只接受2个
2. md_processor.rs:89 - `add_memory()` 调用使用了3个参数，但方法只接受2个

**原因**：这些文件在之前的修改中被改了，但 store.rs 中的方法签名没有对应的 category 参数

**解决方案**：
- 方案1：恢复 organize_memory.rs 和 md_processor.rs 到原始状态（推荐）
- 方案2：在 store.rs 中添加 category 参数支持

**建议**：使用方案1，因为当前设计不需要 category 参数，分类逻辑已经在其他地方处理

## 测试计划

### 1. 单元测试
- [ ] 测试 LockType 枚举
- [ ] 测试 `get_lock_file_age()` 函数
- [ ] 测试 `cleanup_expired_locks()` 函数
- [ ] 测试 `acquire_lock_with_cleanup()` 函数

### 2. 集成测试
- [ ] 测试交互模式和命令行模式并发运行
- [ ] 测试MCP服务器与其他模式并发运行
- [ ] 测试锁文件过期自动清理
- [ ] 测试程序正常退出时删除锁文件

### 3. 手动测试
- [ ] 运行 lock_cleaner --once 清理过期锁文件
- [ ] 运行 lock_cleaner 定时清理
- [ ] 测试各种异常情况下的锁文件处理

## 优先级总结

### 🔴 高优先级（必须完成）
1. 修复编译错误
2. store.rs 使用 acquire_lock_with_cleanup
3. main.rs 添加退出处理
4. mcp_server.rs 添加退出处理

### 🟡 中优先级（应该完成）
5. 命令行工具添加退出处理
6. 添加 ctrlc 依赖
7. 创建 LockGuard 结构体

### 🟢 低优先级（可以延后）
8. 编写单元测试
9. 编写集成测试
10. 更新文档
