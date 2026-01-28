use chrono::{FixedOffset, Utc};

/// 返回当前时间作为上海时区的 ISO 字符串
///
/// # 返回
/// 上海时区（UTC+8）的 ISO 格式时间字符串
pub fn now_iso() -> String {
    let now = Utc::now();
    let shanghai_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let local_time = now.with_timezone(&shanghai_offset);
    local_time.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
}

/// 生成带时间戳和随机后缀的唯一记忆 ID（上海时区）
///
/// # 返回
/// 唯一的记忆 ID，格式：m_YYYYMMDDTHHMMSSZ_randomhex
pub fn make_id() -> String {
    let now = Utc::now();
    let shanghai_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let local_time = now.with_timezone(&shanghai_offset);
    
    let ts = local_time.format("%Y%m%dT%H%M%S%fZ").to_string();
    let rand: String = (0..3)
        .map(|_| {
            let mut buf = [0u8; 1];
            getrandom::getrandom(&mut buf).unwrap();
            format!("{:02x}", buf[0])
        })
        .collect();
    
    format!("m_{}_{}", ts, rand)
}
