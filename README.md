# GmemWorker 项目

一个简单的 AI 开发边界框架，通过全局记忆管理系统，显著消除 AI 助手的幻觉、关注偏移、上下文遗忘等问题，让 AI 开发更高效、更可靠。

## 🎯 项目定位

**GmemWorker** 是一个专为 AI 助手设计的开发边界框架，通过 `GmemoryStore` 全局记忆工具，实现：

- **全局记忆**：存储跨项目的通用规则、编码规范和最佳实践
- **项目独有记忆**：记录特定项目的需求、架构和约定
- **AI 开发指导**：为 AI 助手提供准确的上下文和规范指导
- **开发效率提升**：显著减少 AI 幻觉，提高代码质量和开发速度

## 📁 项目结构

```
GmemWorker/
├── GmemWorker/           # 主工具目录
│   └── bin/              # 可执行文件目录
├── AppProjects/          # 应用项目目录（存放通过本工具开发的实例）
│   ├── gmem_rust_memory_store/  # 全局记忆存储系统
│   └── rust_disk_cleaner/    # C盘清理工具
├── docs/                 # 文档目录
├── ralph/                # 根项目调试目录
└── README.md             # 项目说明文档
```

## 🚀 核心功能

### 1. GmemoryStore 全局记忆存储系统

**消除 AI 幻觉的核心武器**
- **持久化存储**：所有记忆保存在本地 JSON 文件中，支持跨会话访问
- **双模式记忆**：支持全局记忆和项目独有记忆两种模式
- **智能检索**：基于关键词和标签的快速搜索功能
- **记忆注入**：可将相关记忆注入到当前对话上下文中
- **版本控制**：支持记忆的软删除和回滚机制
- **确定性压缩**：将相关记忆压缩为预算约束的 markdown 块
- **MCP 服务器**：提供 Model Context Protocol 服务器，方便 AI 工具助手调用

### 2. AI 开发边界管理

**为 AI 助手设定清晰的开发边界**
- **编码规范**：存储项目特定的编码规范和最佳实践
- **架构约定**：记录重要的技术决策和架构选择
- **上下文保持**：在长期项目中保持上下文连续性
- **团队协作**：通过项目记忆共享团队规范和约定
- **开发流程**：指导 AI 助手遵循正确的开发流程

### 3. 实例项目

**展示本工具的实际应用效果**
- **rust_disk_cleaner**：C盘清理工具，展示如何通过全局记忆指导开发
- **更多实例**：后续将在 AppProjects 目录中添加更多通过本工具开发的项目

## 📦 安装使用

### 全局记忆存储系统

```bash
# 方法1：直接使用编译好的可执行文件
# 编译好的文件位于：GmemWorker/bin/GmemoryStore.exe

# 方法2：从源码编译
cd AppProjects/gmem_rust_memory_store
cargo build --release
# 编译后的文件位于：target/release/GmemoryStore.exe

# 方法3：编译 MCP 服务器
cd AppProjects/gmem_rust_memory_store
cargo build --release --bin gmemory-mcp-server --features full
# 编译后的文件位于：target/release/gmemory-mcp-server.exe
```

**基本使用**：
```bash
# 启动 CLI
GmemoryStore

# 添加全局记忆
> add --tags gmem,rules,coding "所有函数必须添加中文注释，包含参数和返回值说明"

# 添加项目记忆
> add --tags gmem,project,rust "项目使用 Rust 语言开发，优先使用本地 wsl2 环境编译"

# 搜索记忆
> search "编码规范"

# 压缩记忆（注入到 AI 上下文）
> compress "Rust 开发规范" --budget 1000
```

**MCP 服务器使用**：
```bash
# 启动 MCP 服务器（供 AI 工具助手调用）
gmemory-mcp-server

# MCP 服务器会自动读取配置文件
# 配置文件位于：GmemWorker/bin/config/.env.toml
# 可以在 AI 工具助手的配置中添加 MCP 服务器连接
```

**MCP 服务器配置示例**：

在 AI 工具助手的 MCP 配置文件中添加以下配置：

```json
{
  "mcpServers": {
    "gmem-store": {
      "command": "V:\\git_data\\GmemWorker\\GmemWorker\\bin\\gmemory-mcp-server.exe",
      "args": []
    }
  }
}
```

**配置说明**：
- `gmem-store`：MCP 服务器的名称（可自定义）
- `command`：gmemory-mcp-server.exe 的完整路径（请根据实际安装路径修改）
- `args`：启动参数（通常为空数组）

**VS Code 配置示例**：

在 VS Code 的 `settings.json` 中添加：

```json
{
  "mcp.mcpServers": {
    "gmem-store": {
      "command": "V:\\git_data\\GmemWorker\\GmemWorker\\bin\\gmemory-mcp-server.exe",
      "args": []
    }
  }
}
```

**Cursor 配置示例**：

在 Cursor 的配置文件中添加：

```json
{
  "mcpServers": {
    "gmem-store": {
      "command": "V:\\git_data\\GmemWorker\\GmemWorker\\bin\\gmemory-mcp-server.exe",
      "args": []
    }
  }
}
```

**注意**：
- 路径分隔符在 Windows 中需要使用双反斜杠 `\\` 或正斜杠 `/`
- 请将路径修改为实际的 gmemory-mcp-server.exe 安装路径
- 配置完成后重启 AI 工具助手即可生效

### 开发新实例项目

1. **初始化全局记忆**：添加通用开发规范
2. **创建项目记忆**：记录项目特定需求和约定
3. **指导 AI 开发**：让 AI 助手参考记忆进行开发
4. **存储实例**：将开发完成的项目放入 AppProjects 目录

## 🛠️ 核心优势

### 🔍 解决 AI 开发痛点

| **问题** | **传统 AI 开发** | **GmemWorker 解决方案** |
|---------|-----------------|------------------------|
| **幻觉** | 经常产生错误信息 | 通过记忆验证，消除幻觉 |
| **关注偏移** | 容易偏离核心需求 | 通过记忆约束，保持专注 |
| **上下文遗忘** | 长期项目记忆丢失 | 通过持久化记忆，保持上下文 |
| **规范不一致** | 代码风格混乱 | 通过记忆规范，确保一致性 |
| **重复劳动** | 相同问题反复解释 | 通过记忆存储，一次录入多次使用 |

### 📈 开发效率提升

- **减少沟通成本**：无需反复解释项目需求和规范
- **提高代码质量**：AI 助手参考准确的记忆进行开发
- **加速开发周期**：消除错误和返工，开发更顺畅
- **降低维护成本**：代码风格一致，易于理解和维护

## 📚 记忆管理最佳实践

### 全局记忆分类

- **编码规范**：语言特定的编码标准和最佳实践
- **开发流程**：项目管理和开发工作流
- **工具配置**：IDE、编译器和构建工具的配置
- **架构原则**：通用的软件架构设计原则
- **安全规范**：代码安全和最佳实践

### 项目记忆分类

- **项目需求**：详细的功能需求和验收标准
- **技术栈**：项目使用的技术框架和依赖
- **架构设计**：系统架构和模块划分
- **API 约定**：接口设计和命名规范
- **部署流程**：构建、测试和部署流程

## 🔧 常见问题

### Q: GmemWorker 如何消除 AI 幻觉？
**A:** 通过查询相关记忆并注入到 AI 上下文，让 AI 助手基于准确的信息进行开发，避免凭空猜测。

### Q: 如何管理大量的记忆？
**A:** 使用 `compress` 命令将相关记忆压缩为 markdown 块，保持上下文简洁而有效。

### Q: 记忆存储在哪里？
**A:** 全局记忆存储在系统配置的记忆文件中，项目记忆存储在项目根目录的 `.项目名.gmem.json` 文件中。

### Q: 如何更新记忆？
**A:** 采用 "添加新记忆 + 软删除旧记忆" 的方式，确保记忆的版本控制和可回滚性。

## 📞 联系方式

- **作者**：aspnmy
- **邮箱**：support@e2bank.cn
- **GitHub**：https://github.com/aspnmy

## 📄 许可证

MIT License

---

**💡 核心价值**：GmemWorker 不是一个普通的工具集合，而是一个 AI 开发的边界框架，通过记忆管理，让 AI 助手成为真正可靠的开发伙伴。

**🚀 开始使用**：首先运行 `GmemWorker/bin/GmemoryStore.exe` 添加你的第一个全局记忆，然后体验 AI 开发的革命性变化！