use serde::{Deserialize, Serialize};
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
