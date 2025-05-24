use rmcp::{schemars, tool, ServerHandler, ServiceExt};
use tokio::io::{stdin, stdout};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddRequest {
    #[schemars(description = "First number to add")]
    pub a: f64,
    #[schemars(description = "Second number to add")]
    pub b: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MultiplyRequest {
    #[schemars(description = "First number to multiply")]
    pub a: f64,
    #[schemars(description = "Second number to multiply")]
    pub b: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListFilesRequest {
    #[schemars(description = "Directory path (defaults to current directory)")]
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GameCodeMcpServer;

#[tool(tool_box)]
impl GameCodeMcpServer {
    #[tool(description = "Add two numbers together")]
    async fn add(&self, #[tool(aggr)] req: AddRequest) -> String {
        let result = req.a + req.b;
        format!("{{\"result\": {}, \"operation\": \"addition\"}}", result)
    }

    #[tool(description = "Multiply two numbers")]
    async fn multiply(&self, #[tool(aggr)] req: MultiplyRequest) -> String {
        let result = req.a * req.b;
        format!("{{\"result\": {}, \"operation\": \"multiplication\"}}", result)
    }

    #[tool(description = "List files in a directory")]
    async fn list_files(&self, #[tool(aggr)] req: ListFilesRequest) -> String {
        let path = req.path.unwrap_or_else(|| ".".to_string());
        
        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(metadata) = entry.metadata() {
                            files.push(format!(
                                "{{\"name\": \"{}\", \"is_dir\": {}, \"size\": {}}}",
                                entry.file_name().to_string_lossy(),
                                metadata.is_dir(),
                                metadata.len()
                            ));
                        }
                    }
                }
                format!(
                    "{{\"path\": \"{}\", \"files\": [{}]}}",
                    path,
                    files.join(", ")
                )
            }
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for GameCodeMcpServer {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting GameCode MCP Server...");
    
    let server = GameCodeMcpServer;
    let transport = (stdin(), stdout());
    
    let service = server.serve(transport).await?;
    let quit_reason = service.waiting().await?;
    
    eprintln!("Server quit: {:?}", quit_reason);
    Ok(())
}