use crate::cli_tool::{ArgType, CliArg, CliTool};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    #[serde(default)]
    default: Option<String>,
}

#[derive(Default, Debug, Clone)]
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
            home::home_dir().map(|d| {
                d.join(".config/gamecode-mcp/tools.yaml")
                    .to_string_lossy()
                    .to_string()
            }),
            // 3. Current directory
            Some("./tools.yaml".to_string()),
        ];

        for location in locations.into_iter().flatten() {
            debug!("Checking for tools config at: {}", location);
            if std::path::Path::new(&location).exists() {
                info!("Loading tools from: {}", location);
                return self.load_from_yaml(&location).await;
            }
        }

        Err("No tools.yaml found!\n\n\
            To get started:\n\
            1. Copy tools.yaml.example to one of these locations:\n\
               - $GAMECODE_TOOLS_FILE (if set)\n\
               - ~/.config/gamecode-mcp/tools.yaml\n\
               - ./tools.yaml\n\
            2. Customize it with your tools\n\
            3. Restart Claude Desktop\n\n\
            Example: cp tools.yaml.example ~/.config/gamecode-mcp/tools.yaml"
            .to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_load_from_yaml_valid() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("test_tools.yaml");
        
        let yaml_content = r#"
tools:
  - name: test_tool
    description: A test tool
    command: echo
    args:
      - name: message
        description: Message to echo
        required: true
        type: string
        cli_flag: null
    internal_handler: null
"#;
        
        let mut file = File::create(&yaml_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();
        
        let manager = DynamicToolManager::new();
        let result = manager.load_from_yaml(yaml_path.to_str().unwrap()).await;
        
        assert!(result.is_ok());
        
        let tools = manager.list_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "test_tool");
        assert_eq!(tools[0].1, "A test tool");
    }

    #[tokio::test]
    async fn test_load_from_yaml_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("invalid.yaml");
        
        let yaml_content = r#"
tools:
  - name: test_tool
    description: Missing required fields
"#;
        
        let mut file = File::create(&yaml_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();
        
        let manager = DynamicToolManager::new();
        let result = manager.load_from_yaml(yaml_path.to_str().unwrap()).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse YAML"));
    }

    #[tokio::test]
    async fn test_convert_arg_types() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("arg_types.yaml");
        
        let yaml_content = r#"
tools:
  - name: arg_test
    description: Test various arg types
    command: test
    args:
      - name: str_arg
        description: String argument
        required: true
        type: string
        cli_flag: "--string"
      - name: num_arg
        description: Number argument
        required: false
        type: number
        cli_flag: "--number"
      - name: bool_arg
        description: Boolean argument
        required: false
        type: boolean
        cli_flag: "--bool"
      - name: arr_arg
        description: Array argument
        required: false
        type: array
        cli_flag: "--array"
"#;
        
        let mut file = File::create(&yaml_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();
        
        let manager = DynamicToolManager::new();
        manager.load_from_yaml(yaml_path.to_str().unwrap()).await.unwrap();
        
        let tools = manager.tools.read().await;
        let tool = tools.get("arg_test").unwrap();
        
        assert_eq!(tool.args.len(), 4);
        assert!(matches!(tool.args[0].arg_type, ArgType::String));
        assert!(matches!(tool.args[1].arg_type, ArgType::Number));
        assert!(matches!(tool.args[2].arg_type, ArgType::Boolean));
        assert!(matches!(tool.args[3].arg_type, ArgType::Array));
    }

    #[tokio::test]
    async fn test_execute_nonexistent_tool() {
        let manager = DynamicToolManager::new();
        let result = manager.execute_tool("nonexistent", HashMap::new()).await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tool not found: nonexistent");
    }

    #[tokio::test]
    async fn test_load_tool_with_internal_handler() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("internal.yaml");
        
        let yaml_content = r#"
tools:
  - name: internal_add
    description: Internal add handler
    command: internal
    args:
      - name: a
        description: First number
        required: true
        type: number
        cli_flag: null
      - name: b
        description: Second number
        required: true
        type: number
        cli_flag: null
    internal_handler: add
"#;
        
        let mut file = File::create(&yaml_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();
        
        let manager = DynamicToolManager::new();
        manager.load_from_yaml(yaml_path.to_str().unwrap()).await.unwrap();
        
        let mut params = HashMap::new();
        params.insert("a".to_string(), serde_json::json!(3));
        params.insert("b".to_string(), serde_json::json!(4));
        
        let result = manager.execute_tool("internal_add", params).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["result"], 7);
        assert_eq!(parsed["operation"], "addition");
    }
}
