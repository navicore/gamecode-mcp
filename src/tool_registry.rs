use crate::cli_tool::{ArgType, CliArg, CliTool};
use serde_json::Value;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, CliTool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: CliTool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&CliTool> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<&CliTool> {
        self.tools.values().collect()
    }

    // Helper to create a sample registry with example tools
    pub fn with_examples() -> Self {
        let mut registry = Self::new();

        // Example: A JSON formatter tool
        registry.register(CliTool {
            name: "json_format".to_string(),
            description: "Format JSON data".to_string(),
            command: "jq".to_string(),
            args: vec![
                CliArg {
                    name: "filter".to_string(),
                    description: "JQ filter expression".to_string(),
                    required: true,
                    arg_type: ArgType::String,
                    cli_flag: None, // Positional argument
                },
                CliArg {
                    name: "compact".to_string(),
                    description: "Compact output".to_string(),
                    required: false,
                    arg_type: ArgType::Boolean,
                    cli_flag: Some("-c".to_string()),
                },
            ],
            internal_handler: None,
        });

        // Example: A file info tool
        registry.register(CliTool {
            name: "file_info".to_string(),
            description: "Get file information".to_string(),
            command: "stat".to_string(),
            args: vec![
                CliArg {
                    name: "path".to_string(),
                    description: "File path".to_string(),
                    required: true,
                    arg_type: ArgType::String,
                    cli_flag: None,
                },
                CliArg {
                    name: "format".to_string(),
                    description: "Output format".to_string(),
                    required: false,
                    arg_type: ArgType::String,
                    cli_flag: Some("-f".to_string()),
                },
            ],
            internal_handler: None,
        });

        registry
    }
}

// Convert CLI tool to MCP tool metadata
impl CliTool {
    pub fn to_mcp_metadata(&self) -> serde_json::Value {
        let mut params = serde_json::Map::new();

        for arg in &self.args {
            let mut field = serde_json::Map::new();
            field.insert(
                "description".to_string(),
                Value::String(arg.description.clone()),
            );
            field.insert(
                "type".to_string(),
                Value::String(
                    match arg.arg_type {
                        ArgType::String => "string",
                        ArgType::Number => "number",
                        ArgType::Boolean => "boolean",
                        ArgType::Array => "array",
                    }
                    .to_string(),
                ),
            );

            params.insert(arg.name.clone(), Value::Object(field));
        }

        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": params,
                "required": self.args.iter()
                    .filter(|a| a.required)
                    .map(|a| a.name.clone())
                    .collect::<Vec<_>>()
            }
        })
    }
}
