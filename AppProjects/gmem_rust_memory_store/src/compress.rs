use crate::record::{MemoryRecord, SearchHit, CompressResult};
use crate::store::score_record;

/// 确定性地将相关记忆压缩为预算约束的 markdown 块
/// 使用确定性截断（无 LLM）- 包含记忆直到预算耗尽
///
/// # 参数
/// * `records` - 记忆记录数组
/// * `query` - 查找相关记忆的搜索查询
/// * `budget` - 输出的最大字符数（最小 200）
/// * `limit` - 考虑的最大记忆数（默认 25）
///
/// # 返回
/// 带有 markdown 和元数据的 CompressResult
pub fn compress_deterministic(
    records: &Vec<MemoryRecord>,
    query: &str,
    budget: usize,
    limit: Option<usize>,
) -> CompressResult {
    let budget = budget.max(200);
    let limit = limit.unwrap_or(25);

    let hits = search_records(records, query, Some(limit));

    let mut lines: Vec<String> = Vec::new();
    lines.push("# Copilot Context (auto)".to_string());
    lines.push(String::new());
    lines.push("## Relevant memory".to_string());

    for h in &hits {
        let tag_str = if h.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", h.tags.join(", "))
        };
        lines.push(format!("- ({}){} {}", h.id, tag_str, h.text));
    }

    let md = lines.join("\n") + "\n";
    let md_len = md.len();
    if md.len() <= budget {
        return CompressResult {
            markdown: md,
            included: hits,
            budget,
            used: md_len,
        };
    }

    let mut out: Vec<String> = Vec::new();
    let mut size = 0;
    for line in &lines {
        if size + line.len() + 1 > budget {
            break;
        }
        out.push(line.clone());
        size += line.len() + 1;
    }

    let md2 = out.join("\n") + "\n";
    let md2_len = md2.len();
    CompressResult {
        markdown: md2,
        included: hits,
        budget,
        used: md2_len,
    }
}

/// 使用 LLM 压缩记忆（需要 llm feature）
/// 此功能是预留的，需要外部 LLM 服务支持
///
/// # 参数
/// * `records` - 记忆记录数组
/// * `query` - 查找相关记忆的搜索查询
/// * `budget` - 输出的最大字符数（最小 200）
/// * `limit` - 考虑的最大记忆数（默认 25）
///
/// # 返回
/// 带有 markdown 和元数据的 CompressResult
#[cfg(feature = "llm")]
pub async fn compress_with_llm(
    records: &Vec<MemoryRecord>,
    query: &str,
    budget: usize,
    limit: Option<usize>,
) -> Result<CompressResult, Box<dyn std::error::Error>> {
    let budget = budget.max(200);
    let limit = limit.unwrap_or(25);

    let hits = search_records(records, query, Some(limit));

    let context: String = hits
        .iter()
        .map(|h| format!("{}: {}", h.id, h.text))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "请将以下相关记忆压缩为一个简洁的 markdown 文档，最多 {} 个字符：\n\n查询: {}\n\n记忆:\n{}",
        budget, query, context
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", "Bearer YOUR_API_KEY")
        .json(&serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": budget / 2,
        }))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let markdown = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("# Copilot Context (LLM)\n\n无法生成压缩内容")
        .to_string();

    let used = markdown.len();
    Ok(CompressResult {
        markdown,
        included: hits,
        budget,
        used,
    })
}

/// 搜索记录（内部辅助函数）
///
/// # 参数
/// * `records` - 记忆记录数组
/// * `query` - 搜索查询
/// * `limit` - 返回的最大结果数
///
/// # 返回
/// 按分数降序排列的搜索命中数组
fn search_records(
    records: &Vec<MemoryRecord>,
    query: &str,
    limit: Option<usize>,
) -> Vec<SearchHit> {
    let mut hits: Vec<SearchHit> = Vec::new();
    let limit = limit.unwrap_or(10);

    for r in records {
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
    hits.into_iter().take(std::cmp::max(1, limit)).collect()
}
