use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// 日志级别
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub enabled: bool,
    pub logs_dir: PathBuf,
    pub max_size: u64,
    pub level: LogLevel,
    pub debug_mode: bool,
}

/// 日志记录器
pub struct Logger {
    config: LogConfig,
    current_file: Option<PathBuf>,
    file_handle: Option<fs::File>,
    file_size: u64,
    rotation_count: usize,
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new(config: LogConfig) -> Self {
        Self {
            config,
            current_file: None,
            file_handle: None,
            file_size: 0,
            rotation_count: 0,
        }
    }

    /// 初始化日志记录器
    pub fn init(&mut self) -> io::Result<()> {
        if !self.config.enabled && !self.config.debug_mode {
            return Ok(());
        }

        // 确保日志目录存在
        if let Err(e) = fs::create_dir_all(&self.config.logs_dir) {
            return Err(e);
        }

        // 初始化日志文件
        self.rotate_log_file()?;
        Ok(())
    }

    /// 记录日志
    pub fn log(&mut self, level: LogLevel, message: &str) {
        if level < self.config.level && !self.config.debug_mode {
            return;
        }

        let timestamp = self.get_timestamp();
        let log_message = format!("[{}] [{}] {}", timestamp, level.as_str(), message);

        // 在debug模式下或error级别时，输出到控制台
        if self.config.debug_mode || level == LogLevel::Error {
            println!("{}", log_message);
        }

        // 如果启用了日志文件，则写入文件
        if self.config.enabled {
            if let Err(e) = self.write_to_file(&log_message) {
                eprintln!("Failed to write log: {}", e);
            }
        }
    }

    /// 写入日志到文件
    fn write_to_file(&mut self, message: &str) -> io::Result<()> {
        // 检查是否需要轮换日志文件
        if self.file_size >= self.config.max_size {
            self.rotate_log_file()?;
        }

        // 确保文件句柄存在
        if self.file_handle.is_none() {
            self.rotate_log_file()?;
        }

        // 写入日志
        if let Some(file) = &mut self.file_handle {
            writeln!(file, "{}", message)?;
            file.flush()?;
            self.file_size += message.len() as u64 + 1; // +1 for newline
        }

        Ok(())
    }

    /// 轮换日志文件
    fn rotate_log_file(&mut self) -> io::Result<()> {
        // 关闭当前文件
        self.file_handle = None;

        // 生成新的日志文件名
        let file_name = self.generate_log_file_name();
        let file_path = self.config.logs_dir.join(file_name);

        // 打开新文件
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        // 更新状态
        self.current_file = Some(file_path);
        self.file_handle = Some(file);
        self.file_size = 0;
        self.rotation_count += 1;

        Ok(())
    }

    /// 生成日志文件名
    fn generate_log_file_name(&self) -> String {
        // 简单的时间戳格式：yyyy-mm-dd-hh
        let timestamp = self.get_date_hour();
        
        if self.rotation_count == 0 {
            format!("{}.log", timestamp)
        } else {
            format!("{}.log{}", timestamp, self.rotation_count)
        }
    }

    /// 获取日期和小时
    fn get_date_hour(&self) -> String {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let seconds = since_epoch.as_secs();
        
        // 简单的时间戳计算
        let hours = seconds / 3600;
        let days = hours / 24;
        
        // 假设从2024-01-01开始
        let start_year = 2024;
        let start_month = 1;
        let start_day = 1;
        
        // 这里应该使用chrono库来正确计算日期，这里简化处理
        format!("{:04}-{:02}-{:02}-{:02}", start_year, start_month, start_day + days, hours % 24)
    }

    /// 获取详细时间戳
    fn get_timestamp(&self) -> String {
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let seconds = since_epoch.as_secs();
        let nanos = since_epoch.subsec_nanos();
        
        format!("{}.{:09}", seconds, nanos)
    }
}

// 全局日志记录器
lazy_static::lazy_static! {
    pub static ref GLOBAL_LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
}

/// 初始化全局日志记录器
pub fn init_global_logger(config: LogConfig) -> io::Result<()> {
    let mut logger = Logger::new(config);
    logger.init()?;
    *GLOBAL_LOGGER.lock().unwrap() = Some(logger);
    Ok(())
}

/// 记录debug级别日志
pub fn debug(message: &str) {
    if let Some(logger) = &mut *GLOBAL_LOGGER.lock().unwrap() {
        logger.log(LogLevel::Debug, message);
    }
}

/// 记录info级别日志
pub fn info(message: &str) {
    if let Some(logger) = &mut *GLOBAL_LOGGER.lock().unwrap() {
        logger.log(LogLevel::Info, message);
    }
}

/// 记录warn级别日志
pub fn warn(message: &str) {
    if let Some(logger) = &mut *GLOBAL_LOGGER.lock().unwrap() {
        logger.log(LogLevel::Warn, message);
    }
}

/// 记录error级别日志
pub fn error(message: &str) {
    if let Some(logger) = &mut *GLOBAL_LOGGER.lock().unwrap() {
        logger.log(LogLevel::Error, message);
    }
}
