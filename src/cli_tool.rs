use std::collections::HashMap;
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CliTool {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<CliArg>,
}

#[derive(Debug, Clone)]
pub struct CliArg {
    pub name: String,
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
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute {}: {}", self.command, e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Command failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Validate JSON output
        let json_result: Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Invalid JSON output: {}", e))?;
        
        Ok(json_result.to_string())
    }
    
    fn format_value(&self, value: &Value, arg_type: &ArgType) -> Result<String, String> {
        match arg_type {
            ArgType::String => Ok(value.as_str()
                .ok_or("Expected string value")?
                .to_string()),
            ArgType::Number => Ok(value.as_f64()
                .ok_or("Expected number value")?
                .to_string()),
            ArgType::Boolean => Ok(value.as_bool()
                .ok_or("Expected boolean value")?
                .to_string()),
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
        }
    };
}