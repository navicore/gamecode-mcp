use crate::cli_tool::{ArgType, CliArg, CliTool};
use serde::Deserialize;
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
    internal_handler: Option<String>,
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
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: ToolConfig =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let mut tools = self.tools.write().await;

        for tool_def in config.tools {
            let cli_tool = self.convert_to_cli_tool(tool_def)?;
            tools.insert(cli_tool.name.clone(), cli_tool);
        }

        Ok(())
    }
    
    pub async fn load_from_default_locations(&self) -> Result<(), String> {
        // Check locations in precedence order
        let locations = vec![
            // 1. Environment variable
            std::env::var("GAMECODE_TOOLS_FILE").ok(),
            // 2. User config directory
            home::home_dir().map(|d| d.join(".config/gamecode-mcp/tools.yaml").to_string_lossy().to_string()),
            // 3. Current directory
            Some("./tools.yaml".to_string()),
        ];
        
        for location in locations.into_iter().flatten() {
            eprintln!("Checking for tools config at: {}", location);
            if std::path::Path::new(&location).exists() {
                eprintln!("Loading tools from: {}", location);
                return self.load_from_yaml(&location).await;
            }
        }
        
        Err(format!(
            "No tools.yaml found!\n\n\
            To get started:\n\
            1. Copy tools.yaml.example to one of these locations:\n\
               - $GAMECODE_TOOLS_FILE (if set)\n\
               - ~/.config/gamecode-mcp/tools.yaml\n\
               - ./tools.yaml\n\
            2. Customize it with your tools\n\
            3. Restart Claude Desktop\n\n\
            Example: cp tools.yaml.example ~/.config/gamecode-mcp/tools.yaml"
        ))
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
            internal_handler: def.internal_handler,
        })
    }

    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<String, String> {
        let tools = self.tools.read().await;

        if let Some(tool) = tools.get(tool_name) {
            tool.execute(params).await
        } else {
            Err(format!("Tool not found: {}", tool_name))
        }
    }

    pub async fn list_tools(&self) -> Vec<(String, String)> {
        let tools = self.tools.read().await;
        tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.description.clone()))
            .collect()
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
