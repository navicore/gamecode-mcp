use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CliTool {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<CliArg>,
    pub internal_handler: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CliArg {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub required: bool,
    pub arg_type: ArgType,
    pub cli_flag: Option<String>, // e.g., "--input" or "-i"
}

#[derive(Debug, Clone)]
pub enum ArgType {
    String,
    Number,
    Boolean,
    Array,
}

impl CliTool {
    pub async fn execute(&self, params: HashMap<String, Value>) -> Result<String, String> {
        // Handle internal commands
        if let Some(handler) = &self.internal_handler {
            return self.execute_internal(handler, params).await;
        }

        let mut cmd = Command::new(&self.command);

        // Map MCP parameters to CLI arguments
        for arg in &self.args {
            if let Some(value) = params.get(&arg.name) {
                match &arg.cli_flag {
                    Some(flag) => {
                        cmd.arg(flag);
                        cmd.arg(self.format_value(value, &arg.arg_type)?);
                    }
                    None => {
                        // Positional argument
                        cmd.arg(self.format_value(value, &arg.arg_type)?);
                    }
                }
            } else if arg.required {
                return Err(format!("Missing required argument: {}", arg.name));
            }
        }

        // Execute command
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute {}: {}", self.command, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Command failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Validate JSON output
        let json_result: Value =
            serde_json::from_str(&stdout).map_err(|e| format!("Invalid JSON output: {}", e))?;

        Ok(json_result.to_string())
    }

    async fn execute_internal(
        &self,
        handler: &str,
        params: HashMap<String, Value>,
    ) -> Result<String, String> {
        match handler {
            "add" => {
                let a = params
                    .get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing or invalid parameter 'a'")?;
                let b = params
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing or invalid parameter 'b'")?;
                Ok(format!(
                    r#"{{"result": {}, "operation": "addition"}}"#,
                    a + b
                ))
            }
            "multiply" => {
                let a = params
                    .get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing or invalid parameter 'a'")?;
                let b = params
                    .get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing or invalid parameter 'b'")?;
                Ok(format!(
                    r#"{{"result": {}, "operation": "multiplication"}}"#,
                    a * b
                ))
            }
            "list_files" => {
                let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");

                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        entries.into_iter().for_each(|entry| {
                            if let Ok(entry) = entry {
                                if let Ok(metadata) = entry.metadata() {
                                    files.push(format!(
                                        r#"{{"name": "{}", "is_dir": {}, "size": {}}}"#,
                                        entry.file_name().to_string_lossy(),
                                        metadata.is_dir(),
                                        metadata.len()
                                    ));
                                }
                            }
                        });
                        Ok(format!(
                            r#"{{"path": "{}", "files": [{}]}}"#,
                            path,
                            files.join(", ")
                        ))
                    }
                    Err(e) => Ok(format!(r#"{{"error": "{}"}}"#, e)),
                }
            }
            _ => Err(format!("Unknown internal handler: {}", handler)),
        }
    }

    fn format_value(&self, value: &Value, arg_type: &ArgType) -> Result<String, String> {
        match arg_type {
            ArgType::String => Ok(value.as_str().ok_or("Expected string value")?.to_string()),
            ArgType::Number => Ok(value.as_f64().ok_or("Expected number value")?.to_string()),
            ArgType::Boolean => Ok(value.as_bool().ok_or("Expected boolean value")?.to_string()),
            ArgType::Array => Ok(serde_json::to_string(value)
                .map_err(|e| format!("Failed to serialize array: {}", e))?),
        }
    }
}

// Macro to simplify tool definition
#[macro_export]
macro_rules! define_cli_tool {
    (
        name: $name:expr,
        description: $desc:expr,
        command: $cmd:expr,
        args: [
            $( {
                name: $arg_name:expr,
                description: $arg_desc:expr,
                required: $required:expr,
                arg_type: $arg_type:expr,
                cli_flag: $cli_flag:expr
            } ),*
        ]
    ) => {
        CliTool {
            name: $name.to_string(),
            description: $desc.to_string(),
            command: $cmd.to_string(),
            args: vec![
                $(
                    CliArg {
                        name: $arg_name.to_string(),
                        description: $arg_desc.to_string(),
                        required: $required,
                        arg_type: $arg_type,
                        cli_flag: $cli_flag,
                    }
                ),*
            ],
            internal_handler: None,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_string_value() {
        let tool = CliTool {
            name: "test".to_string(),
            description: "test tool".to_string(),
            command: "echo".to_string(),
            args: vec![],
            internal_handler: None,
        };
        
        let value = json!("hello world");
        let result = tool.format_value(&value, &ArgType::String);
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_format_number_value() {
        let tool = CliTool {
            name: "test".to_string(),
            description: "test tool".to_string(),
            command: "echo".to_string(),
            args: vec![],
            internal_handler: None,
        };
        
        let value = json!(42.5);
        let result = tool.format_value(&value, &ArgType::Number);
        assert_eq!(result.unwrap(), "42.5");
    }

    #[test]
    fn test_format_boolean_value() {
        let tool = CliTool {
            name: "test".to_string(),
            description: "test tool".to_string(),
            command: "echo".to_string(),
            args: vec![],
            internal_handler: None,
        };
        
        let value = json!(true);
        let result = tool.format_value(&value, &ArgType::Boolean);
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_format_array_value() {
        let tool = CliTool {
            name: "test".to_string(),
            description: "test tool".to_string(),
            command: "echo".to_string(),
            args: vec![],
            internal_handler: None,
        };
        
        let value = json!(["item1", "item2"]);
        let result = tool.format_value(&value, &ArgType::Array);
        assert_eq!(result.unwrap(), r#"["item1","item2"]"#);
    }

    #[tokio::test]
    async fn test_internal_handler_add() {
        let tool = CliTool {
            name: "add".to_string(),
            description: "Add two numbers".to_string(),
            command: "internal".to_string(),
            args: vec![],
            internal_handler: Some("add".to_string()),
        };

        let mut params = HashMap::new();
        params.insert("a".to_string(), json!(5.5));
        params.insert("b".to_string(), json!(2.5));

        let result = tool.execute(params).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["result"], 8.0);
        assert_eq!(parsed["operation"], "addition");
    }

    #[tokio::test]
    async fn test_internal_handler_multiply() {
        let tool = CliTool {
            name: "multiply".to_string(),
            description: "Multiply two numbers".to_string(),
            command: "internal".to_string(),
            args: vec![],
            internal_handler: Some("multiply".to_string()),
        };

        let mut params = HashMap::new();
        params.insert("a".to_string(), json!(4));
        params.insert("b".to_string(), json!(7));

        let result = tool.execute(params).await.unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        assert_eq!(parsed["result"], 28);
        assert_eq!(parsed["operation"], "multiplication");
    }

    #[tokio::test]
    async fn test_missing_required_argument() {
        let tool = CliTool {
            name: "test".to_string(),
            description: "test tool".to_string(),
            command: "echo".to_string(),
            args: vec![
                CliArg {
                    name: "required_arg".to_string(),
                    description: "A required argument".to_string(),
                    required: true,
                    arg_type: ArgType::String,
                    cli_flag: Some("--input".to_string()),
                }
            ],
            internal_handler: None,
        };

        let params = HashMap::new(); // Empty params
        let result = tool.execute(params).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required argument: required_arg"));
    }

    #[tokio::test]
    async fn test_unknown_internal_handler() {
        let tool = CliTool {
            name: "test".to_string(),
            description: "test tool".to_string(),
            command: "internal".to_string(),
            args: vec![],
            internal_handler: Some("unknown_handler".to_string()),
        };

        let params = HashMap::new();
        let result = tool.execute(params).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown internal handler: unknown_handler"));
    }
}
