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

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn update_credentials(&mut self, ssid: String, password: String) {
        self.ssid = ssid;
        self.password = password;
    }

    pub fn update_info(&mut self, name: String, description: Option<String>) {
        self.name = name;
        self.description = description;
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

    pub fn get_network_mut(&mut self, id: &str) -> Option<&mut WiFiNetwork> {
        self.networks.get_mut(id)
    }

    pub fn get_network_by_ssid(&self, ssid: &str) -> Option<&WiFiNetwork> {
        self.networks.values().find(|n| n.ssid == ssid)
    }

    pub fn get_all_networks(&self) -> Vec<&WiFiNetwork> {
        let mut networks: Vec<&WiFiNetwork> = self.networks.values().collect();
        networks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        networks
    }

    pub fn get_active_networks(&self) -> Vec<&WiFiNetwork> {
        let mut networks: Vec<&WiFiNetwork> = self.networks.values()
            .filter(|n| n.is_active)
            .collect();
        networks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        networks
    }

    pub fn remove_network(&mut self, id: &str) -> Option<WiFiNetwork> {
        self.networks.remove(id)
    }

    pub fn deactivate_network(&mut self, id: &str) -> Result<(), NetworkError> {
        match self.networks.get_mut(id) {
            Some(network) => {
                network.deactivate();
                Ok(())
            }
            None => Err(NetworkError::NotFound),
        }
    }

    pub fn activate_network(&mut self, id: &str) -> Result<(), NetworkError> {
        match self.networks.get_mut(id) {
            Some(network) => {
                network.activate();
                Ok(())
            }
            None => Err(NetworkError::NotFound),
        }
    }

    pub fn update_network_credentials(&mut self, id: &str, ssid: String, password: String) -> Result<(), NetworkError> {
        match self.networks.get_mut(id) {
            Some(network) => {
                network.update_credentials(ssid, password);
                Ok(())
            }
            None => Err(NetworkError::NotFound),
        }
    }

    pub fn update_network_info(&mut self, id: &str, name: String, description: Option<String>) -> Result<(), NetworkError> {
        match self.networks.get_mut(id) {
            Some(network) => {
                network.update_info(name, description);
                Ok(())
            }
            None => Err(NetworkError::NotFound),
        }
    }

    pub fn network_count(&self) -> usize {
        self.networks.len()
    }

    pub fn active_network_count(&self) -> usize {
        self.networks.values().filter(|n| n.is_active).count()
    }

    pub fn clear_all_networks(&mut self) {
        self.networks.clear();
    }

    pub fn network_exists(&self, id: &str) -> bool {
        self.networks.contains_key(id)
    }

    pub fn ssid_exists(&self, ssid: &str) -> bool {
        self.networks.values().any(|n| n.ssid == ssid)
    }

    pub fn validate_network_credentials(&self, id: &str, password: &str) -> bool {
        match self.networks.get(id) {
            Some(network) => network.password == password && network.is_active,
            None => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Network not found")]
    NotFound,
    
    #[error("Network already exists")]
    AlreadyExists,
    
    #[error("Invalid network credentials")]
    InvalidCredentials,
    
    #[error("Network is not active")]
    NotActive,
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

    #[test]
    fn test_network_manager() {
        let mut manager = WiFiNetworkManager::new();
        assert_eq!(manager.network_count(), 0);

        let network = WiFiNetwork::new(
            "Test Network".to_string(),
            "TestSSID".to_string(),
            "password123".to_string(),
            None,
        );
        
        manager.add_network(network);
        assert_eq!(manager.network_count(), 1);
        assert_eq!(manager.active_network_count(), 1);
    }

    #[test]
    fn test_find_network_by_ssid() {
        let mut manager = WiFiNetworkManager::new();
        let network = WiFiNetwork::new(
            "Test Network".to_string(),
            "TestSSID".to_string(),
            "password123".to_string(),
            None,
        );
        
        manager.add_network(network);

        let found = manager.get_network_by_ssid("TestSSID");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Network");

        let not_found = manager.get_network_by_ssid("NonExistentSSID");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_deactivate_network() {
        let mut manager = WiFiNetworkManager::new();
        let network = WiFiNetwork::new(
            "Test Network".to_string(),
            "TestSSID".to_string(),
            "password123".to_string(),
            None,
        );
        
        let network_id = network.id.clone();
        manager.add_network(network);
        
        assert_eq!(manager.active_network_count(), 1);
        
        let result = manager.deactivate_network(&network_id);
        assert!(result.is_ok());
        assert_eq!(manager.active_network_count(), 0);
        assert_eq!(manager.network_count(), 1);
    }

    #[test]
    fn test_update_network_credentials() {
        let mut manager = WiFiNetworkManager::new();
        let network = WiFiNetwork::new(
            "Test Network".to_string(),
            "TestSSID".to_string(),
            "password123".to_string(),
            None,
        );
        
        let network_id = network.id.clone();
        manager.add_network(network);
        
        let result = manager.update_network_credentials(
            &network_id,
            "NewSSID".to_string(),
            "newpassword".to_string(),
        );
        
        assert!(result.is_ok());
        
        let updated_network = manager.get_network(&network_id).unwrap();
        assert_eq!(updated_network.ssid, "NewSSID");
        assert_eq!(updated_network.password, "newpassword");
    }

    #[test]
    fn test_validate_network_credentials() {
        let mut manager = WiFiNetworkManager::new();
        let network = WiFiNetwork::new(
            "Test Network".to_string(),
            "TestSSID".to_string(),
            "password123".to_string(),
            None,
        );
        
        let network_id = network.id.clone();
        manager.add_network(network);
        
        assert!(manager.validate_network_credentials(&network_id, "password123"));
        assert!(!manager.validate_network_credentials(&network_id, "wrongpassword"));
        
        // Deactivate network and test again
        manager.deactivate_network(&network_id).unwrap();
        assert!(!manager.validate_network_credentials(&network_id, "password123"));
    }

    #[test]
    fn test_multiple_networks() {
        let mut manager = WiFiNetworkManager::new();
        
        let network1 = WiFiNetwork::new(
            "Network 1".to_string(),
            "SSID1".to_string(),
            "pass1".to_string(),
            None,
        );
        
        let network2 = WiFiNetwork::new(
            "Network 2".to_string(),
            "SSID2".to_string(),
            "pass2".to_string(),
            None,
        );
        
        manager.add_network(network1);
        manager.add_network(network2);
        
        assert_eq!(manager.network_count(), 2);
        assert_eq!(manager.active_network_count(), 2);
        
        let networks = manager.get_all_networks();
        assert_eq!(networks.len(), 2);
        
        assert!(manager.ssid_exists("SSID1"));
        assert!(manager.ssid_exists("SSID2"));
        assert!(!manager.ssid_exists("SSID3"));
    }
}