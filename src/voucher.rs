use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    pub id: String,
    pub code: String,
    pub network_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_used: bool,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Voucher {
    pub fn new(code: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            code,
            network_id: None,
            created_at: chrono::Utc::now(),
            is_used: false,
            used_at: None,
        }
    }

    pub fn new_with_network(code: String, network_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            code,
            network_id: Some(network_id),
            created_at: chrono::Utc::now(),
            is_used: false,
            used_at: None,
        }
    }

    pub fn assign_to_network(&mut self, network_id: String) {
        self.network_id = Some(network_id);
    }

    pub fn remove_from_network(&mut self) {
        self.network_id = None;
    }

    pub fn mark_as_used(&mut self) {
        self.is_used = true;
        self.used_at = Some(chrono::Utc::now());
    }
}

#[derive(Debug)]
pub struct VoucherManager {
    vouchers: HashMap<String, Voucher>,
}

impl VoucherManager {
    pub fn new() -> Self {
        Self {
            vouchers: HashMap::new(),
        }
    }

    pub fn add_voucher(&mut self, voucher: Voucher) {
        self.vouchers.insert(voucher.id.clone(), voucher);
    }

    pub fn add_vouchers(&mut self, vouchers: Vec<Voucher>) {
        for voucher in vouchers {
            self.add_voucher(voucher);
        }
    }

    pub fn get_voucher(&self, id: &str) -> Option<&Voucher> {
        self.vouchers.get(id)
    }

    pub fn get_voucher_by_code(&self, code: &str) -> Option<&Voucher> {
        self.vouchers.values().find(|v| v.code == code)
    }

    pub fn get_all_vouchers(&self) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self.vouchers.values().collect();
        vouchers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        vouchers
    }

    pub fn get_unused_vouchers(&self) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self.vouchers.values()
            .filter(|v| !v.is_used)
            .collect();
        vouchers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        vouchers
    }

    pub fn get_used_vouchers(&self) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self.vouchers.values()
            .filter(|v| v.is_used)
            .collect();
        vouchers.sort_by(|a, b| a.used_at.cmp(&b.used_at));
        vouchers
    }

    pub fn mark_voucher_as_used(&mut self, id: &str) -> Result<(), VoucherError> {
        match self.vouchers.get_mut(id) {
            Some(voucher) => {
                if voucher.is_used {
                    Err(VoucherError::AlreadyUsed)
                } else {
                    voucher.mark_as_used();
                    Ok(())
                }
            }
            None => Err(VoucherError::NotFound),
        }
    }

    pub fn mark_voucher_as_used_by_code(&mut self, code: &str) -> Result<(), VoucherError> {
        let voucher_id = self.vouchers.values()
            .find(|v| v.code == code)
            .map(|v| v.id.clone());

        match voucher_id {
            Some(id) => self.mark_voucher_as_used(&id),
            None => Err(VoucherError::NotFound),
        }
    }

    pub fn voucher_count(&self) -> usize {
        self.vouchers.len()
    }

    pub fn unused_voucher_count(&self) -> usize {
        self.vouchers.values().filter(|v| !v.is_used).count()
    }

    pub fn used_voucher_count(&self) -> usize {
        self.vouchers.values().filter(|v| v.is_used).count()
    }

    pub fn clear_all_vouchers(&mut self) {
        self.vouchers.clear();
    }

    pub fn remove_voucher(&mut self, id: &str) -> Option<Voucher> {
        self.vouchers.remove(id)
    }

    pub fn remove_voucher_by_code(&mut self, code: &str) -> Option<Voucher> {
        let voucher_id = self.vouchers.values()
            .find(|v| v.code == code)
            .map(|v| v.id.clone());

        match voucher_id {
            Some(id) => self.remove_voucher(&id),
            None => None,
        }
    }

    pub fn validate_voucher_code(&self, code: &str) -> bool {
        self.vouchers.values().any(|v| v.code == code && !v.is_used)
    }

    pub fn get_vouchers_for_network(&self, network_id: &str) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self.vouchers.values()
            .filter(|v| v.network_id.as_deref() == Some(network_id))
            .collect();
        vouchers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        vouchers
    }

    pub fn get_unused_vouchers_for_network(&self, network_id: &str) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self.vouchers.values()
            .filter(|v| v.network_id.as_deref() == Some(network_id) && !v.is_used)
            .collect();
        vouchers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        vouchers
    }

    pub fn remove_vouchers_for_network(&mut self, network_id: &str) {
        self.vouchers.retain(|_, v| v.network_id.as_deref() != Some(network_id));
    }

    pub fn voucher_count_for_network(&self, network_id: &str) -> usize {
        self.vouchers.values()
            .filter(|v| v.network_id.as_deref() == Some(network_id))
            .count()
    }

    pub fn unused_voucher_count_for_network(&self, network_id: &str) -> usize {
        self.vouchers.values()
            .filter(|v| v.network_id.as_deref() == Some(network_id) && !v.is_used)
            .count()
    }

    pub fn assign_vouchers_to_network(&mut self, voucher_ids: Vec<String>, network_id: String) {
        for id in voucher_ids {
            if let Some(voucher) = self.vouchers.get_mut(&id) {
                voucher.assign_to_network(network_id.clone());
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoucherError {
    #[error("Voucher not found")]
    NotFound,
    
    #[error("Voucher has already been used")]
    AlreadyUsed,
    
    #[error("Invalid voucher code")]
    InvalidCode,
}

impl Default for VoucherManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_voucher() {
        let voucher = Voucher::new("TEST123".to_string());
        assert_eq!(voucher.code, "TEST123");
        assert!(!voucher.is_used);
        assert!(voucher.used_at.is_none());
        assert!(voucher.network_id.is_none());
    }

    #[test]
    fn test_create_voucher_with_network() {
        let voucher = Voucher::new_with_network("TEST123".to_string(), "network123".to_string());
        assert_eq!(voucher.code, "TEST123");
        assert_eq!(voucher.network_id, Some("network123".to_string()));
        assert!(!voucher.is_used);
    }

    #[test]
    fn test_mark_voucher_as_used() {
        let mut voucher = Voucher::new("TEST123".to_string());
        assert!(!voucher.is_used);
        
        voucher.mark_as_used();
        assert!(voucher.is_used);
        assert!(voucher.used_at.is_some());
    }

    #[test]
    fn test_voucher_manager() {
        let mut manager = VoucherManager::new();
        assert_eq!(manager.voucher_count(), 0);

        let voucher = Voucher::new("TEST123".to_string());
        manager.add_voucher(voucher);
        assert_eq!(manager.voucher_count(), 1);
        assert_eq!(manager.unused_voucher_count(), 1);
        assert_eq!(manager.used_voucher_count(), 0);
    }

    #[test]
    fn test_find_voucher_by_code() {
        let mut manager = VoucherManager::new();
        let voucher = Voucher::new("TEST123".to_string());
        manager.add_voucher(voucher);

        let found = manager.get_voucher_by_code("TEST123");
        assert!(found.is_some());
        assert_eq!(found.unwrap().code, "TEST123");

        let not_found = manager.get_voucher_by_code("NOTFOUND");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_mark_voucher_as_used_by_code() {
        let mut manager = VoucherManager::new();
        let voucher = Voucher::new("TEST123".to_string());
        manager.add_voucher(voucher);

        assert!(manager.validate_voucher_code("TEST123"));
        
        let result = manager.mark_voucher_as_used_by_code("TEST123");
        assert!(result.is_ok());
        assert!(!manager.validate_voucher_code("TEST123"));
        assert_eq!(manager.used_voucher_count(), 1);
        assert_eq!(manager.unused_voucher_count(), 0);
    }

    #[test]
    fn test_add_multiple_vouchers() {
        let mut manager = VoucherManager::new();
        let vouchers = vec![
            Voucher::new("CODE1".to_string()),
            Voucher::new("CODE2".to_string()),
            Voucher::new("CODE3".to_string()),
        ];

        manager.add_vouchers(vouchers);
        assert_eq!(manager.voucher_count(), 3);
        assert!(manager.validate_voucher_code("CODE1"));
        assert!(manager.validate_voucher_code("CODE2"));
        assert!(manager.validate_voucher_code("CODE3"));
    }

    #[test]
    fn test_network_voucher_management() {
        let mut manager = VoucherManager::new();
        let voucher1 = Voucher::new_with_network("NET1-CODE1".to_string(), "network1".to_string());
        let voucher2 = Voucher::new_with_network("NET1-CODE2".to_string(), "network1".to_string());
        let voucher3 = Voucher::new_with_network("NET2-CODE1".to_string(), "network2".to_string());

        manager.add_voucher(voucher1);
        manager.add_voucher(voucher2);
        manager.add_voucher(voucher3);

        assert_eq!(manager.voucher_count(), 3);
        assert_eq!(manager.voucher_count_for_network("network1"), 2);
        assert_eq!(manager.voucher_count_for_network("network2"), 1);

        let network1_vouchers = manager.get_vouchers_for_network("network1");
        assert_eq!(network1_vouchers.len(), 2);

        manager.remove_vouchers_for_network("network1");
        assert_eq!(manager.voucher_count(), 1);
        assert_eq!(manager.voucher_count_for_network("network2"), 1);
    }
}