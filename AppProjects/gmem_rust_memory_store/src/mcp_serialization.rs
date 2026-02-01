use serde::{Deserialize, Serialize};  
use serde_json::{json, Value};  

/// MCP协议序列化模块
/// 处理Rust蛇形命名法与MCP驼峰命名法的转换

/// JSON-RPC请求结构体
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC响应结构体
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC错误结构体
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// MCP工具结构体
#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// 工具调用参数结构体
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Option<Value>,
}

/// 工具响应结果结构体
#[derive(Debug, Serialize)]
pub struct ToolResponse {
    pub success: bool,
    pub message: String,
    pub result: Option<Value>,
}

/// 命名转换工具函数
/// 将蛇形命名转换为驼峰命名
/// 
/// # 参数
/// * `snake_case` - 蛇形命名的字符串
/// 
/// # 返回
/// 驼峰命名的字符串
pub fn snake_to_camel(snake_case: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for c in snake_case.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

/// 命名转换工具函数
/// 将驼峰命名转换为蛇形命名
/// 
/// # 参数
/// * `camel_case` - 驼峰命名的字符串
/// 
/// # 返回
/// 蛇形命名的字符串
pub fn camel_to_snake(camel_case: &str) -> String {
    let mut result = String::new();
    
    for (i, c) in camel_case.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap());
    }
    
    result
}

/// 创建成功的JSON-RPC响应
/// 
/// # 参数
/// * `id` - 请求ID
/// * `result` - 响应结果
/// 
/// # 返回
/// JSON-RPC响应结构体
pub fn create_success_response(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

/// 创建错误的JSON-RPC响应
/// 
/// # 参数
/// * `id` - 请求ID
/// * `code` - 错误代码
/// * `message` - 错误消息
/// 
/// # 返回
/// JSON-RPC响应结构体
pub fn create_error_response(id: Value, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
        }),
    }
}

/// 创建工具列表响应
/// 
/// # 参数
/// * `id` - 请求ID
/// * `tools` - 工具列表
/// 
/// # 返回
/// JSON-RPC响应结构体
pub fn create_tools_list_response(id: Value, tools: Vec<Tool>) -> JsonRpcResponse {
    create_success_response(id, json!({ "tools": tools }))
}

/// 解析工具调用参数
/// 
/// # 参数
/// * `params` - 原始参数值
/// 
/// # 返回
/// 解析后的工具调用参数
pub fn parse_tool_call_params(params: Option<Value>) -> Result<ToolCallParams, String> {
    match params {
        Some(p) => {
            let tool_name = match p.get("name") {
                Some(Value::String(name)) => name.clone(),
                _ => return Err("Missing or invalid tool name".to_string()),
            };
            
            let arguments = p.get("arguments").cloned();
            
            Ok(ToolCallParams {
                name: tool_name,
                arguments,
            })
        },
        None => Err("Missing params".to_string()),
    }
}
