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

    pub fn get_all_vouchers(&self) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self.vouchers.values().collect();
        vouchers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        vouchers
    }

    pub fn voucher_count(&self) -> usize {
        self.vouchers.len()
    }

    pub fn get_vouchers_for_network(&self, network_id: &str) -> Vec<&Voucher> {
        let mut vouchers: Vec<&Voucher> = self
            .vouchers
            .values()
            .filter(|v| v.network_id.as_deref() == Some(network_id))
            .collect();
        vouchers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        vouchers
    }

    pub fn remove_vouchers_for_network(&mut self, network_id: &str) {
        self.vouchers
            .retain(|_, v| v.network_id.as_deref() != Some(network_id));
    }

    pub fn voucher_count_for_network(&self, network_id: &str) -> usize {
        self.vouchers
            .values()
            .filter(|v| v.network_id.as_deref() == Some(network_id))
            .count()
    }

    pub fn unused_voucher_count_for_network(&self, network_id: &str) -> usize {
        self.vouchers
            .values()
            .filter(|v| v.network_id.as_deref() == Some(network_id) && !v.is_used)
            .count()
    }
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
}
