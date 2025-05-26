use crate::cli_tool::{CliTool, CliArg, ArgType};
use rmcp::{tool, schemars};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
struct ToolConfig {
    tools: Vec<ToolDefinition>,
}

#[derive(Debug, Deserialize)]
struct ToolDefinition {
    name: String,
    description: String,
    command: String,
    args: Vec<ArgDefinition>,
    #[serde(default)]
    static_flags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ArgDefinition {
    name: String,
    description: String,
    required: bool,
    #[serde(rename = "type")]
    arg_type: String,
    cli_flag: Option<String>,
    #[serde(default)]
    default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DynamicToolManager {
    tools: Arc<RwLock<HashMap<String, CliTool>>>,
}

impl DynamicToolManager {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_from_yaml(&self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: ToolConfig = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        
        let mut tools = self.tools.write().await;
        
        for tool_def in config.tools {
            let cli_tool = self.convert_to_cli_tool(tool_def)?;
            tools.insert(cli_tool.name.clone(), cli_tool);
        }
        
        Ok(())
    }
    
    fn convert_to_cli_tool(&self, def: ToolDefinition) -> Result<CliTool, String> {
        let mut args = Vec::new();
        
        for arg_def in def.args {
            let arg_type = match arg_def.arg_type.as_str() {
                "string" => ArgType::String,
                "number" => ArgType::Number,
                "boolean" => ArgType::Boolean,
                "array" => ArgType::Array,
                _ => return Err(format!("Unknown arg type: {}", arg_def.arg_type)),
            };
            
            args.push(CliArg {
                name: arg_def.name,
                description: arg_def.description,
                required: arg_def.required,
                arg_type,
                cli_flag: arg_def.cli_flag,
            });
        }
        
        Ok(CliTool {
            name: def.name,
            description: def.description,
            command: def.command,
            args,
        })
    }
    
    pub async fn execute_tool(&self, tool_name: &str, params: HashMap<String, serde_json::Value>) -> Result<String, String> {
        let tools = self.tools.read().await;
        
        if let Some(tool) = tools.get(tool_name) {
            tool.execute(params).await
        } else {
            Err(format!("Tool not found: {}", tool_name))
        }
    }
    
    pub async fn list_tools(&self) -> Vec<(String, String)> {
        let tools = self.tools.read().await;
        tools.iter()
            .map(|(name, tool)| (name.clone(), tool.description.clone()))
            .collect()
    }
}

// Generic MCP request for dynamic tools
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DynamicToolRequest {
    #[schemars(description = "Name of the CLI tool to execute")]
    pub tool: String,
    #[schemars(description = "Parameters for the tool as key-value pairs")]
    pub params: HashMap<String, serde_json::Value>,
}

// This is what you'd add to your GameCodeMcpServer
impl GameCodeMcpServer {
    // Initialize the dynamic tool manager (you'd store this in the server struct)
    pub fn dynamic_tools() -> DynamicToolManager {
        static TOOLS: std::sync::OnceLock<DynamicToolManager> = std::sync::OnceLock::new();
        TOOLS.get_or_init(|| {
            let manager = DynamicToolManager::new();
            // Load tools on startup
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Err(e) = manager.load_from_yaml("tools.yaml").await {
                        eprintln!("Failed to load tools: {}", e);
                    }
                })
            });
            manager
        }).clone()
    }
    
    #[tool(description = "Execute a dynamically loaded CLI tool")]
    async fn run_tool(&self, #[tool(aggr)] req: DynamicToolRequest) -> String {
        let manager = Self::dynamic_tools();
        
        match manager.execute_tool(&req.tool, req.params).await {
            Ok(result) => result,
            Err(e) => format!(r#"{{"error": "{}"}}"#, e),
        }
    }
    
    #[tool(description = "List all available dynamic tools")]
    async fn list_dynamic_tools(&self) -> String {
        let manager = Self::dynamic_tools();
        let tools = manager.list_tools().await;
        
        let tool_list: Vec<serde_json::Value> = tools.into_iter()
            .map(|(name, desc)| serde_json::json!({
                "name": name,
                "description": desc
            }))
            .collect();
        
        serde_json::json!({
            "tools": tool_list
        }).to_string()
    }
}

// Example of what the YAML tools would return:
// 
// If you have a CLI tool that returns:
// {"status": "success", "result": {"files": 10, "size": 1024}}
//
// The MCP response would automatically be:
// {
//   "jsonrpc": "2.0",
//   "id": 123,
//   "result": "{\"status\": \"success\", \"result\": {\"files\": 10, \"size\": 1024}}"
// }
//
// The rmcp crate handles the JSONRPC wrapper - your tool just returns the JSON string!