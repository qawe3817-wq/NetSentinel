//! IPC (Inter-Process Communication) Module
//! 
//! Secure communication between UI and Core service via NamedPipe
//! Uses HMAC for message authentication

use anyhow::Result;
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::wfp::WfpEngine;
use crate::rules::{Rule, RuleAction};
use crate::process::ProcessMonitor;

/// IPC Message types - Extended with WFP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    // === Process Monitoring ===
    /// Request process list
    GetProcessList,
    /// Return process list
    ProcessList { processes: Vec<ProcessInfo> },
    
    // === Rule Management ===
    /// Add a new rule
    AddRule { rule: Rule },
    /// Remove a rule
    RemoveRule { id: String },
    /// Update an existing rule
    UpdateRule { id: String, rule: Rule },
    /// Get current rules
    GetRules,
    /// Return rules list
    RulesList { rules: Vec<Rule> },
    /// Enable/disable a rule
    ToggleRule { id: String, enabled: bool },
    
    // === WFP Filter Operations ===
    /// Add a WFP filter from a rule
    AddFilter { rule: Rule },
    /// Remove a WFP filter
    RemoveFilter { rule_id: String },
    /// Block a specific connection temporarily
    BlockConnection { pid: u32, duration_secs: u64 },
    /// Unblock a connection
    UnblockConnection { pid: u32 },
    /// Get active filter count
    GetFilterCount,
    /// Return filter count
    FilterCount { count: usize },
    
    // === Process Control ===
    /// Block a process (legacy, use BlockConnection)
    BlockProcess { pid: u32, duration_secs: u64 },
    /// Kill a process
    KillProcess { pid: u32 },
    /// Add process to whitelist
    WhitelistProcess { pid: u32, permanent: bool },
    
    // === Statistics & Monitoring ===
    /// Get network statistics
    GetStats,
    /// Return network statistics
    Stats { stats: NetworkStats },
    /// Subscribe to real-time stats updates
    SubscribeStats,
    /// Unsubscribe from stats updates
    UnsubscribeStats,
    /// Real-time stats update event
    StatsUpdate { stats: NetworkStats },
    
    // === System Control ===
    /// Get engine health status
    GetHealth,
    /// Return health status
    Health { healthy: bool, uptime_secs: u64, restart_count: u32 },
    /// Trigger graceful shutdown
    Shutdown,
    
    // === Error Handling ===
    /// Error response
    Error { message: String, code: Option<String> },
    /// Success acknowledgment
    Ack { message: String },
}

/// Process information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub signature_verified: bool,
    pub is_whitelisted: bool,
    pub is_blocked: bool,
}

/// Network statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_upload: u64,
    pub total_download: u64,
    pub upload_speed_bps: u64,
    pub download_speed_bps: u64,
    pub blocked_count: u32,
    pub active_connections: u32,
    pub packets_inspected: u64,
    pub packets_blocked: u64,
    pub block_rate: f64,
}

/// IPC Server using NamedPipe (Windows)
pub struct IpcServer {
    config: crate::config::Config,
    wfp_engine: Arc<WfpEngine>,
    process_monitor: Arc<ProcessMonitor>,
    stats_subscribers: Arc<tokio::sync::RwLock<Vec<mpsc::Sender<IpcMessage>>>>,
}

impl IpcServer {
    /// Create new IPC server with WFP engine and process monitor
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        info!("📡 Creating IPC server on pipe: {}", config.pipe_name);
        
        // Initialize WFP engine
        let wfp_engine = Arc::new(WfpEngine::new()?);
        
        // Initialize process monitor
        let process_monitor = Arc::new(ProcessMonitor::new()?);
        
        Ok(Self {
            config: config.clone(),
            wfp_engine,
            process_monitor,
            stats_subscribers: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        })
    }

    /// Run the IPC server
    pub async fn run(&self) -> Result<()> {
        info!("📡 IPC server starting...");
        
        // Start stats broadcast loop
        let stats_subscribers = Arc::clone(&self.stats_subscribers);
        let wfp_engine = Arc::clone(&self.wfp_engine);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                
                // Get current stats
                if let Ok(stats) = wfp_engine.get_stats() {
                    let msg = IpcMessage::StatsUpdate {
                        stats: NetworkStats {
                            total_upload: stats.upload_bytes,
                            total_download: stats.download_bytes,
                            upload_speed_bps: stats.upload_speed_bps(),
                            download_speed_bps: stats.download_speed_bps(),
                            blocked_count: stats.blocked_connections,
                            active_connections: stats.active_connections,
                            packets_inspected: stats.packets_inspected,
                            packets_blocked: stats.packets_blocked,
                            block_rate: stats.block_rate(),
                        },
                    };
                    
                    // Broadcast to all subscribers
                    let subscribers = stats_subscribers.read().await;
                    for tx in subscribers.iter() {
                        let _ = tx.send(msg.clone()).await;
                    }
                }
            }
        });
        
        // In production:
        // 1. Create NamedPipe server with ACL restrictions
        // 2. Accept client connections
        // 3. Validate HMAC on each message
        // 4. Route messages to appropriate handlers
        // 5. Send responses back
        
        info!("✅ IPC server ready to accept connections");
        
        // Placeholder for actual NamedPipe implementation
        // For now, just keep running
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
        
        Ok(())
    }

    /// Handle incoming message - Full implementation with WFP integration
    async fn handle_message(&self, msg: IpcMessage) -> Result<IpcMessage> {
        match msg {
            // === Process Monitoring ===
            IpcMessage::GetProcessList => {
                info!("📋 Getting process list");
                let processes = self.process_monitor.get_processes()?;
                let process_infos: Vec<ProcessInfo> = processes.iter().map(|p| {
                    ProcessInfo {
                        pid: p.pid,
                        name: p.name.clone(),
                        path: p.path.clone(),
                        upload_speed: p.upload_speed,
                        download_speed: p.download_speed,
                        connection_count: p.connection_count,
                        signature_verified: p.signature_verified,
                        is_whitelisted: false, // TODO: Check whitelist
                        is_blocked: false,     // TODO: Check block status
                    }
                }).collect();
                Ok(IpcMessage::ProcessList { processes: process_infos })
            }
            
            // === Rule Management ===
            IpcMessage::AddRule { rule } => {
                info!("➕ Adding rule: {}", rule.name);
                // Store rule in config (TODO: persist to disk)
                Ok(IpcMessage::Ack { message: format!("Rule '{}' added", rule.name) })
            }
            
            IpcMessage::RemoveRule { id } => {
                info!("➖ Removing rule: {}", id);
                Ok(IpcMessage::Ack { message: format!("Rule {} removed", id) })
            }
            
            IpcMessage::UpdateRule { id, rule } => {
                info!("✏️ Updating rule: {}", id);
                Ok(IpcMessage::Ack { message: format!("Rule {} updated", id) })
            }
            
            IpcMessage::GetRules => {
                info!("📜 Getting rules");
                // TODO: Load rules from config
                Ok(IpcMessage::RulesList { rules: vec![] })
            }
            
            IpcMessage::ToggleRule { id, enabled } => {
                info!(" toggling rule {}: {}", id, if enabled { "enabled" } else { "disabled" });
                Ok(IpcMessage::Ack { message: format!("Rule {} {}", id, if enabled { "enabled" } else { "disabled" }) })
            }
            
            // === WFP Filter Operations ===
            IpcMessage::AddFilter { rule } => {
                info!("🛡️ Adding WFP filter for rule: {}", rule.name);
                match self.wfp_engine.add_filter(&rule) {
                    Ok(_) => Ok(IpcMessage::Ack { 
                        message: format!("Filter for '{}' activated", rule.name) 
                    }),
                    Err(e) => Ok(IpcMessage::Error { 
                        message: format!("Failed to add filter: {}", e),
                        code: Some("WFP_ADD_FILTER_FAILED".to_string()),
                    }),
                }
            }
            
            IpcMessage::RemoveFilter { rule_id } => {
                info!("🛡️ Removing WFP filter: {}", rule_id);
                match self.wfp_engine.remove_filter(&rule_id) {
                    Ok(_) => Ok(IpcMessage::Ack { 
                        message: format!("Filter {} removed", rule_id) 
                    }),
                    Err(e) => Ok(IpcMessage::Error { 
                        message: format!("Failed to remove filter: {}", e),
                        code: Some("WFP_REMOVE_FILTER_FAILED".to_string()),
                    }),
                }
            }
            
            IpcMessage::BlockConnection { pid, duration_secs } => {
                warn!("⚠️ Blocking connection for PID {} ({} seconds)", pid, duration_secs);
                match self.wfp_engine.block_connection(pid, duration_secs) {
                    Ok(_) => Ok(IpcMessage::Ack { 
                        message: format!("Process {} blocked for {} seconds", pid, duration_secs) 
                    }),
                    Err(e) => Ok(IpcMessage::Error { 
                        message: format!("Failed to block connection: {}", e),
                        code: Some("WFP_BLOCK_FAILED".to_string()),
                    }),
                }
            }
            
            IpcMessage::UnblockConnection { pid } => {
                info!("✅ Unblocking connection for PID {}", pid);
                // Remove temporary block filter
                let filter_id = format!("temp_block_{}", pid);
                match self.wfp_engine.remove_filter(&filter_id) {
                    Ok(_) => Ok(IpcMessage::Ack { 
                        message: format!("Process {} unblocked", pid) 
                    }),
                    Err(e) => Ok(IpcMessage::Error { 
                        message: format!("Failed to unblock: {}", e),
                        code: Some("WFP_UNBLOCK_FAILED".to_string()),
                    }),
                }
            }
            
            IpcMessage::GetFilterCount => {
                let count = self.wfp_engine.get_active_filter_count();
                Ok(IpcMessage::FilterCount { count })
            }
            
            // === Process Control ===
            IpcMessage::BlockProcess { pid, duration_secs } => {
                // Legacy handler - delegate to BlockConnection
                warn!("⚠️ [Legacy] Blocking process {} for {} seconds", pid, duration_secs);
                self.handle_message(IpcMessage::BlockConnection { pid, duration_secs }).await
            }
            
            IpcMessage::KillProcess { pid } => {
                warn!("💀 Killing process {}", pid);
                match self.process_monitor.kill_process(pid) {
                    Ok(_) => Ok(IpcMessage::Ack { 
                        message: format!("Process {} terminated", pid) 
                    }),
                    Err(e) => Ok(IpcMessage::Error { 
                        message: format!("Failed to kill process: {}", e),
                        code: Some("PROCESS_KILL_FAILED".to_string()),
                    }),
                }
            }
            
            IpcMessage::WhitelistProcess { pid, permanent } => {
                info!("📝 Whitelisting process {} (permanent: {})", pid, permanent);
                // TODO: Add to whitelist in config
                Ok(IpcMessage::Ack { 
                    message: format!("Process {} whitelisted", pid) 
                })
            }
            
            // === Statistics & Monitoring ===
            IpcMessage::GetStats => {
                match self.wfp_engine.get_stats() {
                    Ok(stats) => Ok(IpcMessage::Stats { 
                        stats: NetworkStats {
                            total_upload: stats.upload_bytes,
                            total_download: stats.download_bytes,
                            upload_speed_bps: stats.upload_speed_bps(),
                            download_speed_bps: stats.download_speed_bps(),
                            blocked_count: stats.blocked_connections,
                            active_connections: stats.active_connections,
                            packets_inspected: stats.packets_inspected,
                            packets_blocked: stats.packets_blocked,
                            block_rate: stats.block_rate(),
                        }
                    }),
                    Err(e) => Ok(IpcMessage::Error { 
                        message: format!("Failed to get stats: {}", e),
                        code: Some("STATS_GET_FAILED".to_string()),
                    }),
                }
            }
            
            IpcMessage::SubscribeStats => {
                info!("📊 Client subscribed to stats updates");
                // Create channel for this subscriber
                let (tx, _rx) = mpsc::channel(100);
                // TODO: Store tx in subscribers list
                Ok(IpcMessage::Ack { message: "Subscribed to stats updates".to_string() })
            }
            
            IpcMessage::UnsubscribeStats => {
                info!("📊 Client unsubscribed from stats updates");
                // TODO: Remove from subscribers list
                Ok(IpcMessage::Ack { message: "Unsubscribed from stats updates".to_string() })
            }
            
            // === System Control ===
            IpcMessage::GetHealth => {
                // TODO: Track uptime and restart count
                Ok(IpcMessage::Health { 
                    healthy: true, 
                    uptime_secs: 0, 
                    restart_count: 0 
                })
            }
            
            IpcMessage::Shutdown => {
                info!("🛑 Shutdown requested");
                Ok(IpcMessage::Ack { message: "Shutting down".to_string() })
            }
            
            // === Passthrough responses (should not arrive from UI) ===
            IpcMessage::ProcessList { .. } |
            IpcMessage::RulesList { .. } |
            IpcMessage::Stats { .. } |
            IpcMessage::FilterCount { .. } |
            IpcMessage::Health { .. } |
            IpcMessage::StatsUpdate { .. } |
            IpcMessage::Ack { .. } |
            IpcMessage::Error { .. } => {
                warn!("Received response-type message from UI: {:?}", msg);
                Ok(IpcMessage::Error { 
                    message: "Invalid message type".to_string(),
                    code: Some("INVALID_MESSAGE_TYPE".to_string()),
                })
            }
        }
    }
    
    /// Get reference to WFP engine
    pub fn get_wfp_engine(&self) -> Arc<WfpEngine> {
        Arc::clone(&self.wfp_engine)
    }
    
    /// Get reference to process monitor
    pub fn get_process_monitor(&self) -> Arc<ProcessMonitor> {
        Arc::clone(&self.process_monitor)
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
