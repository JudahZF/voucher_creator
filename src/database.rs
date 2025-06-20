use crate::voucher::Voucher;
use crate::wifi_network::WiFiNetwork;
use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        let db = Self { pool };
        db.migrate().await?;

        Ok(db)
    }

    async fn migrate(&self) -> Result<()> {
        // Create wifi_networks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wifi_networks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                ssid TEXT NOT NULL,
                password TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create vouchers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vouchers (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL UNIQUE,
                network_id TEXT,
                created_at TEXT NOT NULL,
                is_used BOOLEAN NOT NULL DEFAULT FALSE,
                used_at TEXT,
                is_printed BOOLEAN NOT NULL DEFAULT FALSE,
                printed_at TEXT,
                FOREIGN KEY (network_id) REFERENCES wifi_networks (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add new columns if they don't exist (for existing databases)
        let _ = sqlx::query("ALTER TABLE vouchers ADD COLUMN is_printed BOOLEAN NOT NULL DEFAULT FALSE")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE vouchers ADD COLUMN printed_at TEXT")
            .execute(&self.pool)
            .await;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_vouchers_network_id ON vouchers(network_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_vouchers_is_used ON vouchers(is_used)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_vouchers_is_printed ON vouchers(is_printed)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // WiFi Network operations
    pub async fn create_network(&self, network: &WiFiNetwork) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO wifi_networks (id, name, ssid, password, description, created_at, is_active)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&network.id)
        .bind(&network.name)
        .bind(&network.ssid)
        .bind(&network.password)
        .bind(&network.description)
        .bind(network.created_at.to_rfc3339())
        .bind(network.is_active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_network(&self, id: &str) -> Result<Option<WiFiNetwork>> {
        let row = sqlx::query(
            "SELECT id, name, ssid, password, description, created_at, is_active FROM wifi_networks WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(WiFiNetwork {
                id: row.get("id"),
                name: row.get("name"),
                ssid: row.get("ssid"),
                password: row.get("password"),
                description: row.get("description"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .with_timezone(&chrono::Utc),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_networks(&self) -> Result<Vec<WiFiNetwork>> {
        let rows = sqlx::query(
            "SELECT id, name, ssid, password, description, created_at, is_active FROM wifi_networks ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut networks = Vec::new();
        for row in rows {
            networks.push(WiFiNetwork {
                id: row.get("id"),
                name: row.get("name"),
                ssid: row.get("ssid"),
                password: row.get("password"),
                description: row.get("description"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .with_timezone(&chrono::Utc),
                is_active: row.get("is_active"),
            });
        }

        Ok(networks)
    }

    pub async fn delete_network(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM wifi_networks WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_vouchers(&self, vouchers: &[Voucher]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for voucher in vouchers {
            sqlx::query(
                r#"
                INSERT INTO vouchers (id, code, network_id, created_at, is_used, used_at, is_printed, printed_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
            )
            .bind(&voucher.id)
            .bind(&voucher.code)
            .bind(&voucher.network_id)
            .bind(voucher.created_at.to_rfc3339())
            .bind(voucher.is_used)
            .bind(voucher.used_at.map(|dt| dt.to_rfc3339()))
            .bind(voucher.is_printed)
            .bind(voucher.printed_at.map(|dt| dt.to_rfc3339()))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_all_vouchers(&self) -> Result<Vec<Voucher>> {
        let rows = sqlx::query(
            "SELECT id, code, network_id, created_at, is_used, used_at, is_printed, printed_at FROM vouchers ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut vouchers = Vec::new();
        for row in rows {
            vouchers.push(Voucher {
                id: row.get("id"),
                code: row.get("code"),
                network_id: row.get("network_id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .with_timezone(&chrono::Utc),
                is_used: row.get("is_used"),
                used_at: row
                    .get::<Option<String>, _>("used_at")
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()?,
                is_printed: row.try_get("is_printed").unwrap_or(false),
                printed_at: row
                    .try_get::<Option<String>, _>("printed_at")
                    .unwrap_or(None)
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()?,
            });
        }

        Ok(vouchers)
    }

    pub async fn get_vouchers_for_network(&self, network_id: &str) -> Result<Vec<Voucher>> {
        let rows = sqlx::query(
            "SELECT id, code, network_id, created_at, is_used, used_at, is_printed, printed_at FROM vouchers WHERE network_id = ?1 ORDER BY created_at ASC"
        )
        .bind(network_id)
        .fetch_all(&self.pool)
        .await?;

        let mut vouchers = Vec::new();
        for row in rows {
            vouchers.push(Voucher {
                id: row.get("id"),
                code: row.get("code"),
                network_id: row.get("network_id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .with_timezone(&chrono::Utc),
                is_used: row.get("is_used"),
                used_at: row
                    .get::<Option<String>, _>("used_at")
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()?,
                is_printed: row.try_get("is_printed").unwrap_or(false),
                printed_at: row
                    .try_get::<Option<String>, _>("printed_at")
                    .unwrap_or(None)
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()?,
            });
        }

        Ok(vouchers)
    }

    pub async fn mark_voucher_as_used(&self, voucher_id: &str) -> Result<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query(
            "UPDATE vouchers SET is_used = TRUE, used_at = ?1 WHERE id = ?2 AND is_used = FALSE",
        )
        .bind(&now)
        .bind(voucher_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_voucher_as_unused(&self, voucher_id: &str) -> Result<bool> {
        let result =
            sqlx::query("UPDATE vouchers SET is_used = FALSE, used_at = NULL WHERE id = ?1")
                .bind(voucher_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_vouchers_as_printed(&self, voucher_ids: &[String]) -> Result<usize> {
        let mut tx = self.pool.begin().await?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut count = 0;

        for voucher_id in voucher_ids {
            let result = sqlx::query(
                "UPDATE vouchers SET is_printed = TRUE, printed_at = ?1 WHERE id = ?2 AND is_printed = FALSE",
            )
            .bind(&now)
            .bind(voucher_id)
            .execute(&mut *tx)
            .await?;
            
            count += result.rows_affected() as usize;
        }

        tx.commit().await?;
        Ok(count)
    }

    pub async fn get_unprinted_vouchers_for_network(&self, network_id: &str, limit: Option<usize>) -> Result<Vec<Voucher>> {
        let query = if let Some(limit) = limit {
            format!(
                "SELECT id, code, network_id, created_at, is_used, used_at, is_printed, printed_at FROM vouchers WHERE network_id = ?1 AND is_printed = FALSE ORDER BY created_at ASC LIMIT {}",
                limit
            )
        } else {
            "SELECT id, code, network_id, created_at, is_used, used_at, is_printed, printed_at FROM vouchers WHERE network_id = ?1 AND is_printed = FALSE ORDER BY created_at ASC".to_string()
        };

        let rows = sqlx::query(&query)
            .bind(network_id)
            .fetch_all(&self.pool)
            .await?;

        let mut vouchers = Vec::new();
        for row in rows {
            vouchers.push(Voucher {
                id: row.get("id"),
                code: row.get("code"),
                network_id: row.get("network_id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .with_timezone(&chrono::Utc),
                is_used: row.get("is_used"),
                used_at: row
                    .get::<Option<String>, _>("used_at")
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()?,
                is_printed: row.try_get("is_printed").unwrap_or(false),
                printed_at: row
                    .try_get::<Option<String>, _>("printed_at")
                    .unwrap_or(None)
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()?,
            });
        }

        Ok(vouchers)
    }

    pub async fn get_voucher_counts(&self, network_id: &str) -> Result<VoucherCounts> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(CASE WHEN is_used = TRUE THEN 1 END) as used,
                COUNT(CASE WHEN is_used = FALSE THEN 1 END) as unused,
                COUNT(CASE WHEN is_printed = TRUE THEN 1 END) as printed,
                COUNT(CASE WHEN is_printed = FALSE THEN 1 END) as unprinted
            FROM vouchers
            WHERE network_id = ?1
            "#,
        )
        .bind(network_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(VoucherCounts {
            total: row.get::<i64, _>("total") as usize,
            used: row.get::<i64, _>("used") as usize,
            unused: row.get::<i64, _>("unused") as usize,
            printed: row.get::<i64, _>("printed") as usize,
            unprinted: row.get::<i64, _>("unprinted") as usize,
        })
    }
}

#[derive(Debug)]
pub struct VoucherCounts {
    pub total: usize,
    pub used: usize,
    pub unused: usize,
    pub printed: usize,
    pub unprinted: usize,
}
