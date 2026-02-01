use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use serde_json;
use crate::record::{MemoryRecord, StoreStats, SearchHit};
use crate::timestamp::{now_iso, make_id};
use crate::keywords::extract_keywords;
use crate::lock::{acquire_lock_with_cleanup, LockType};

const DEFAULT_MEMORY_PATH: &str = ".copilot-memory.json";

/// 记忆存储结构
pub struct MemoryStore {
    memory_path: PathBuf,
    lock_path: PathBuf,
    lock_type: LockType,
}

impl MemoryStore {
    /// 创建新的记忆存储实例
    ///
    /// # 参数
    /// * `memory_path` - 记忆文件路径（可选，默认为 .copilot-memory.json）
    /// * `lock_type` - 锁文件类型（可选，默认为 Cli）
    ///
    /// # 返回
    /// 新的记忆存储实例
    pub fn new(memory_path: Option<&str>, lock_type: Option<LockType>) -> Self {
        let mp = resolve_memory_path(memory_path);
        let lt = lock_type.unwrap_or(LockType::Cli);
        let lock = resolve_lock_path(&mp, lt);
        Self {
            memory_path: mp,
            lock_path: lock,
            lock_type: lt,
        }
    }

    /// 从磁盘加载记忆存储
    ///
    /// # 返回
    /// 包含所有记录的向量
    pub fn load(&self) -> io::Result<Vec<MemoryRecord>> {
        if !self.memory_path.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(&self.memory_path)?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }

        let data: Vec<MemoryRecord> = serde_json::from_str(&raw)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(data)
    }

    /// 添加新记忆到存储
    /// 自动从文本中提取关键词以改进搜索
    ///
    /// # 参数
    /// * `text` - 记忆内容（必需）
    /// * `tags` - 用于分类的可选标签
    ///
    /// # 返回
    /// 创建的记忆记录
    ///
    /// # 错误
    /// 如果文本为空则返回错误
    pub fn add_memory(&self, text: &str, tags: Option<Vec<String>>) -> io::Result<MemoryRecord> {
        let _lock = acquire_lock_with_cleanup(&self.lock_path, None, Some(300))?;
        let records = self.load()?;

        let t = text.trim();
        if t.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot add an empty memory."));
        }

        let keywords = extract_keywords(t);
        let rec = MemoryRecord {
            id: make_id(),
            text: t.to_string(),
            tags: normalize_tags(tags),
            keywords,
            created_at: now_iso(),
            updated_at: now_iso(),
            deleted_at: None,
        };

        let mut new_records = records;
        new_records.push(rec.clone());
        atomic_write(&self.memory_path, &new_records)?;

        Ok(rec)
    }

    /// 搜索记忆并按相关性排序
    ///
    /// # 参数
    /// * `query` - 搜索查询（空格分隔的关键词）
    /// * `limit` - 返回的最大结果数（默认 10）
    ///
    /// # 返回
    /// 按分数降序排列的搜索命中数组
    pub fn search(&self, query: &str, limit: Option<usize>) -> io::Result<Vec<SearchHit>> {
        let records = self.load()?;
        let limit = limit.unwrap_or(10);

        let mut hits: Vec<SearchHit> = Vec::new();
        for r in &records {
            if r.deleted_at.is_some() {
                continue;
            }
            let score = score_record(r, query);
            if score <= 0.0 {
                continue;
            }
            hits.push(SearchHit {
                id: r.id.clone(),
                text: r.text.clone(),
                tags: r.tags.clone(),
                keywords: r.keywords.clone(),
                created_at: r.created_at.clone(),
                updated_at: r.updated_at.clone(),
                score,
            });
        }

        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(hits.into_iter().take(std::cmp::max(1, limit)).collect())
    }

    /// 计算记忆存储的统计信息
    ///
    /// # 返回
    /// 包括计数和标签频率的统计信息
    pub fn compute_stats(&self) -> io::Result<StoreStats> {
        let records = self.load()?;

        let mut tags: HashMap<String, usize> = HashMap::new();
        let mut deleted = 0;

        for r in &records {
            if r.deleted_at.is_some() {
                deleted += 1;
            }
            for t in &r.tags {
                *tags.entry(t.clone()).or_insert(0) += 1;
            }
        }

        Ok(StoreStats {
            total: records.len(),
            active: records.len() - deleted,
            deleted,
            tags,
        })
    }

    /// 软删除记忆（标记为已删除）
    ///
    /// # 参数
    /// * `id` - 记忆 ID
    ///
    /// # 返回
    /// 是否找到并删除了记忆
    pub fn soft_delete(&self, id: &str) -> io::Result<bool> {
        let _lock = acquire_lock_with_cleanup(&self.lock_path, None, Some(300))?;
        let mut records = self.load()?;
        
        let found = records.iter_mut().any(|r| {
            if r.id == id && r.deleted_at.is_none() {
                r.deleted_at = Some(now_iso());
                r.updated_at = now_iso();
                true
            } else {
                false
            }
        });
        
        if found {
            atomic_write(&self.memory_path, &records)?;
        }
        
        Ok(found)
    }

    /// 硬删除记忆（永久删除）
    ///
    /// # 参数
    /// * `id` - 记忆 ID（可选）
    /// * `tag` - 标签匹配（可选）
    /// * `match_text` - 文本匹配（可选）
    ///
    /// # 返回
    /// 删除的记忆数量
    pub fn purge(&self, id: Option<&str>, tag: Option<&str>, match_text: Option<&str>) -> io::Result<usize> {
        let _lock = acquire_lock_with_cleanup(&self.lock_path, None, Some(300))?;
        let mut records = self.load()?;
        
        let initial_len = records.len();
        records.retain(|r| {
            if let Some(id_val) = id {
                if r.id == id_val {
                    return false;
                }
            }
            if let Some(tag_val) = tag {
                if r.tags.iter().any(|t| t == tag_val) {
                    return false;
                }
            }
            if let Some(text_val) = match_text {
                if r.text.contains(text_val) {
                    return false;
                }
            }
            true
        });
        
        let purged = initial_len - records.len();
        if purged > 0 {
            atomic_write(&self.memory_path, &records)?;
        }
        
        Ok(purged)
    }

    /// 导出所有记忆为 JSON 字符串
    ///
    /// # 返回
    /// JSON 格式的记忆数据
    pub fn export_json(&self) -> io::Result<String> {
        let records = self.load()?;
        serde_json::to_string_pretty(&records)
            .map_err(io::Error::other)
    }

    /// 从 JSON 导入记忆
    ///
    /// # 参数
    /// * `json_data` - JSON 格式的记忆数据
    ///
    /// # 返回
    /// (成功数量, 跳过数量, 失败数量)
    pub fn import_json(&self, json_data: &str) -> io::Result<(usize, usize, usize)> {
        let _lock = acquire_lock_with_cleanup(&self.lock_path, None, Some(300))?;
        let mut records = self.load()?;

        let imported: Vec<MemoryRecord> = serde_json::from_str(json_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let existing_ids: std::collections::HashSet<String> = 
            records.iter().map(|r| r.id.clone()).collect();

        let mut success = 0;
        let mut skipped = 0;

        for mut rec in imported {
            if existing_ids.contains(&rec.id) {
                skipped += 1;
                continue;
            }

            rec.created_at = now_iso();
            rec.updated_at = now_iso();
            records.push(rec);
            success += 1;
        }

        atomic_write(&self.memory_path, &records)?;

        Ok((success, skipped, 0))
    }

    /// 获取锁文件路径
    ///
    /// # 返回
    /// 锁文件路径
    pub fn get_lock_path(&self) -> &std::path::Path {
        &self.lock_path
    }

    /// 获取锁类型
    ///
    /// # 返回
    /// 锁类型
    pub fn get_lock_type(&self) -> LockType {
        self.lock_type
    }
}

/// 规范化标签为小写、修剪、唯一值
fn normalize_tags(tags: Option<Vec<String>>) -> Vec<String> {
    match tags {
        Some(tags) => {
            let mut out = std::collections::HashSet::new();
            for t in tags {
                let cleaned = t.trim().to_lowercase();
                if !cleaned.is_empty() {
                    out.insert(cleaned);
                }
            }
            out.into_iter().collect()
        }
        None => Vec::new(),
    }
}

/// 解析记忆文件路径
fn resolve_memory_path(p: Option<&str>) -> PathBuf {
    let raw = p.unwrap_or(DEFAULT_MEMORY_PATH).trim();
    let path = Path::new(raw);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    }
}

/// 解析锁文件路径
/// 解析锁文件路径
///
/// # 参数
/// * `memory_path` - 记忆文件路径
/// * `lock_type` - 锁文件类型
///
/// # 返回
/// 锁文件路径
fn resolve_lock_path(memory_path: &Path, lock_type: LockType) -> PathBuf {
    let lock_name = format!(".copilot-memory{}", lock_type.suffix());
    memory_path.parent().unwrap().join(lock_name)
}

/// 使用临时文件 + 重命名模式原子性写入
fn atomic_write(path: &Path, data: &Vec<MemoryRecord>) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = format!("{}.tmp.{}.tmp", path.display(), std::process::id());
    let tmp = Path::new(&tmp_path);

    let json = serde_json::to_string_pretty(data)
        .map_err(io::Error::other)?;

    fs::write(tmp, json)?;
    fs::rename(tmp, path)?;

    Ok(())
}

/// 计算记录相对于查询的相关性分数
///
/// 评分公式：
/// - 文本中每个关键词匹配 +5 分
/// - 每个标签匹配 +8 分
/// - 每个提取的关键词匹配 +6 分
/// - 时效性 +0-5 分（更新 = 更高）
///
/// # 参数
/// * `r` - 要评分的记忆记录
/// * `query` - 搜索查询
///
/// # 返回
/// 数值相关性分数（0 = 无匹配）
pub fn score_record(r: &MemoryRecord, query: &str) -> f64 {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0.0;
    }

    let text = r.text.to_lowercase();
    let mut score = 0.0;

    for token in q.split_whitespace() {
        let re = regex::Regex::new(&format!(r"(?i){}", regex::escape(token))).unwrap();
        let hits = re.find_iter(&text).count();
        score += hits as f64 * 5.0;

        if r.tags.iter().any(|t| t.to_lowercase() == token) {
            score += 8.0;
        }

        if r.keywords.iter().any(|k| k == token) {
            score += 6.0;
        }
    }

    let age_ms = chrono::Utc::now()
        .signed_duration_since(
            chrono::DateTime::parse_from_rfc3339(&r.updated_at)
                .unwrap_or_else(|_| chrono::DateTime::parse_from_rfc3339(&r.created_at).unwrap())
                .with_timezone(&chrono::Utc)
        )
        .num_milliseconds()
        .abs();

    let days = age_ms as f64 / (1000.0 * 60.0 * 60.0 * 24.0);
    let recency = (5.0 - (days / 30.0).min(5.0)).max(0.0);
    score += recency;

    score
}
