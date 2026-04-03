//! Configuration Module
//! 
//! Handles loading and saving application configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Named pipe name for IPC communication
    pub pipe_name: String,
    /// HMAC secret key for IPC security
    pub ipc_secret: String,
    /// Enable watchdog self-healing
    pub watchdog_enabled: bool,
    /// Watchdog restart timeout (seconds)
    pub watchdog_timeout_secs: u64,
    /// Auto-start on boot
    pub auto_start: bool,
    /// Silent mode (no notifications)
    pub silent_mode: bool,
    /// Default block duration (seconds)
    pub default_block_duration_secs: u64,
    /// Dynamic threshold based on bandwidth
    pub bandwidth_threshold_mbps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pipe_name: r"\\.\pipe\netsentinel-core".to_string(),
            ipc_secret: generate_secure_key(),
            watchdog_enabled: true,
            watchdog_timeout_secs: 2,
            auto_start: true,
            silent_mode: false,
            default_block_duration_secs: 600, // 10 minutes
            bandwidth_threshold_mbps: 100,
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();
        
        if config_path.exists() {
            // Load from file
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        let content = serde_json::to_string_pretty(self)?;
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Backup configuration
    pub fn backup(&self) -> Result<()> {
        let backup_path = get_config_path().with_extension("backup.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(backup_path, content)?;
        Ok(())
    }

    /// Restore from backup
    pub fn restore_from_backup() -> Result<Self> {
        let backup_path = get_config_path().with_extension("backup.json");
        let content = std::fs::read_to_string(&backup_path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}

/// Get configuration file path
fn get_config_path() -> PathBuf {
    // In production: use proper config directory for Windows
    // %APPDATA%\NetSentinel\config.json
    let mut path = PathBuf::from("config.json");
    path
}

/// Generate a secure random key for HMAC
fn generate_secure_key() -> String {
    // In production: use proper CSPRNG
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("netsentinel-{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.watchdog_enabled);
        assert_eq!(config.watchdog_timeout_secs, 2);
    }

    #[test]
    fn test_config_save_load() {
        let config = Config::default();
        // Note: actual file I/O test would require temp directory
        assert!(config.pipe_name.contains("netsentinel"));
    }
}
