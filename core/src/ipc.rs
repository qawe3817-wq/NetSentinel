//! IPC (Inter-Process Communication) Module
//! 
//! Secure communication between UI and Core service via NamedPipe
//! Uses HMAC for message authentication

use anyhow::Result;
use tracing::{info, warn};
use serde::{Serialize, Deserialize};

/// IPC Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    /// Request process list
    GetProcessList,
    /// Return process list
    ProcessList { processes: Vec<ProcessInfo> },
    /// Add a new rule
    AddRule { rule: crate::rules::Rule },
    /// Remove a rule
    RemoveRule { id: String },
    /// Get current rules
    GetRules,
    /// Return rules list
    RulesList { rules: Vec<crate::rules::Rule> },
    /// Block a process
    BlockProcess { pid: u32, duration_secs: u64 },
    /// Get network statistics
    GetStats,
    /// Return network statistics
    Stats { stats: NetworkStats },
    /// Error response
    Error { message: String },
}

/// Process information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub signature_verified: bool,
}

/// Network statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_upload: u64,
    pub total_download: u64,
    pub blocked_count: u32,
}

/// IPC Server using NamedPipe (Windows)
pub struct IpcServer {
    _config: crate::config::Config,
}

impl IpcServer {
    /// Create new IPC server
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        info!("Creating IPC server on pipe: {}", config.pipe_name);
        Ok(Self { _config: config.clone() })
    }

    /// Run the IPC server
    pub async fn run(&self) -> Result<()> {
        info!("IPC server starting...");
        
        // In production:
        // 1. Create NamedPipe server with ACL restrictions
        // 2. Accept client connections
        // 3. Validate HMAC on each message
        // 4. Route messages to appropriate handlers
        // 5. Send responses back
        
        // Placeholder for actual implementation
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
        
        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&self, msg: IpcMessage) -> Result<IpcMessage> {
        match msg {
            IpcMessage::GetProcessList => {
                Ok(IpcMessage::ProcessList { processes: vec![] })
            }
            IpcMessage::GetRules => {
                Ok(IpcMessage::RulesList { rules: vec![] })
            }
            IpcMessage::GetStats => {
                Ok(IpcMessage::Stats { stats: NetworkStats::default() })
            }
            IpcMessage::BlockProcess { pid, duration_secs } => {
                info!("Blocking process {} for {} seconds", pid, duration_secs);
                Ok(IpcMessage::Stats { stats: NetworkStats::default() })
            }
            _ => {
                warn!("Unhandled message type");
                Ok(IpcMessage::Error { message: "Not implemented".to_string() })
            }
        }
    }
}

/// HMAC validation for IPC security
pub mod security {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    /// Generate HMAC for message authentication
    pub fn generate_hmac(key: &[u8], message: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(message);
        mac.finalize().into_bytes().to_vec()
    }

    /// Verify HMAC signature
    pub fn verify_hmac(key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(message);
        mac.verify_slice(signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::security::*;

    #[test]
    fn test_hmac_roundtrip() {
        let key = b"test-secret-key";
        let message = b"test message";
        
        let signature = generate_hmac(key, message);
        assert!(verify_hmac(key, message, &signature));
        assert!(!verify_hmac(key, b"wrong message", &signature));
    }
}
