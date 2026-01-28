use gmem_rust_memory_store::MemoryStore;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let memory_path = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    let _store = MemoryStore::new(memory_path);

    println!("MCP Server for Copilot Memory Store");
    println!("This feature is under development");
    
    Ok(())
}
