# GmemoryStore

High-performance, type-safe Copilot Memory Store implementation in Rust.

## Language

- [English (this document)](#)
- [中文说明](README-zh.md)

## Project Overview

This project reimplements the core functionality of Copilot Memory Store using Rust, providing a high-performance, type-safe memory storage service. It supports memory CRUD operations, intelligent search, relevance scoring, deterministic compression, and more.

## Core Features

- **High Performance**: Based on Rust compiled language, excellent performance
- **Type Safety**: Fully utilizes Rust's type system to avoid runtime errors
- **Concurrent Safety**: Ensures multi-process concurrent access safety through file locking mechanism
- **Atomic Operations**: Uses temporary file + rename pattern to ensure data consistency
- **Intelligent Search**: Based on keyword, tag, and text content relevance scoring
- **Deterministic Compression**: Compresses related memories into budget-constrained markdown blocks
- **Shanghai Time Zone**: All timestamps use UTC+8 time zone
- **Complete CLI**: Provides interactive command line interface
- **Version Management**: Automatically generates version numbers with timestamps
- **Icon Embedding**: Supports icon embedding on Windows platform
- **Log Management**: Provides log display, clearing, and status query functions

## Installation

### Compile from Source

```bash
# Clone repository
git clone https://github.com/aspnmy/gmem-worker.git
cd gmem-worker

# Compile release version
cargo build --release

# Or use PowerShell script (auto compile and rename)
powershell -ExecutionPolicy Bypass -File build_and_rename.ps1

# Compiled executable is located at target/release/GmemoryStore_v0.1.0-YYYYMMDDHHSS.exe
```

### Direct Download

Download the executable for your platform from the [Releases](https://github.com/aspnmy/gmem-worker/releases) page.

## Usage

### Start CLI

```bash
# Use default memory file path
GmemoryStore

# Specify custom memory file path
GmemoryStore /path/to/memory.json
```

### Command List

#### Add Memory

```bash
> add This is a test memory
✅ Added m_20260128T123456789Z_abc

> add --tags work,important Complete project refactoring task
✅ Added m_20260128T123456789Z_def
```

#### Search Memory

```bash
> search test
1. This is a test memory (score: 25.0)

> search work --limit 5
1. Complete project refactoring task [work, important] (score: 33.0)
```

#### View Statistics

```bash
> stats
Total: 10, Active: 8, Deleted: 2

Tags:
  - work: 5
  - important: 3
  - study: 2
```

#### Soft Delete Memory

```bash
> delete m_20260128T123456789Z_abc
✅ Deleted m_20260128T123456789Z_abc
```

#### Hard Delete Memory

```bash
# Delete by ID
> purge --id m_20260128T123456789Z_abc
✅ Purged 1 memories

# Delete by tag
> purge --tag work
✅ Purged 5 memories

# Delete by text content
> purge --text test
✅ Purged 2 memories
```

#### Compress Memory

```bash
> compress project --budget 1000 --limit 10
--- Compressed Output (856 / 1000 chars) ---
# Copilot Context (auto)

## Relevant memory
- (m_20260128T123456789Z_def) [work, important] Complete project refactoring task
- (m_20260128T123456789Z_ghi) [study] Learn Rust language
--- End ---
```

#### Export Memory

```bash
> export
[
  {
    "id": "m_20260128T123456789Z_def",
    "text": "Complete project refactoring task",
    "tags": ["work", "important"],
    "keywords": ["project", "refactoring", "task"],
    "created_at": "2026-01-28T12:34:56.789+08:00",
    "updated_at": "2026-01-28T12:34:56.789+08:00",
    "deleted_at": null
  }
]
```

#### Import Memory

```bash
> import memories.json
✅ Imported: 5, Skipped: 2, Failed: 0
```

#### Log Management

```bash
# Show recent logs
> logs show

# Clear all logs
> logs clear
✅ Logs cleared

# Show logs status
> logs status
Logs file size: 1.2 KB
```

#### Configuration Management

```bash
# Show config file path
> whereiscfg
Config file: C:\Users\username\.config\GmemoryStore\config.toml
```

#### Help

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

## Core Features

### MCP Server

This project implements an MCP (Model Context Protocol) server for interacting with AI models, providing memory management functionality.

### Start MCP Server

```bash
# Use default configuration
cargo run --release --bin gmemory_mcp_server

# Or use compiled executable
target/release/gmemory_mcp_server
```

### MCP Server Tools

The MCP server implements the following tools:

- `add_memory` - Add a new memory
- `search_memory` - Search for memories
- `compress_memory` - Compress memories
- `delete_memory` - Delete a memory
- `get_stats` - Get memory store statistics

### Memory Record Structure

```rust
pub struct MemoryRecord {
    pub id: String,              // Unique identifier
    pub text: String,            // Memory content
    pub tags: Vec<String>,       // User tags
    pub keywords: Vec<String>,    // Automatically extracted keywords
    pub created_at: String,      // Creation time (Shanghai time zone)
    pub updated_at: String,      // Update time (Shanghai time zone)
    pub deleted_at: Option<String>, // Deletion time (soft delete)
}
```

### Relevance Scoring Algorithm

Scoring formula:
- Each keyword match in text: +5 points
- Each tag match: +8 points
- Each extracted keyword match: +6 points
- Recency: +0-5 points (more recent = higher score)

### Keyword Extraction

- Automatically extracts meaningful keywords from text
- Filters common English stop words (like "the", "is", "and", etc.)
- Sorts by frequency, returns up to 10 keywords

### File Locking Mechanism

- Uses atomic file creation (`create_new`) as locking mechanism
- Default timeout: 2500ms
- Supports custom timeout

### Atomic Operations

- Uses temporary file + rename pattern
- Ensures data consistency, avoids data corruption during write interruptions

### Version Management

- **ver file**: Stores major version number (e.g., 0.1.0)
- **build.rs**: Automatically generates complete version number
  - Reads major version from ver file
  - Generates timestamp (YYYYMMDDHHSS)
  - Builds complete version format: `v0.1.0-YYYYMMDDHHSS`
- **PowerShell script**: Automatically compiles and renames executable
  - Source file: `GmemoryStore.exe`
  - Target file: `GmemoryStore_v0.1.0-YYYYMMDDHHSS.exe`

### Icon Embedding

- **Windows platform**: Uses embed-resource library to embed icons
- **icon.rc**: Defines icon resource file
- **Icon path**: Copy from `E:\AI_Worker\exe_ico\devrom.ico` to project directory

## Development Guide

### Project Structure

```
gmem_rust_memory_store/
├── src/
│   ├── record.rs      # Core data structures
│   ├── timestamp.rs   # Timestamp processing (Shanghai time zone)
│   ├── keywords.rs    # Keyword extraction
│   ├── lock.rs        # File locking mechanism
│   ├── store.rs       # Memory storage core
│   ├── compress.rs    # Deterministic compression
│   ├── logs.rs        # Log management
│   ├── config.rs      # Configuration management
│   ├── cli.rs         # Command line interface
│   ├── lib.rs         # Library entry point
│   ├── main.rs        # CLI executable
│   ├── mcp_server.rs  # MCP server
│   ├── mcp_serialization.rs # MCP serialization
├── src/bin/
│   ├── organize_timer.rs    # Timer-based memory organization
│   ├── organize_once.rs     # One-time memory organization
│   ├── remove_timer_lock.rs # Remove timer lock
│   ├── lock_cleaner.rs      # Lock cleaner tool
│   ├── import_json.rs       # JSON import tool
│   └── ...
├── build.rs           # Build script (version generation, icon compilation)
├── build_and_rename.ps1 # Compile and rename script
├── build_all.ps1      # One-time compile all versions script
├── Cargo.toml         # Project configuration file
├── devrom.ico         # Application icon
├── icon.rc            # Icon resource configuration
├── README.md          # Project documentation (English)
├── README-zh.md       # Project documentation (Chinese)
├── test_regex.rs      # Test file
├── ver                # Version file
└── .gitignore         # Git ignore file
```

### Dependencies

- `chrono` - Time processing (Shanghai time zone support)
- `serde` / `serde_json` - Serialization/deserialization
- `regex` - Regular expressions (keyword extraction)
- `getrandom` - Random number generation
- `fastrand` - Fast random number generation
- `dirs` - Cross-platform directory paths
- `embed-resource` - Windows icon embedding
- `winres` - Windows resource management

### Compilation and Testing

```bash
# Development version compilation
cargo build

# Release version compilation
cargo build --release

# Or use PowerShell script (auto compile and rename)
powershell -ExecutionPolicy Bypass -File build_and_rename.ps1

# One-time compile all versions (recommended)
powershell -ExecutionPolicy Bypass -File build_all.ps1

# Run tests
cargo test

# Code check
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt
```

### Feature Flags

- `default` - Default features (no additional functionality)
- `async` - Enable async support (requires tokio)
- `llm` - Enable LLM compression functionality (requires reqwest)
- `full` - Enable all features (`async` + `llm`)

## License

MIT License

## Contribution

Welcome to submit Issues and Pull Requests!

## Author

aspnmy <support@e2bank.cn>

## Related Links

- [GitHub Repository](https://github.com/aspnmy/gmem-worker)
- [Design Document](https://github.com/aspnmy/gmem-worker/blob/main/AppProjects/gmem_rust_memory_store/docs/gmem_rust_memory_store项目设计.md)
