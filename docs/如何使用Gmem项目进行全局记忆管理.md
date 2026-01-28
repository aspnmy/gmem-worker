# 【浅谈】如何让AI辅助编辑代码能够自我进化自我更新(二)——Gmem项目进行全局记忆管理

项目路径：

https://github.com/aspnmy/copilot-memory-store.git

如果要完整体验此框架需要git仓库到本地

```bash
git clone https://github.com/aspnmy/copilot-memory-store.git
```

## 一、Gmem项目简介

Gmem（Global Memory）是一个强大的本地JSON记忆存储系统，专为AI助手设计，支持持久化保存和检索重要信息、偏好设置、决策和上下文。该系统通过CLI命令行工具和MCP服务器，为GitHub Copilot等AI助手提供了跨会话、跨项目的记忆能力。

### 1.1 核心特性

- **持久化存储**：所有记忆保存在本地JSON文件中，支持跨会话访问
- **双模式记忆**：支持全局记忆和项目独有记忆两种模式
- **智能检索**：基于关键词和标签的快速搜索功能
- **记忆注入**：可将相关记忆注入到当前对话上下文中
- **版本控制**：支持记忆的软删除和回滚机制
- **自动备份**：支持定期自动备份记忆数据

### 1.2 应用场景

- **开发规范管理**：保存项目特定的编码规范和最佳实践
- **偏好设置**：记录个人的开发偏好和工具配置
- **决策记录**：保存重要的技术决策和架构选择
- **上下文保持**：在长期项目中保持上下文连续性
- **团队协作**：通过项目记忆共享团队规范和约定

## 二、Gmem项目架构

### 2.1 项目结构

```
copilot-memory-store/
├── bin/                          # 可执行文件
│   ├── copilot-memory-store.bat  # CLI工具启动脚本
│   ├── copilot-memory-mcp.bat    # MCP服务器启动脚本
│   └── copilot-memory-trae.bat   # Trae MCP服务器启动脚本
├── dist/                         # 编译后的JavaScript文件
│   ├── cli.js                    # CLI命令行工具
│   ├── mcp-server.js             # MCP服务器
│   ├── trae-mcp-server.js        # Trae MCP服务器
│   ├── memoryStore.js            # 记忆存储核心模块
│   └── ...
├── src/                          # TypeScript源代码
│   ├── cli.ts                    # CLI命令行工具源码
│   ├── memoryStore.ts            # 记忆存储核心模块源码
│   ├── config.ts                 # 配置管理模块
│   └── ...
├── docs/                         # 文档目录
│   ├── CLI_GUIDE.md              # CLI使用指南
│   ├── CONTEXT_MEMORY_TYPES.md   # 记忆类型说明
│   └── ...
├── examples/                     # 示例文件
│   └── scenarios/                # 场景示例
├── .copilot-memory-store.gmem.json  # 项目独有记忆文件
├── project-memory.json           # 全局记忆文件
├── package.json                  # Node.js项目配置
└── tsconfig.json                 # TypeScript配置
```

### 2.2 核心模块说明

#### 2.2.1 memoryStore.ts - 记忆存储核心

负责记忆的增删改查操作，支持：
- 记忆的添加、搜索、删除、软删除
- 基于关键词和标签的检索
- 记忆的压缩和合并
- 记忆的导入导出

#### 2.2.2 cli.ts - 命令行工具

提供完整的CLI接口，支持：
- `add`：添加新记忆
- `search`：搜索记忆
- `compress`：压缩记忆
- `delete`：删除记忆
- `purge`：批量删除
- `export`：导出记忆
- `stats`：统计信息

#### 2.2.3 mcp-server.ts - MCP服务器

实现Model Context Protocol服务器，为AI助手提供标准化的记忆访问接口。

## 三、Gmem安装与配置

### 3.1 环境要求

- Node.js >= 16.x
- npm 或 yarn
- TypeScript（开发环境）

### 3.2 安装步骤

#### 3.2.1 克隆项目

```bash
git clone https://github.com/aspnmy/copilot-memory-store.git
cd copilot-memory-store
```

#### 3.2.2 安装依赖

```bash
npm install
```

#### 3.2.3 编译项目

```bash
npm run build
```

#### 3.2.4 配置环境变量

复制 `.env.example` 为 `.env` 并配置：

```bash
# 记忆文件路径（相对或绝对路径）
MEMORY_PATH=project-memory.json

# 项目名称
PROJECT_NAME=My Project

# DeepSeek API密钥（可选，用于LLM压缩）
DEEPSEEK_API_KEY=

# 自动备份配置
BACKUP_INTERVAL=3600000
BACKUP_FORMAT=json
BACKUP_DIR=./backups
MAX_BACKUPS=10
COMPRESS_BACKUPS=false
```

### 3.3 验证安装

```bash
# 查看帮助
node dist/cli.js help

# 查看统计信息
node dist/cli.js stats
```

## 四、Gmem基本使用

### 4.1 新增记忆

#### 4.1.1 基本语法

```bash
node dist/cli.js add "要保存的内容"
```

#### 4.1.2 带标签的记忆

```bash
node dist/cli.js add "使用TypeScript严格模式开发" --tags gmem,preference,typescript
```

#### 4.1.3 实际示例

```bash
# 添加编码规范
node dist/cli.js add "所有函数必须添加注释，注释中需要包含函数的参数和返回值" --tags gmem,rules,coding

# 添加项目配置
node dist/cli.js add "项目使用Rust语言开发，优先使用本地wsl2环境编译和运行" --tags gmem,rust,development

# 添加偏好设置
node dist/cli.js add "使用2空格缩进，不使用分号" --tags gmem,preference,format
```

### 4.2 查询记忆

#### 4.2.1 基本搜索

```bash
node dist/cli.js search "关键词"
```

#### 4.2.2 限制结果数量

```bash
node dist/cli.js search "rust" --limit 5
```

#### 4.2.3 原始输出格式

```bash
node dist/cli.js search "开发" --raw
```

#### 4.2.4 实际示例

```bash
# 搜索编码规范
node dist/cli.js search "注释"

# 搜索Rust相关记忆
node dist/cli.js search "rust"

# 搜索开发规则
node dist/cli.js search "规则" --limit 10
```

### 4.3 记忆压缩

当记忆数量较多时，可以使用压缩功能将相关记忆合并：

```bash
node dist/cli.js compress --query "开发规范" --budget 5 --limit 10
```

参数说明：
- `--query`：压缩的关键词
- `--budget`：压缩后的记忆数量
- `--limit`：参与压缩的记忆数量
- `--llm`：使用LLM进行智能压缩（需要配置DEEPSEEK_API_KEY）

### 4.4 删除记忆

#### 4.4.1 软删除（推荐）

```bash
node dist/cli.js delete <记忆ID>
```

软删除的记忆会被标记为已删除，但不会真正删除，可以回滚。

#### 4.4.2 批量删除

```bash
# 按ID删除
node dist/cli.js purge --id <记忆ID>

# 按匹配内容删除
node dist/cli.js purge --match "关键词"

# 按标签删除
node dist/cli.js purge --tag "标签名"

# 预览删除（不实际执行）
node dist/cli.js purge --match "关键词" --dry-run
```

### 4.5 导出记忆

```bash
node dist/cli.js export
```

导出所有记忆到JSON文件，便于备份和迁移。

### 4.6 查看统计信息

```bash
node dist/cli.js stats
```

显示记忆总数、活跃记忆数、已删除记忆数和热门标签统计。

## 五、规则管理与合并

### 5.1 实战一个Gmem合并外部项目规则到全局记忆
- 现在我们来实战将一个AI编程规范文件加入全局记忆
- 假设我们有一个AI编程规范文件项目 `https://github.com/aspnmy/ai_project_rules.git`，我们需要将其加入全局记忆
- 我们可以使用以下命令将文件内容加入全局记忆：

```AI聊天框
我说：总结https://github.com/aspnmy/ai_project_rules.git项目中所有有效规则，加入全局记忆并进行整理优化，合并重复规则为一条规则，每个规则之间用换行符隔开，软删除过期规则便于回滚和历史管理，优化冲突的规则，确保所有规则都符合项目需求
AI：现在开始读取项目规则，我找到了哪些规则，现在开始写入全局记忆，…… 全局记忆写入完成，现在开始合并规则，…… 规则合并完成，全局记忆更新完成……

```

### 5.2 实战如何在新项目中使用Gmem项目进行全局记忆管理

- 首先在新项目中，很关键的需要将gmem项目中.env文件中的记忆文件路径MEMORY_PATH加入到系统全局变量中，这样新建的项目AI助手可以自动搜索全局变量获得统一记忆文件。

- 其次Gmem的业务逻辑，一台机器上最好只使用一个版本的gmem项目，否则会导致记忆文件冲突，目前的版本号v0.3.0。

- Gmem项目升级，先发送指令给AI，导出所有记忆到文件，然后再通过git pull进行版本升级，最后再导入记忆文件。避免大版本号的升级造成的记忆文件错误。导入记忆文件的时候直接让AI助手分析记忆文件再导入，不要直接覆盖记忆文件，可能由于跨版本的问题造成解析异常

```bash
export MEMORY_PATH=project-memory.json
```


### 5.3 规则分类体系

Gmem支持多层次的规则管理，按照优先级分为：

#### 5.1.1 最高优先级
- IDE语法错误和编译错误必须立即解决

#### 5.1.2 高优先级（🔴）
- Git工作流规则
- 临时文件管理规则
- 文件路径使用规则
- 文本版本控制规则
- IDE规范规则

#### 5.1.3 中优先级（🟡）
- 目录结构整理规则
- 配置管理规则
- 字符编码规范
- 文件名命名规范规则
- WSL2纯净开发环境规则
- autotest规则

#### 5.1.4 低优先级（🟢）
- 规则优化建议
- 代码风格警告
- 静态分析建议

### 5.2 新增规则

#### 5.2.1 添加高优先级规则

```bash
node dist/cli.js add "临时文件管理规则(🔴高优先级)：临时文件必须使用temp-前缀，严禁将临时文件提交到版本控制，调试完成后必须清理临时文件" --tags gmem,rules,files,high
```

#### 5.2.2 添加中优先级规则

```bash
node dist/cli.js add "目录结构整理规则(🟡中优先级)：根目录只存放主规则文件和必要的配置文件，rules目录存放次级规则文件，Util目录存放功能脚本，Logs目录存放日志文件，docs目录存放文档类文件" --tags gmem,rules,directory,medium
```

#### 5.2.3 添加开发规范

```bash
node dist/cli.js add "所有的函数都需要添加注释，注释中需要包含函数的参数和返回值。所有的注释都需要使用Markdown格式或者中文表示，不能使用其他格式。所有的注释都需要在函数定义之前添加，不能在函数定义之后添加" --tags gmem,rules,coding,comments
```

### 5.3 合并规则

当存在重复或相似的规则时，可以进行合并：

#### 5.3.1 识别重复规则

```bash
node dist/cli.js search "Git" --limit 20
```

#### 5.3.2 合并示例

假设有以下两条规则：

规则1：
```bash
node dist/cli.js add "Git提交规则：所有代码修改必须先提交到git仓库" --tags gmem,rules,git
```

规则2：
```bash
node dist/cli.js add "Git分支规则：平台适配需要新建dev-platform分支，严禁直接修改master/main分支" --tags gmem,rules,git
```

合并为一条完整规则：

```bash
node dist/cli.js add "Git工作流规则(🔴高优先级)：1)代码修改-所有代码修改必须先提交到git仓库，平台适配需要新建dev-platform分支，严禁直接修改master/main分支；2)提交前检查-同步代码到远程仓库前必须先按照规则文件中的语法规范对代码进行检查，如果IDE报错以IDE优化建议为准进行修正；3)仓库配置-用户没有输入同步私有库指令前默认只提交到github，默认新发布的仓库为私有仓库；4)分支管理-如果只是简单修改可以直接在master分支上修改，如果需要修改的分支是master或main分支需要先把本地提交新建一个分支然后从远程拉取master或main分支最后个版本代码再以主分支建立dev-platform分支进行修改" --tags gmem,rules,git,high
```

#### 5.3.3 软删除旧规则

```bash
node dist/cli.js delete <规则1的ID>
node dist/cli.js delete <规则2的ID>
```

### 5.4 更新规则

#### 5.4.1 查找规则

```bash
node dist/cli.js search "规则关键词"
```

#### 5.4.2 更新规则流程

1. 添加新规则（更新后的版本）
2. 软删除旧规则
3. 验证新规则生效

示例：

```bash
# 1. 添加更新后的规则
node dist/cli.js add "IDE规范规则(🔴高优先级)：1)语法错误处理-语法错误必须立即修复优先级高于所有其他规则；2)编译错误处理-编译错误必须解决优先级高于代码规范；3)代码风格警告-代码风格警告遵循代码书写格式规则；4)静态分析建议-静态分析建议参考使用；5)IDE规范优先-如果编写代码中IDE提示规范高于目前语法规范按IDE要求修改代码规范；6)版本管理-修改语法规范时需要标明对应版本号确保语法规范的版本兼容性" --tags gmem,rules,ide,high

# 2. 软删除旧规则
node dist/cli.js delete <旧规则的ID>

# 3. 验证新规则
node dist/cli.js search "IDE规范"
```

### 5.5 规则优先级冲突解决

当规则之间出现冲突时，按照以下原则解决：

1. **高优先级覆盖低优先级**：🔴高优先级 > 🟡中优先级 > 🟢低优先级
2. **具体规则覆盖通用规则**：针对特定场景的规则优先于通用规则
3. **IDE错误优先**：IDE错误 > 代码规范 > 其他规则

## 六、生成项目

### 6.1 项目记忆管理

Gmem支持两种记忆模式：全局记忆和项目独有记忆。

#### 6.1.1 全局记忆

适用于所有项目的通用规则和偏好设置。

存储位置：`project-memory.json`（在Gmem项目根目录）

示例：
```bash
node dist/cli.js add "所有的开发项目，使用时区都以shanghai为准" --tags gmem,preference,timezone
```

#### 6.1.2 项目独有记忆

仅适用于当前项目的特定规则和配置。

存储位置：`.项目名.gmem.json`（在项目根目录）

创建项目记忆文件：

```bash
# 在项目根目录创建 .项目名.gmem.json
echo '{"projectName":"my-project","memories":[]}' > .my-project.gmem.json
```

添加项目记忆：

```bash
node dist/cli.js add --MEMORY_PATH=.my-project.gmem.json "这个项目使用TypeScript + React技术栈" --tags gmem,architecture,typescript,react
```

### 6.2 项目初始化流程

#### 6.2.1 创建新项目

```bash
# 1. 创建项目目录
mkdir my-new-project
cd my-new-project

# 2. 初始化项目
git init
npm init -y

# 3. 创建项目记忆文件
echo '{"projectName":"my-new-project","memories":[]}' > .my-new-project.gmem.json

# 4. 添加项目特定记忆
node dist/cli.js add --MEMORY_PATH=.my-new-project.gmem.json "项目使用Rust语言开发" --tags gmem,rust,development

# 5. 添加项目规范
node dist/cli.js add --MEMORY_PATH=.my-new-project.gmem.json "所有函数都需要添加中文注释" --tags gmem,rules,coding,comments
```

#### 6.2.2 接管已有项目

```bash
# 1. 进入项目目录
cd existing-project

# 2. 检查是否已有项目记忆文件
ls -la | grep ".gmem.json"

# 3. 如果没有，创建项目记忆文件
echo '{"projectName":"existing-project","memories":[]}' > .existing-project.gmem.json

# 4. 添加项目记忆
node dist/cli.js add --MEMORY_PATH=.existing-project.gmem.json "项目使用Python + Django技术栈" --tags gmem,architecture,python,django
```

### 6.3 项目记忆使用场景

#### 6.3.1 技术栈记录

```bash
node dist/cli.js add --MEMORY_PATH=.project.gmem.json "项目使用Rust + Actix-web + PostgreSQL技术栈" --tags gmem,architecture,rust,actix,postgres
```

#### 6.3.2 编码规范

```bash
node dist/cli.js add --MEMORY_PATH=.project.gmem.json "所有Rust函数必须添加中文注释，注释包含函数名、参数说明、返回值说明和功能描述" --tags gmem,rules,rust,coding,comments
```

#### 6.3.3 项目约定

```bash
node dist/cli.js add --MEMORY_PATH=.project.gmem.json "API端点使用/api/v1/前缀，所有API返回JSON格式" --tags gmem,rules,api,architecture
```

#### 6.3.4 开发流程

```bash
node dist/cli.js add --MEMORY_PATH=.project.gmem.json "开发流程：1)创建feature分支；2)开发功能；3)运行测试；4)提交代码；5)创建PR；6)代码审查；7)合并到main分支" --tags gmem,workflow,development
```

## 七、书写文章

### 7.1 使用Gmem辅助写作

Gmem可以辅助记录写作规范和文章模板，提高写作效率。

#### 7.1.1 记录写作规范

```bash
node dist/cli.js add "CSDN博客写作规范：1)文章标题使用【】括号标注类型，如【实战】、【浅谈】、【教程】；2)文章必须包含TOC目录；3)代码块必须指定语言类型；4)重要概念使用粗体标注；5)文章结尾包含总结和参考链接" --tags gmem,writing,csdn,blog
```

#### 7.1.2 记录文章模板

```bash
node dist/cli.js add "CSDN博客文章模板：# 标题\n\n项目路径：\n\nhttps://github.com/xxx/xxx.git\n\n## 一、简介\n\n## 二、架构\n\n## 三、安装\n\n## 四、使用\n\n## 五、总结\n\n## 参考链接" --tags gmem,writing,template,csdn
```

#### 7.1.3 记录Markdown语法

```bash
node dist/cli.js add "Markdown常用语法：# 一级标题、## 二级标题、**粗体**、*斜体*、`代码`、```代码块```、[链接](url)、![图片](url)、- 列表、1. 有序列表、> 引用" --tags gmem,writing,markdown
```

### 7.2 文章写作流程

#### 7.2.1 准备阶段

1. **查询相关记忆**

```bash
node dist/cli.js search "CSDN"
```

2. **注入记忆到上下文**

在AI聊天窗口中输入：
```
记忆注入：CSDN博客写作规范
```

3. **开始写作**

根据注入的规范和模板开始写作。

#### 7.2.2 写作阶段

1. **创建文章文件**

```bash
# 在docs目录下创建文章
mkdir -p docs
touch docs/如何使用Gmem进行全局记忆管理.md
```

2. **按照模板写作**

参考注入的模板和规范进行写作。

3. **保存写作习惯**

```bash
node dist/cli.js add "写作习惯：先写大纲，再填充内容，最后检查格式和语法" --tags gmem,writing,habit
```

### 7.3 文章示例

#### 7.3.1 技术文章

```bash
node dist/cli.js add "技术文章写作要点：1)技术文章需要包含项目路径和git clone命令；2)需要详细说明技术原理和实现细节；3)需要提供完整的代码示例；4)需要包含实际运行结果截图；5)需要总结技术要点和注意事项" --tags gmem,writing,technical
```

#### 7.3.2 教程文章

```bash
node dist/cli.js add "教程文章写作要点：1)教程文章需要从零开始，适合初学者；2)每个步骤都需要详细说明；3)需要包含常见问题和解决方案；4)需要提供完整的代码和配置文件；5)需要包含验证步骤和预期结果" --tags gmem,writing,tutorial
```

#### 7.3.3 实战文章

```bash
node dist/cli.js add "实战文章写作要点：1)实战文章需要基于真实项目；2)需要包含完整的开发流程；3)需要展示实际遇到的问题和解决方案；4)需要包含性能优化和最佳实践；5)需要总结经验和教训" --tags gmem,writing,practice
```

## 八、高级功能

### 8.1 记忆压缩

当记忆数量较多时，可以使用压缩功能将相关记忆合并：

```bash
# 使用关键词压缩
node dist/cli.js compress --query "开发规范" --budget 5 --limit 10

# 使用LLM智能压缩
node dist/cli.js compress --query "编码规范" --budget 3 --limit 8 --llm
```

### 8.2 自动备份

配置自动备份功能，定期备份记忆数据：

在 `.env` 文件中配置：

```bash
BACKUP_INTERVAL=3600000
BACKUP_FORMAT=json
BACKUP_DIR=./backups
MAX_BACKUPS=10
COMPRESS_BACKUPS=false
```

手动触发备份：

```bash
npm run backup:once
```

### 8.3 记忆导入导出

#### 8.3.1 导出记忆

```bash
# 导出为JSON
npm run memory:export

# 导出为Markdown
npm run memory:export:md

# 导出为CSV
npm run memory:export:csv

# 导出为文本
npm run memory:export:text

# 导出为嵌入向量
npm run memory:export:embedding
```

#### 8.3.2 导入记忆

```bash
npm run memory:import
```

### 8.4 MCP服务器集成

Gmem提供MCP服务器，可以与支持MCP的AI助手集成。

#### 8.4.1 启动MCP服务器

```bash
# 启动标准MCP服务器
npm run mcp

# 启动Trae MCP服务器
npm run trae
```

#### 8.4.2 配置MCP客户端

在MCP客户端配置文件中添加：

```json
{
  "mcpServers": {
    "trae-memory-store": {
      "command": "V:\\git_data\\copilot-memory-store\\bin\\copilot-memory-trae.bat",
      "args": []
    }
  }
}
```

## 九、最佳实践

### 9.1 记忆命名规范

- **明确性**：记忆内容要清晰明确，便于后续搜索
- **标签使用**：合理使用标签可以提高检索效率
- **关键词选择**：选择容易记忆和搜索的关键词

示例：

```bash
# 好的记忆
node dist/cli.js add "Rust函数注释规范：所有函数必须添加中文注释，包含函数名、参数说明、返回值说明和功能描述" --tags gmem,rules,rust,coding,comments

# 不好的记忆
node dist/cli.js add "注释规范" --tags gmem,rules
```

### 9.2 标签系统

使用标签来组织和分类记忆：

- `gmem`：Gmem记忆系统的默认标签
- `preference`：偏好设置
- `decision`：决策记录
- `fact`：事实信息
- `architecture`：架构设计
- `development`：开发相关
- `rules`：规则规范
- `high`：高优先级
- `medium`：中优先级
- `low`：低优先级

示例：

```bash
node dist/cli.js add "使用TypeScript严格模式" --tags gmem,preference,typescript,high
node dist/cli.js add "项目采用微服务架构" --tags gmem,decision,architecture,medium
```

### 9.3 记忆维护

- **定期查看**：定期查看记忆列表，删除过时的记忆
- **合并重复**：合并重复或相似的记忆
- **更新过时**：更新过时的记忆内容
- **备份重要**：定期备份重要的记忆数据

```bash
# 查看统计信息
node dist/cli.js stats

# 搜索特定标签
node dist/cli.js search "标签:high"

# 导出备份
node dist/cli.js export
```

### 9.4 工作流程

#### 9.4.1 项目启动流程

```
1. 读取全局记忆
   ↓
2. 检查项目记忆文件
   ↓
3. 如果没有，创建项目记忆文件
   ↓
4. 添加项目特定记忆
   ↓
5. 开始开发
```

#### 9.4.2 开发流程

```
1. 查询相关记忆
   ↓
2. 注入记忆到上下文
   ↓
3. 按照规范开发
   ↓
4. 添加新的记忆
   ↓
5. 更新规则
```

#### 9.4.3 文章写作流程

```
1. 查询写作规范
   ↓
2. 注入记忆到上下文
   ↓
3. 按照模板写作
   ↓
4. 检查格式和语法
   ↓
5. 保存文章
```

## 十、常见问题

### 10.1 记忆未保存

**问题**：添加记忆后查询不到

**解决**：
- 检查配置路径是否正确
- 确认有写入权限
- 查看错误日志
- 检查是否使用了正确的MEMORY_PATH参数

### 10.2 搜索不到记忆

**问题**：搜索关键词找不到相关记忆

**解决**：
- 确认记忆已成功保存
- 尝试使用不同的关键词
- 检查标签是否正确
- 使用 `--limit` 参数增加结果数量

### 10.3 记忆注入失败

**问题**：记忆注入到上下文失败

**解决**：
- 确认查询关键词正确
- 检查记忆是否存在
- 尝试增加 limit 参数
- 检查MCP服务器是否正常运行

### 10.4 规则冲突

**问题**：不同规则之间出现冲突

**解决**：
- 按照优先级解决：高优先级 > 中优先级 > 低优先级
- 具体规则覆盖通用规则
- IDE错误优先于代码规范
- 合并或更新冲突的规则

### 10.5 性能问题

**问题**：记忆数量较多时性能下降

**解决**：
- 使用压缩功能合并相关记忆
- 定期清理过时的记忆
- 使用软删除而不是硬删除
- 考虑使用嵌入向量进行检索

## 十一、总结

Gmem是一个强大的全局记忆管理系统，通过持久化存储、智能检索、记忆注入等功能，为AI助手提供了跨会话、跨项目的记忆能力。

### 11.1 核心优势

1. **持久化存储**：所有记忆保存在本地JSON文件中，支持跨会话访问
2. **双模式记忆**：支持全局记忆和项目独有记忆两种模式
3. **智能检索**：基于关键词和标签的快速搜索功能
4. **记忆注入**：可将相关记忆注入到当前对话上下文中
5. **版本控制**：支持记忆的软删除和回滚机制
6. **自动备份**：支持定期自动备份记忆数据

### 11.2 应用场景

1. **开发规范管理**：保存项目特定的编码规范和最佳实践
2. **偏好设置**：记录个人的开发偏好和工具配置
3. **决策记录**：保存重要的技术决策和架构选择
4. **上下文保持**：在长期项目中保持上下文连续性
5. **团队协作**：通过项目记忆共享团队规范和约定
6. **文章写作**：记录写作规范和模板，提高写作效率

### 11.3 最佳实践

1. **明确性**：记忆内容要清晰明确，便于后续搜索
2. **标签使用**：合理使用标签可以提高检索效率
3. **定期维护**：定期查看记忆列表，删除过时的记忆
4. **备份重要**：定期备份重要的记忆数据
5. **遵循流程**：按照工作流程使用Gmem，提高效率

### 11.4 未来展望

Gmem项目仍在持续发展中，未来计划添加以下功能：

1. **AI增强**：集成更多AI功能，如自动分类、智能推荐
2. **协作功能**：支持团队协作和共享记忆
3. **可视化界面**：提供Web界面，方便管理和查看记忆
4. **更多导出格式**：支持更多导出格式，如PDF、Word等
5. **云同步**：支持云同步，跨设备访问记忆

通过使用Gmem，我们可以让AI助手更好地理解我们的需求和偏好，提高开发效率和写作质量，成为我们的得力助手！

## 参考链接

- Gmem项目地址：https://github.com/aspnmy/copilot-memory-store
- Gmem使用指南：https://github.com/aspnmy/copilot-memory-store/blob/main/Gmem使用指南.md
- CLI使用指南：https://github.com/aspnmy/copilot-memory-store/blob/main/docs/CLI_GUIDE.md
- 记忆类型说明：https://github.com/aspnmy/copilot-memory-store/blob/main/docs/CONTEXT_MEMORY_TYPES.md

## 相关代码

### 示例1：添加编码规范

```bash
node dist/cli.js add "所有的函数都需要添加注释，注释中需要包含函数的参数和返回值。所有的注释都需要使用Markdown格式或者中文表示，不能使用其他格式。所有的注释都需要在函数定义之前添加，不能在函数定义之后添加" --tags gmem,rules,coding,comments
```

### 示例2：搜索记忆

```bash
node dist/cli.js search "编码规范" --limit 10
```

### 示例3：压缩记忆

```bash
node dist/cli.js compress --query "开发规范" --budget 5 --limit 10
```

### 示例4：导出记忆

```bash
node dist/cli.js export
```

## 附录

### A. 完整命令列表

```bash
# 添加记忆
node dist/cli.js add "内容" --tags 标签1,标签2

# 搜索记忆
node dist/cli.js search "关键词" --limit N --raw

# 压缩记忆
node dist/cli.js compress --query "关键词" --budget N --limit N --llm

# 删除记忆
node dist/cli.js delete <记忆ID>

# 批量删除
node dist/cli.js purge --id <记忆ID>
node dist/cli.js purge --match "关键词"
node dist/cli.js purge --tag "标签名"
node dist/cli.js purge --match "关键词" --dry-run

# 导出记忆
node dist/cli.js export

# 查看统计
node dist/cli.js stats

# 查看帮助
node dist/cli.js help
```

### B. 标签系统

| 标签 | 说明 | 示例 |
|------|------|------|
| gmem | Gmem记忆系统的默认标签 | 所有记忆 |
| preference | 偏好设置 | 时区、编码风格 |
| decision | 决策记录 | 技术选型、架构决策 |
| fact | 事实信息 | 项目信息、环境配置 |
| architecture | 架构设计 | 技术栈、系统架构 |
| development | 开发相关 | 开发流程、工具配置 |
| rules | 规则规范 | 编码规范、Git规范 |
| high | 高优先级 | 重要规则 |
| medium | 中优先级 | 一般规则 |
| low | 低优先级 | 参考建议 |

### C. 配置文件说明

#### .env 配置文件

```bash
# 记忆文件路径
MEMORY_PATH=project-memory.json

# 项目名称
PROJECT_NAME=My Project

# DeepSeek API密钥
DEEPSEEK_API_KEY=

# 自动备份配置
BACKUP_INTERVAL=3600000
BACKUP_FORMAT=json
BACKUP_DIR=./backups
MAX_BACKUPS=10
COMPRESS_BACKUPS=false
```

### D. 项目记忆文件格式

```json
{
  "projectName": "my-project",
  "memories": [
    {
      "id": "m_20260127T131000000Z_project",
      "text": "项目使用TypeScript + React技术栈",
      "tags": ["typescript", "react", "tech-stack"],
      "keywords": ["ts", "typescript", "react"],
      "createdAt": "2026-01-27T13:10:00.000+08:00",
      "updatedAt": "2026-01-27T13:10:00.000+08:00"
    }
  ]
}
```

## 常见问题

### Q1：Gmem和传统的笔记工具有什么区别？

A：Gmem专为AI助手设计，支持记忆注入到AI对话上下文中，并且支持MCP协议，可以与支持MCP的AI助手无缝集成。同时，Gmem支持双模式记忆（全局记忆和项目独有记忆），更适合开发场景。

### Q2：如何备份和迁移记忆数据？

A：Gmem提供了导出功能，可以将记忆导出为JSON、Markdown、CSV等多种格式。同时，Gmem支持自动备份功能，可以定期备份记忆数据。

### Q3：Gmem支持哪些AI助手？

A：Gmem支持所有支持MCP协议的AI助手，包括GitHub Copilot、Claude、Trae等。通过MCP服务器，AI助手可以访问和操作Gmem中的记忆。

### Q4：如何处理记忆冲突？

A：Gmem按照优先级处理记忆冲突：高优先级 > 中优先级 > 低优先级。具体规则覆盖通用规则，IDE错误优先于代码规范。

### Q5：Gmem的性能如何？

A：Gmem使用本地JSON文件存储，性能良好。对于大量记忆，可以使用压缩功能合并相关记忆，提高检索效率。

## 版权声明

本文为博主原创文章，遵循 CC 4.0 BY-NC-SA 版权协议，转载请附上原文出处链接和本声明。

---

**作者**：aspnmy

**日期**：2026-01-27

**标签**：Gmem, AI助手, 全局记忆, 记忆管理, 开发工具

**分类**：AI工具, 开发效率

**摘要**：本文详细介绍如何使用Gmem项目进行全局记忆管理，包括新增记忆、合并规则、更新规则、生成项目和书写文章等内容。Gmem是一个强大的本地JSON记忆存储系统，专为AI助手设计，支持持久化保存和检索重要信息、偏好设置、决策和上下文。

**关键词**：Gmem, 全局记忆, 记忆管理, AI助手, 开发规范, 规则管理, 项目管理, 文章写作

---

## 更新日志

- **2026-01-27**：初始版本发布
