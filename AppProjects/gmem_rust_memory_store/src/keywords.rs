use regex::Regex;
use std::collections::HashMap;

/// 常见英文停用词，在关键词提取时过滤掉
/// 这些词太常见，对搜索相关性没有帮助
const STOP_WORDS: &[&str] = &[
    "i", "me", "my", "we", "our", "you", "your", "he", "she", "it", "they", "them",
    "a", "an", "the", "this", "that", "these", "those",
    "is", "am", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
    "can", "may", "might", "must", "shall",
    "and", "or", "but", "if", "then", "else", "when", "where", "why", "how",
    "all", "each", "every", "both", "few", "more", "most", "some", "any", "no",
    "not", "only", "own", "same", "so", "than", "too", "very",
    "just", "also", "now", "here", "there", "about", "after", "before",
    "to", "from", "up", "down", "in", "out", "on", "off", "over", "under",
    "with", "without", "for", "of", "at", "by", "as", "into", "through",
    "like", "want", "use", "using", "used", "prefer", "always", "never",
];

/// 从文本中提取有意义的关键词用于搜索索引
/// 过滤停用词并按频率返回前 10 个
///
/// # 参数
/// * `text` - 要提取关键词的文本
///
/// # 返回
/// 最多 10 个关键词的数组，按频率排序
pub fn extract_keywords(text: &str) -> Vec<String> {
    let word_re = Regex::new(r"[a-z0-9]+").unwrap();
    let words: Vec<String> = word_re
        .find_iter(text.to_lowercase().as_str())
        .map(|m| m.as_str().to_string())
        .filter(|w| w.len() > 2 && !STOP_WORDS.contains(&w.as_str()))
        .collect();

    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in &words {
        *freq.entry(word.clone()).or_insert(0) += 1;
    }

    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter()
        .take(10)
        .map(|(word, _)| word)
        .collect()
}
