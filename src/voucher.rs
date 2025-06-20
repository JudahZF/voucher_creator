use serde::{Deserialize, Serialize};
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
