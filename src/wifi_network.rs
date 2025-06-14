use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiNetwork {
    pub id: String,
    pub name: String,
    pub ssid: String,
    pub password: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

impl WiFiNetwork {
    pub fn new(name: String, ssid: String, password: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            ssid,
            password,
            description,
            created_at: chrono::Utc::now(),
            is_active: true,
        }
    }
}

#[derive(Debug)]
pub struct WiFiNetworkManager {
    networks: HashMap<String, WiFiNetwork>,
}

impl WiFiNetworkManager {
    pub fn new() -> Self {
        Self {
            networks: HashMap::new(),
        }
    }

    pub fn add_network(&mut self, network: WiFiNetwork) {
        self.networks.insert(network.id.clone(), network);
    }

    pub fn get_network(&self, id: &str) -> Option<&WiFiNetwork> {
        self.networks.get(id)
    }

    pub fn get_all_networks(&self) -> Vec<&WiFiNetwork> {
        let mut networks: Vec<&WiFiNetwork> = self.networks.values().collect();
        networks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        networks
    }

    pub fn remove_network(&mut self, id: &str) -> Option<WiFiNetwork> {
        self.networks.remove(id)
    }
}

impl Default for WiFiNetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_network() {
        let network = WiFiNetwork::new(
            "Test Network".to_string(),
            "TestSSID".to_string(),
            "password123".to_string(),
            Some("Test description".to_string()),
        );

        assert_eq!(network.name, "Test Network");
        assert_eq!(network.ssid, "TestSSID");
        assert_eq!(network.password, "password123");
        assert_eq!(network.description, Some("Test description".to_string()));
        assert!(network.is_active);
        assert!(!network.id.is_empty());
    }
}
