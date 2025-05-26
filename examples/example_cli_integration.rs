// Example of how to add a CLI tool as an MCP tool
use rmcp::{tool, schemars};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CliToolRequest {
    #[schemars(description = "Tool name to execute")]
    pub tool: String,
    #[schemars(description = "Parameters for the tool")]
    pub params: HashMap<String, Value>,
}

// This would be added to your GameCodeMcpServer impl block
impl GameCodeMcpServer {
    #[tool(description = "Execute a registered CLI tool")]
    async fn execute_cli_tool(&self, #[tool(aggr)] req: CliToolRequest) -> String {
        // In a real implementation, you'd have a registry here
        // For now, let's show the pattern
        
        match req.tool.as_str() {
            "example_json_tool" => {
                // Execute a CLI tool that returns JSON
                let mut cmd = std::process::Command::new("echo");
                cmd.arg(r#"{"status": "success", "data": "example"}"#);
                
                match cmd.output() {
                    Ok(output) => {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    }
                    Err(e) => {
                        format!(r#"{{"error": "Failed to execute: {}"}}"#, e)
                    }
                }
            }
            _ => {
                format!(r#"{{"error": "Unknown tool: {}"}}"#, req.tool)
            }
        }
    }
}

// Alternative approach: Generate individual MCP methods for each CLI tool
// This requires code generation or macros
macro_rules! create_cli_tool_method {
    ($method_name:ident, $tool_name:expr, $description:expr, $command:expr) => {
        #[tool(description = $description)]
        async fn $method_name(&self, #[tool(aggr)] params: HashMap<String, Value>) -> String {
            let mut cmd = std::process::Command::new($command);
            
            // Add arguments based on params
            if let Some(input) = params.get("input") {
                if let Some(s) = input.as_str() {
                    cmd.arg(s);
                }
            }
            
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        format!(
                            r#"{{"error": "Command failed: {}"}}"#, 
                            String::from_utf8_lossy(&output.stderr)
                        )
                    }
                }
                Err(e) => {
                    format!(r#"{{"error": "Failed to execute {}: {}"}}"#, $tool_name, e)
                }
            }
        }
    };
}