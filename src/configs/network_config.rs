use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Address for server 0
    pub server0_addr: String,
    /// Address for server 1
    pub server1_addr: String,
    /// Port for server 0
    pub server0_to_server1_port: u16,
    /// Port for dealer to server 0 communication
    pub dealer_to_server0_port: u16,
    /// Port for dealer to server 1 communication
    pub dealer_to_server1_port: u16,
    /// Port for client to server 0 communication
    pub client_to_server0_port: u16,
    /// Port for client to server 1 communication
    pub client_to_server1_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkConfigFile {
    network: NetworkConfig,
}

impl NetworkConfig {
    /// Load network configuration from a JSON file.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {}: {}", path, e))?;

        let config_file: NetworkConfigFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file {}: {}", path, e))?;
        Ok(config_file.network)
    }
}
