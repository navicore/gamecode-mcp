use rmcp::{schemars, tool, ServerHandler, ServiceExt};
use tokio::io::{stdin, stdout};
use std::collections::HashMap;

mod cli_tool;
mod tool_registry;
mod dynamic_tools;

use dynamic_tools::DynamicToolManager;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunToolRequest {
    #[schemars(description = "Name of the tool to execute")]
    pub tool: String,
    #[schemars(description = "Parameters for the tool as key-value pairs")]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct GameCodeMcpServer {
    tool_manager: DynamicToolManager,
}

impl GameCodeMcpServer {
    pub fn new() -> Self {
        Self {
            tool_manager: DynamicToolManager::new(),
        }
    }
    
    pub async fn initialize(&self) {
        if let Err(e) = self.tool_manager.load_from_default_locations().await {
            eprintln!("WARNING: {}", e);
            eprintln!("\nThe server will start but no tools will be available.");
            eprintln!("Please create a tools.yaml file to enable tools.");
        }
    }
}

// Dynamic tool execution - all tools come from YAML now
#[tool(tool_box)]
impl GameCodeMcpServer {
    #[tool(description = "Execute a tool defined in tools.yaml")]
    async fn run(&self, #[tool(aggr)] req: RunToolRequest) -> String {
        match self.tool_manager.execute_tool(&req.tool, req.params).await {
            Ok(result) => result,
            Err(e) => format!(r#"{{"error": "{}"}}"#, e),
        }
    }
    
    #[tool(description = "List all available tools from tools.yaml")]
    async fn list_tools(&self) -> String {
        let tools = self.tool_manager.list_tools().await;
        
        let tool_list: Vec<serde_json::Value> = tools.into_iter()
            .map(|(name, desc)| serde_json::json!({
                "name": name,
                "description": desc
            }))
            .collect();
        
        serde_json::json!({
            "tools": tool_list,
            "total": tool_list.len()
        }).to_string()
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
            instructions: Some(
                "GameCode MCP Server - Dynamic CLI tool integration. Configure tools in tools.yaml".to_string()
            ),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting GameCode MCP Server v{}...", env!("CARGO_PKG_VERSION"));
    eprintln!("Loading tool configuration...");

    let server = GameCodeMcpServer::new();
    
    // Initialize the server and load tools
    server.initialize().await;
    eprintln!("Server initialized");

    let transport = (stdin(), stdout());
    eprintln!("Transport setup complete");

    eprintln!("Starting MCP service...");
    let service = match server.serve(transport).await {
        Ok(s) => {
            eprintln!("MCP service started successfully!");
            eprintln!("Use 'list_tools' to see available tools");
            s
        }
        Err(e) => {
            eprintln!("ERROR: Failed to start MCP service: {:?}", e);
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