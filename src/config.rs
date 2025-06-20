use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use anyhow::{Result, Context};

/// Configuration structure that maps to the config.toml file
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Path to the templates directory (relative to project root)
    pub templates_dir: String,
    
    /// Path for the database file (relative to project root)
    pub database_path: String,
    
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
}

/// Server-specific configuration
#[derive(Debug, Deserialize, Default)]
pub struct ServerConfig {
    /// Default host address
    #[serde(default = "default_host")]
    pub default_host: String,
    
    /// Default port number
    #[serde(default = "default_port")]
    pub default_port: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

impl Config {
    /// Load configuration from config.toml in the current directory or specified path
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_file()?;
        let config_content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
        
        let config: Config = toml::from_str(&config_content)
            .with_context(|| "Failed to parse config.toml")?;
        
        Ok(config)
    }
    
    /// Find the config.toml file, first checking the current directory
    fn find_config_file() -> Result<PathBuf> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let config_path = current_dir.join("config.toml");
        
        if config_path.exists() {
            return Ok(config_path);
        }
        
        // Check if there's an environment variable specifying the config path
        if let Ok(env_path) = env::var("VOUCHER_CONFIG_PATH") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                return Ok(path);
            }
        }
        
        // If we get here, use the default config in the current directory
        // This will likely fail, but we'll provide a better error message
        Ok(config_path)
    }
    
    /// Get the absolute path to the templates directory
    pub fn templates_dir_path(&self) -> Result<PathBuf> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        Ok(current_dir.join(&self.templates_dir))
    }
    
    /// Get the absolute path to the database file
    pub fn database_file_path(&self) -> Result<PathBuf> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        Ok(current_dir.join(&self.database_path))
    }
    
    /// Ensure all configured directories exist
    pub fn ensure_directories_exist(&self) -> Result<()> {
        // Ensure templates directory exists
        let templates_dir = self.templates_dir_path()?;
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)
                .with_context(|| format!("Failed to create templates directory: {}", templates_dir.display()))?;
            println!("Created templates directory: {}", templates_dir.display());
        }
        
        // Ensure database directory exists (if path contains directories)
        if let Some(parent) = Path::new(&self.database_path).parent() {
            if !parent.as_os_str().is_empty() {
                let db_parent_dir = env::current_dir()?.join(parent);
                if !db_parent_dir.exists() {
                    fs::create_dir_all(&db_parent_dir)
                        .with_context(|| format!("Failed to create database directory: {}", db_parent_dir.display()))?;
                    println!("Created database directory: {}", db_parent_dir.display());
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate a database URL for SQLx
    pub fn database_url(&self) -> Result<String> {
        let db_path = self.database_file_path()?;
        Ok(format!("sqlite:{}", db_path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[test]
    fn test_parse_config() {
        let config_content = r#"
            templates_dir = "custom_templates"
            database_path = "data/my_database.db"
            
            [server]
            default_host = "0.0.0.0"
            default_port = 8080
        "#;
        
        let config: Config = toml::from_str(config_content).unwrap();
        
        assert_eq!(config.templates_dir, "custom_templates");
        assert_eq!(config.database_path, "data/my_database.db");
        assert_eq!(config.server.default_host, "0.0.0.0");
        assert_eq!(config.server.default_port, 8080);
    }
    
    #[test]
    fn test_default_server_config() {
        let config_content = r#"
            templates_dir = "templates"
            database_path = "vouchers.db"
        "#;
        
        let config: Config = toml::from_str(config_content).unwrap();
        
        assert_eq!(config.server.default_host, "127.0.0.1");
        assert_eq!(config.server.default_port, 3000);
    }
    
    #[test]
    fn test_database_url() {
        let temp_dir = tempdir().unwrap();
        let current_dir = temp_dir.path();
        
        env::set_current_dir(current_dir).unwrap();
        
        let config = Config {
            templates_dir: "templates".to_string(),
            database_path: "data/app.db".to_string(),
            server: ServerConfig::default(),
        };
        
        let db_url = config.database_url().unwrap();
        assert!(db_url.contains("sqlite:"));
        assert!(db_url.contains("data/app.db"));
    }
}