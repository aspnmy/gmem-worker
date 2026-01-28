use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::timestamp::now_iso;

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

/// 通过删除锁文件来释放文件锁
///
/// # 参数
/// * `lock_path` - 锁文件路径
pub fn release_lock(lock_path: &Path) {
    let _ = fs::remove_file(lock_path);
}
