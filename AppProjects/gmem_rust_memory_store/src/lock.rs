use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::timestamp::now_iso;

/// 锁文件类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    /// 交互模式锁
    Interactive,
    /// 命令行模式锁
    Cli,
    /// MCP服务器锁
    Mcp,
}

impl LockType {
    /// 获取锁文件后缀
    ///
    /// # 返回
    /// 锁文件后缀
    pub fn suffix(&self) -> &'static str {
        match self {
            LockType::Interactive => ".interactive.lock",
            LockType::Cli => ".cli.lock",
            LockType::Mcp => ".mcp.lock",
        }
    }
}

/// 获取文件锁以实现并发访问安全
/// 使用原子文件创建（wx 标志）作为锁定机制
///
/// # 参数
/// * `lock_path` - 锁文件路径
/// * `timeout_ms` - 等待锁的最大时间（默认 2500ms）
///
/// # 返回
/// 锁文件句柄
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
            Ok(mut file) => {
                writeln!(file, "{} {}", std::process::id(), now_iso())?;
                return Ok(file);
            }
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                if start.elapsed().as_millis() > timeout as u128 {
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

/// 获取文件锁以实现并发访问安全
/// 使用原子文件创建（wx 标志）作为锁定机制
/// 在获取锁之前会检查并清理过期的锁文件
///
/// # 参数
/// * `lock_path` - 锁文件路径
/// * `timeout_ms` - 等待锁的最大时间（默认 2500ms）
/// * `max_age_seconds` - 锁文件最大年龄（秒），超过此年龄的锁文件会被自动删除（默认 300秒=5分钟）
///
/// # 返回
/// 锁文件句柄
///
/// # 错误
/// 如果在超时时间内无法获取锁则返回错误
pub fn acquire_lock_with_cleanup(lock_path: &Path, timeout_ms: Option<u64>, max_age_seconds: Option<u64>) -> io::Result<File> {
    let max_age = max_age_seconds.unwrap_or(300);
    
    // 检查并清理过期的锁文件
    if lock_path.exists() {
        if let Ok(age) = get_lock_file_age(lock_path) {
            if age > max_age {
                println!("发现过期锁文件 ({}秒)，自动删除: {}", age, lock_path.display());
                let _ = fs::remove_file(lock_path);
            }
        }
    }
    
    acquire_lock(lock_path, timeout_ms)
}

/// 获取锁文件年龄（秒）
///
/// # 参数
/// * `lock_path` - 锁文件路径
///
/// # 返回
/// 锁文件年龄（秒）
fn get_lock_file_age(lock_path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(lock_path)?;
    let modified = metadata.modified()?;
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(modified)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(duration.as_secs())
}

/// 清理过期的锁文件
///
/// # 参数
/// * `lock_dir` - 锁文件所在目录
/// * `max_age_seconds` - 锁文件最大年龄（秒），超过此年龄的锁文件会被删除
///
/// # 返回
/// 清理的锁文件数量
pub fn cleanup_expired_locks(lock_dir: &Path, max_age_seconds: Option<u64>) -> usize {
    let max_age = max_age_seconds.unwrap_or(300);
    let mut cleaned = 0;
    
    if !lock_dir.exists() {
        return cleaned;
    }
    
    let lock_suffixes = [".interactive.lock", ".cli.lock", ".mcp.lock"];
    
    if let Ok(entries) = fs::read_dir(lock_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // 检查是否是锁文件
                if lock_suffixes.iter().any(|suffix| file_name.ends_with(suffix)) {
                    if let Ok(age) = get_lock_file_age(&path) {
                        if age > max_age {
                            println!("清理过期锁文件 ({}秒): {}", age, path.display());
                            if fs::remove_file(&path).is_ok() {
                                cleaned += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    cleaned
}

/// 通过删除锁文件来释放文件锁
///
/// # 参数
/// * `lock_path` - 锁文件路径
pub fn release_lock(lock_path: &Path) {
    let _ = fs::remove_file(lock_path);
}
