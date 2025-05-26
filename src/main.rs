use rmcp::{ServerHandler, ServiceExt, schemars, tool};
use std::collections::HashMap;
use tokio::io::{stdin, stdout};
use tracing::{debug, error, info, warn};

mod cli_tool;
mod dynamic_tools;

use dynamic_tools::DynamicToolManager;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunToolRequest {
    #[schemars(description = "Name of the tool to execute")]
    pub tool: String,
    #[schemars(description = "Parameters for the tool as key-value pairs")]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
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
            warn!("{}", e);
            warn!("The server will start but no tools will be available.");
            warn!("Please create a tools.yaml file to enable tools.");
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

        let tool_list: Vec<serde_json::Value> = tools
            .into_iter()
            .map(|(name, desc)| {
                serde_json::json!({
                    "name": name,
                    "description": desc
                })
            })
            .collect();

        serde_json::json!({
            "tools": tool_list,
            "total": tool_list.len()
        })
        .to_string()
    }
}

#[tool(tool_box)]
impl ServerHandler for GameCodeMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: Default::default(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { list_changed: None }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "gamecode".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "GameCode MCP Server - Dynamic CLI tool integration. Configure tools in tools.yaml"
                    .to_string(),
            ),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gamecode_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!(
        "Starting GameCode MCP Server v{}...",
        env!("CARGO_PKG_VERSION")
    );
    info!("Loading tool configuration...");

    let server = GameCodeMcpServer::new();

    // Initialize the server and load tools
    server.initialize().await;
    info!("Server initialized");

    let transport = (stdin(), stdout());
    debug!("Transport setup complete");

    info!("Starting MCP service...");
    let service = match server.serve(transport).await {
        Ok(s) => {
            info!("MCP service started successfully!");
            info!("Use 'list_tools' to see available tools");
            s
        }
        Err(e) => {
            error!("Failed to start MCP service: {:?}", e);
            return Err(e.into());
        }
    };

    info!("Service is running, waiting for requests...");
    let quit_reason = match service.waiting().await {
        Ok(reason) => {
            info!("Service terminated normally: {:?}", reason);
            reason
        }
        Err(e) => {
            error!("Service error while waiting: {:?}", e);
            return Err(e.into());
        }
    };

    info!("Server quit: {:?}", quit_reason);
    Ok(())
}
