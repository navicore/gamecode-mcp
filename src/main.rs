use rmcp::{ServerHandler, ServiceExt, schemars, tool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{stdin, stdout};
use tracing::{debug, error, info, warn};
use flag_rs::{CommandBuilder, Flag, FlagType, FlagValue};

mod audit;
mod cli_tool;
mod dynamic_tools;

use audit::AuditJournal;
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
    audit: AuditJournal,
}

impl GameCodeMcpServer {
    pub fn new(audit: AuditJournal) -> Self {
        Self {
            tool_manager: DynamicToolManager::new(),
            audit,
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
        // Log tool invocation to audit journal
        self.audit.log_tool_invocation(&req.tool).await;
        
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

    // Parse command line arguments
    let audit_log = Arc::new(Mutex::new(String::new()));
    let audit_log_clone = Arc::clone(&audit_log);
    
    let app = CommandBuilder::new("gamecode-mcp")
        .short("GameCode MCP Server")
        .long("Dynamic CLI tool integration for Claude through YAML configuration")
        .flag(
            Flag::new("audit-log")
                .short('a')
                .usage("Directory path for audit logs (daily rotating JSON lines format)")
                .value_type(FlagType::String)
                .default(FlagValue::String("".to_string()))
        )
        .run(move |ctx| {
            // Get audit log path from flags
            if let Some(log_path) = ctx.flag("audit-log") {
                let mut log = audit_log_clone.lock().unwrap();
                *log = log_path.to_string();
            }
            Ok(())
        })
        .build();
    
    // Execute the app to parse args
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = app.execute(args) {
        error!("Failed to parse arguments: {}", e);
        return Err(e.into());
    }
    
    // Set up audit journal if path provided
    let audit_log_path = audit_log.lock().unwrap();
    let audit_path = if !audit_log_path.is_empty() {
        Some(PathBuf::from(audit_log_path.clone()))
    } else {
        None
    };
    drop(audit_log_path);
    let audit = AuditJournal::new(audit_path);

    info!(
        "Starting GameCode MCP Server v{}...",
        env!("CARGO_PKG_VERSION")
    );
    if audit.is_enabled() {
        info!("Audit logging enabled");
    }
    info!("Loading tool configuration...");

    let server = GameCodeMcpServer::new(audit);

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
