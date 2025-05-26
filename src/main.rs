use rmcp::{schemars, tool, ServerHandler, ServiceExt};
use tokio::io::{stdin, stdout};

mod cli_tool;
mod tool_registry;

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
        format!(
            "{{\"result\": {}, \"operation\": \"multiplication\"}}",
            result
        )
    }

    #[tool(description = "List files in a directory")]
    async fn list_files(&self, #[tool(aggr)] req: ListFilesRequest) -> String {
        let path = req.path.unwrap_or_else(|| ".".to_string());

        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                entries.into_iter().for_each(|entry| {
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
                });
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
impl ServerHandler for GameCodeMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: Default::default(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { 
                    list_changed: None 
                }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "gamecode".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some("GameCode MCP Server - Provides tools for arithmetic operations and file listing".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting GameCode MCP Server...");
    eprintln!("Creating server instance...");

    let server = GameCodeMcpServer;
    eprintln!("Server instance created");

    let transport = (stdin(), stdout());
    eprintln!("Transport setup complete");

    eprintln!("Attempting to serve...");
    let service = match server.serve(transport).await {
        Ok(s) => {
            eprintln!("Service created successfully!");
            s
        }
        Err(e) => {
            eprintln!("ERROR: Failed to create service: {:?}", e);
            return Err(e.into());
        }
    };

    eprintln!("Service is running, waiting for requests...");
    let quit_reason = match service.waiting().await {
        Ok(reason) => {
            eprintln!("Service terminated normally: {:?}", reason);
            reason
        }
        Err(e) => {
            eprintln!("ERROR: Service error while waiting: {:?}", e);
            return Err(e.into());
        }
    };

    eprintln!("Server quit: {:?}", quit_reason);
    Ok(())
}
