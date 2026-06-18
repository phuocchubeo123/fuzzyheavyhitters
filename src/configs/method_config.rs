use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodConfig {
    pub check_method: String,
    pub threshold_method: String,
}

impl MethodConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {}: {}", path, e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {}: {}", path, e))
    }
}

impl MethodConfig {
    pub fn from_protocol(protocol: &crate::configs::cli_config::ProtocolParameters) -> Self {
        let (check_method, threshold_method) = match protocol.share_method.as_str() {
            "FSS" => ("FSS".to_string(), "FSS".to_string()),
            _ => ("GC".to_string(), "GC".to_string()),
        };

        Self {
            check_method,
            threshold_method,
        }
    }
}
