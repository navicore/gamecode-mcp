use chrono::{DateTime, Local, Utc};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
pub struct AuditJournal {
    directory: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
struct AuditEntry {
    timestamp: DateTime<Utc>,
    tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<String>,
}

impl AuditJournal {
    pub fn new(directory: Option<PathBuf>) -> Self {
        if let Some(ref dir) = directory {
            // Create directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(dir) {
                error!("Failed to create audit directory {:?}: {}", dir, e);
                return Self { directory: None };
            }
            info!("Audit journal enabled in directory: {:?}", dir);
        } else {
            debug!("Audit journal disabled");
        }

        Self { directory }
    }
    
    fn get_daily_file_path(&self) -> Option<PathBuf> {
        self.directory.as_ref().map(|dir| {
            let date = Local::now().format("%Y-%m-%d");
            dir.join(format!("audit-{}.jsonl", date))
        })
    }

    pub async fn log_tool_invocation(&self, tool_name: &str) {
        if let Some(file_path) = self.get_daily_file_path() {
            let entry = AuditEntry {
                timestamp: Utc::now(),
                tool_name: tool_name.to_string(),
                user: std::env::var("USER").ok(),
                hostname: hostname::get()
                    .ok()
                    .and_then(|h| h.to_str().map(|s| s.to_string())),
            };

            // Open file in append mode for each write (safer for concurrent access)
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
            {
                Ok(mut file) => {
                    match serde_json::to_string(&entry) {
                        Ok(json) => {
                            if let Err(e) = writeln!(file, "{}", json) {
                                error!("Failed to write audit entry: {}", e);
                            } else if let Err(e) = file.flush() {
                                error!("Failed to flush audit journal: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize audit entry: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to open audit file {:?}: {}", file_path, e);
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.directory.is_some()
    }
}