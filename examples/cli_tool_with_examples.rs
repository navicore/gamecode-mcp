use serde::{Deserialize, Serialize};

// Enhanced tool definition that supports examples
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolDefinitionWithExample {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<ArgDefinition>,
    #[serde(default)]
    pub static_flags: Vec<String>,
    // New field for example output
    pub example_output: Option<serde_json::Value>,
}

// Helper function to generate rich descriptions with examples
impl ToolDefinitionWithExample {
    pub fn generate_full_description(&self) -> String {
        let mut desc = self.description.clone();
        
        if let Some(example) = &self.example_output {
            desc.push_str("\n\nExample output:\n```json\n");
            desc.push_str(&serde_json::to_string_pretty(example).unwrap_or_default());
            desc.push_str("\n```");
        }
        
        desc
    }
}