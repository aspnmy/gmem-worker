use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆记录结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// 唯一标识符（格式：m_YYYYMMDDTHHMMSSZ_randomhex）
    pub id: String,
    /// 记忆内容文本
    pub text: String,
    /// 用户提供的标签用于分类
    pub tags: Vec<String>,
    /// 自动提取的关键词用于改进搜索
    pub keywords: Vec<String>,
    /// 记忆创建时的 ISO 时间戳
    #[serde(alias = "createdAt")]
    pub created_at: String,
    /// 记忆最后修改时的 ISO 时间戳
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
    /// 如果软删除则为 ISO 时间戳，否则为 null
    #[serde(alias = "deletedAt")]
    pub deleted_at: Option<String>,
}

/// 记忆存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    /// 总记录数（包括已删除的）
    pub total: usize,
    /// 活跃记录数（未删除的）
    pub active: usize,
    /// 软删除记录数
    pub deleted: usize,
    /// 标签频率映射
    pub tags: HashMap<String, usize>,
}

/// 带相关性分数的搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub text: String,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    /// 相关性分数（越高越相关）
    pub score: f64,
}

/// 确定性压缩的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    /// 压缩后的 markdown 输出
    pub markdown: String,
    /// 包含的搜索命中
    pub included: Vec<SearchHit>,
    /// 请求的字符预算
    pub budget: usize,
    /// 实际使用的字符数
    pub used: usize,
}
