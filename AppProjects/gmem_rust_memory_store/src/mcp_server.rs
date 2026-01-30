use gmem_rust_memory_store::MemoryStore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let memory_path = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    let store = MemoryStore::new(memory_path);
    
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = TokioBufReader::new(stdin);
    let mut writer = stdout;
    
    let mut line = String::new();
    
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            break;
        }
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        match serde_json::from_str::<JsonRpcRequest>(line) {
            Ok(request) => {
                let response = handle_request(&store, &request).await;
                let response_json = serde_json::to_string(&response)?;
                writer.write_all(response_json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writer.write_all(response_json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }
    }
    
    Ok(())
}

async fn handle_request(store: &MemoryStore, request: &JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id.clone()),
        "tools/list" => handle_tools_list(store, request.id.clone()),
        "tools/call" => handle_tools_call(store, request.params.clone(), request.id.clone()).await,
        "shutdown" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(json!({})),
            error: None,
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    }
}

fn handle_initialize(id: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "gmem-store",
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

fn handle_tools_list(_store: &MemoryStore, id: Value) -> JsonRpcResponse {
    let tools = vec![
        Tool {
            name: "add_memory".to_string(),
            description: "Add a new memory to the store".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The memory text to store"
                    },
                    "tags": {
                        "type": "string",
                        "description": "Comma-separated tags (optional)"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "search_memory".to_string(),
            description: "Search for memories in the store".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results (optional)"
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "compress_memory".to_string(),
            description: "Compress related memories into a markdown block".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Query to find related memories"
                    },
                    "budget": {
                        "type": "number",
                        "description": "Maximum character budget"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of memories to compress"
                    }
                },
                "required": ["query", "budget"]
            }),
        },
        Tool {
            name: "delete_memory".to_string(),
            description: "Soft delete a memory by ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Memory ID to delete"
                    }
                },
                "required": ["id"]
            }),
        },
        Tool {
            name: "get_stats".to_string(),
            description: "Get memory store statistics".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ];
    
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({ "tools": tools })),
        error: None,
    }
}

async fn handle_tools_call(store: &MemoryStore, params: Option<Value>, id: Value) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                }),
            }
        }
    };
    
    let tool_name = match params.get("name") {
        Some(Value::String(name)) => name.clone(),
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing tool name".to_string(),
                }),
            }
        }
    };
    
    let arguments = match params.get("arguments") {
        Some(args) => args.clone(),
        None => json!({}),
    };
    
    match tool_name.as_str() {
        "add_memory" => handle_add_memory(store, arguments, id),
        "search_memory" => handle_search_memory(store, arguments, id),
        "compress_memory" => handle_compress_memory(store, arguments, id),
        "delete_memory" => handle_delete_memory(store, arguments, id),
        "get_stats" => handle_get_stats(store, id),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Tool not found: {}", tool_name),
            }),
        },
    }
}

fn handle_add_memory(store: &MemoryStore, arguments: Value, id: Value) -> JsonRpcResponse {
    let text = match arguments.get("text") {
        Some(Value::String(t)) => t.clone(),
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing or invalid text parameter".to_string(),
                }),
            }
        }
    };
    
    let tags: Vec<String> = match arguments.get("tags") {
        Some(Value::String(t)) => t.split(',').map(|s| s.trim().to_string()).collect(),
        _ => vec![],
    };
    
    match store.add_memory(&text, Some(tags)) {
        Ok(record) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "success": true,
                "id": record.id,
                "message": "Memory added successfully"
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to add memory: {}", e),
            }),
        },
    }
}

fn handle_search_memory(store: &MemoryStore, arguments: Value, id: Value) -> JsonRpcResponse {
    let query = match arguments.get("query") {
        Some(Value::String(q)) => q.clone(),
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing or invalid query parameter".to_string(),
                }),
            }
        }
    };
    
    let limit: usize = match arguments.get("limit") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(10) as usize,
        _ => 10,
    };
    
    match store.search(&query, Some(limit)) {
        Ok(results) => {
            let memories: Vec<Value> = results.iter().map(|hit| {
                json!({
                    "id": hit.id,
                    "text": hit.text,
                    "tags": hit.tags,
                    "score": hit.score,
                    "created_at": hit.created_at
                })
            }).collect();
            
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "memories": memories,
                    "count": memories.len()
                })),
                error: None,
            }
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to search memory: {}", e),
            }),
        },
    }
}

fn handle_compress_memory(store: &MemoryStore, arguments: Value, id: Value) -> JsonRpcResponse {
    let query = match arguments.get("query") {
        Some(Value::String(q)) => q.clone(),
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing or invalid query parameter".to_string(),
                }),
            }
        }
    };
    
    let budget: usize = match arguments.get("budget") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(1000) as usize,
        _ => 1000,
    };
    
    let limit: usize = match arguments.get("limit") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(10) as usize,
        _ => 10,
    };
    
    match store.search(&query, Some(limit)) {
        Ok(results) => {
            let compressed = results.iter()
                .map(|hit| format!("- ({}) [{}] {}", hit.id, hit.tags.join(", "), hit.text))
                .collect::<Vec<_>>()
                .join("\n");
            
            let markdown = format!(
                "# Copilot Context (auto)\n\n## Relevant memory\n{}\n--- End ---",
                compressed
            );
            
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "compressed": markdown,
                    "length": markdown.len(),
                    "budget": budget
                })),
                error: None,
            }
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to compress memory: {}", e),
            }),
        },
    }
}

fn handle_delete_memory(store: &MemoryStore, arguments: Value, id: Value) -> JsonRpcResponse {
    let memory_id = match arguments.get("id") {
        Some(Value::String(id)) => id.clone(),
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing or invalid id parameter".to_string(),
                }),
            }
        }
    };
    
    match store.soft_delete(&memory_id) {
        Ok(_) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "success": true,
                "message": "Memory deleted successfully"
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to delete memory: {}", e),
            }),
        },
    }
}

fn handle_get_stats(store: &MemoryStore, id: Value) -> JsonRpcResponse {
    match store.compute_stats() {
        Ok(stats) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "total": stats.total,
                "active": stats.active,
                "deleted": stats.deleted,
                "tags": stats.tags
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to get stats: {}", e),
            }),
        },
    }
}
