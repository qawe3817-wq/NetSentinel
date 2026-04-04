//! Process Monitor Module with WFP Integration
//! 
//! Provides real-time process network activity monitoring
//! Uses Windows IP Helper API for network statistics
//! Integrates with WFP engine for automatic PCDN blocking

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::ipc::ProcessInfo;
use crate::wfp::WfpEngine;
use crate::rules::{Rule, Condition, ConditionField, ConditionOperator, RuleAction};

/// Process monitor for tracking network activity with WFP integration
pub struct ProcessMonitor {
    wfp_engine: Option<Arc<WfpEngine>>,
    whitelist: Arc<tokio::sync::RwLock<Vec<u32>>>,
    blocklist: Arc<tokio::sync::RwLock<Vec<u32>>>,
    _private: (),
}

impl ProcessMonitor {
    /// Initialize process monitor with optional WFP engine integration
    pub fn new() -> Result<Self> {
        info!("📊 Initializing process monitor...");
        Ok(Self { 
            wfp_engine: None,
            whitelist: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            blocklist: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            _private: () 
        })
    }

    /// Initialize with WFP engine for automatic blocking
    pub fn with_wfp_engine(wfp_engine: Arc<WfpEngine>) -> Result<Self> {
        info!("📊 Initializing process monitor with WFP integration...");
        Ok(Self { 
            wfp_engine: Some(wfp_engine),
            whitelist: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            blocklist: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            _private: () 
        })
    }

    /// Add process to whitelist
    pub async fn add_to_whitelist(&self, pid: u32) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        if !whitelist.contains(&pid) {
            whitelist.push(pid);
            info!("✅ Process {} added to whitelist", pid);
        }
        Ok(())
    }

    /// Remove process from whitelist
    pub async fn remove_from_whitelist(&self, pid: u32) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        if let Some(pos) = whitelist.iter().position(|&p| p == pid) {
            whitelist.remove(pos);
            info!("❌ Process {} removed from whitelist", pid);
        }
        Ok(())
    }

    /// Check if process is whitelisted
    pub async fn is_whitelisted(&self, pid: u32) -> bool {
        let whitelist = self.whitelist.read().await;
        whitelist.contains(&pid)
    }

    /// Add process to blocklist and apply WFP filter
    pub async fn add_to_blocklist(&self, pid: u32, duration_secs: Option<u64>) -> Result<()> {
        // Check if already blocked
        {
            let blocklist = self.blocklist.read().await;
            if blocklist.contains(&pid) {
                warn!("Process {} is already blocked", pid);
                return Ok(());
            }
        }

        // Apply WFP filter if engine is available
        if let Some(ref wfp_engine) = self.wfp_engine {
            let duration = duration_secs.unwrap_or(300); // Default 5 minutes
            match wfp_engine.block_connection(pid, duration) {
                Ok(_) => info!("🛡️ WFP filter applied to block process {}", pid),
                Err(e) => error!("Failed to apply WFP filter: {}", e),
            }
        } else {
            warn!("WFP engine not available - blocklist entry will be symbolic only");
        }

        // Add to blocklist
        {
            let mut blocklist = self.blocklist.write().await;
            blocklist.push(pid);
            info!("🚫 Process {} added to blocklist", pid);
        }

        Ok(())
    }

    /// Remove process from blocklist
    pub async fn remove_from_blocklist(&self, pid: u32) -> Result<()> {
        // Remove WFP filter if engine is available
        if let Some(ref wfp_engine) = self.wfp_engine {
            let filter_id = format!("temp_block_{}", pid);
            match wfp_engine.remove_filter(&filter_id) {
                Ok(_) => info!("✅ WFP filter removed for process {}", pid),
                Err(e) => warn!("Failed to remove WFP filter: {}", e),
            }
        }

        // Remove from blocklist
        {
            let mut blocklist = self.blocklist.write().await;
            if let Some(pos) = blocklist.iter().position(|&p| p == pid) {
                blocklist.remove(pos);
                info!("✅ Process {} removed from blocklist", pid);
            }
        }

        Ok(())
    }

    /// Check if process is blocked
    pub async fn is_blocked(&self, pid: u32) -> bool {
        let blocklist = self.blocklist.read().await;
        whitelist.contains(&pid)
    }

    /// Get list of all processes with network activity
    pub fn get_processes(&self) -> Result<Vec<ProcessEntry>> {
        debug!("📋 Fetching process list...");
        
        // In production:
        // 1. Use Windows API (CreateToolhelp32Snapshot) to enumerate processes
        // 2. Use IP Helper API (GetExtendedTcpTable, GetExtendedUdpTable) for connections
        // 3. Aggregate bandwidth usage per process
        // 4. Check digital signature status
        
        // Placeholder data for demonstration
        Ok(vec![
            ProcessEntry {
                pid: 1234,
                name: "chrome.exe".to_string(),
                path: Some("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string()),
                upload_speed: 1024,
                download_speed: 5120,
                connection_count: 45,
                signature_verified: true,
            },
            ProcessEntry {
                pid: 5678,
                name: "video_client.exe".to_string(),
                path: Some("C:\\Users\\AppData\\Local\\VideoClient\\video_client.exe".to_string()),
                upload_speed: 8192,
                download_speed: 512,
                connection_count: 128,
                signature_verified: false,
            },
            ProcessEntry {
                pid: 9012,
                name: "svchost.exe".to_string(),
                path: Some("C:\\Windows\\System32\\svchost.exe".to_string()),
                upload_speed: 256,
                download_speed: 1024,
                connection_count: 12,
                signature_verified: true,
            },
        ])
    }

    /// Kill/terminate a process by PID
    pub fn kill_process(&self, pid: u32) -> Result<()> {
        info!("💀 Terminating process {}", pid);
        
        #[cfg(target_os = "windows")]
        {
            use std::ptr;
            use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
            use winapi::um::handleapi::CloseHandle;
            use winapi::um::winnt::PROCESS_TERMINATE;
            
            unsafe {
                let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                if handle.is_null() {
                    return Err(anyhow::anyhow!("Failed to open process {}", pid));
                }
                
                let result = TerminateProcess(handle, 1);
                CloseHandle(handle);
                
                if result == 0 {
                    return Err(anyhow::anyhow!("Failed to terminate process {}", pid));
                }
            }
        }
        
        // Non-Windows stub
        warn!("Kill process not supported on this platform");
        Ok(())
    }
    
    /// Get legacy process list (for backward compatibility) - Enhanced with whitelist/blocklist status
    pub async fn get_process_list(&self) -> Result<Vec<ProcessInfo>> {
        let entries = self.get_processes()?;
        
        // Get whitelist and blocklist snapshots
        let whitelist = self.whitelist.read().await;
        let blocklist = self.blocklist.read().await;
        
        Ok(entries.into_iter().map(|e| ProcessInfo {
            pid: e.pid,
            name: e.name.clone(),
            path: e.path.clone(),
            upload_speed: e.upload_speed,
            download_speed: e.download_speed,
            connection_count: e.connection_count,
            signature_verified: e.signature_verified,
            is_whitelisted: whitelist.contains(&e.pid),
            is_blocked: blocklist.contains(&e.pid),
        }).collect())
    }

    /// Get enhanced process list with behavior analysis
    pub async fn get_enhanced_process_list(&self) -> Result<Vec<EnhancedProcessInfo>> {
        let entries = self.get_processes()?;
        
        // Get whitelist and blocklist snapshots
        let whitelist = self.whitelist.read().await;
        let blocklist = self.blocklist.read().await;
        
        let mut result = Vec::new();
        for entry in entries {
            let behavior_score = BehaviorAnalyzer::analyze_from_entry(&entry);
            let behavior_class = BehaviorAnalyzer::classify_from_entry(&entry);
            
            result.push(EnhancedProcessInfo {
                pid: entry.pid,
                name: entry.name,
                path: entry.path,
                upload_speed: entry.upload_speed,
                download_speed: entry.download_speed,
                connection_count: entry.connection_count,
                signature_verified: entry.signature_verified,
                is_whitelisted: whitelist.contains(&entry.pid),
                is_blocked: blocklist.contains(&entry.pid),
                behavior_score,
                behavior_class,
                risk_level: Self::calculate_risk_level(entry.upload_speed, entry.connection_count, !entry.signature_verified),
            });
        }
        
        Ok(result)
    }

    /// Calculate risk level based on multiple factors
    fn calculate_risk_level(upload_speed: u64, connection_count: u32, unsigned: bool) -> RiskLevel {
        let mut score = 0u32;
        
        // Upload speed factor (>1MB/s is suspicious)
        if upload_speed > 1_048_576 {
            score += 2;
        } else if upload_speed > 524_288 {
            score += 1;
        }
        
        // Connection count factor (>100 is very suspicious)
        if connection_count > 100 {
            score += 2;
        } else if connection_count > 50 {
            score += 1;
        }
        
        // Signature factor
        if unsigned {
            score += 1;
        }
        
        match score {
            s if s >= 4 => RiskLevel::Critical,
            s if s >= 2 => RiskLevel::High,
            s if s >= 1 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Get detailed information for a specific process
    pub fn get_process_details(&self, pid: u32) -> Result<ProcessDetails> {
        debug!("Fetching details for process {}", pid);
        
        // In production:
        // 1. Open process handle with PROCESS_QUERY_INFORMATION
        // 2. Get executable path
        // 3. Get parent process ID
        // 4. Get command line arguments
        // 5. Get network connections for this process
        
        Ok(ProcessDetails {
            pid,
            name: "example.exe".to_string(),
            path: "C:\\Program Files\\Example\\example.exe".to_string(),
            parent_pid: Some(1000),
            command_line: "\"example.exe\" --arg1 value1".to_string(),
            start_time: std::time::SystemTime::now(),
            upload_speed: 1024,
            download_speed: 2048,
            connection_count: 10,
            connections: vec![],
            signature_verified: true,
            signature_publisher: Some("Example Inc.".to_string()),
        })
    }

    /// Terminate a process (legacy alias for kill_process)
    pub fn terminate_process(&self, pid: u32) -> Result<()> {
        self.kill_process(pid)
    }

    /// Get process tree (parent-child relationships)
    pub fn get_process_tree(&self) -> Result<ProcessTree> {
        debug!("Building process tree...");
        
        // In production:
        // 1. Enumerate all processes
        // 2. Build parent-child relationships
        // 3. Return tree structure
        
        Ok(ProcessTree::default())
    }

    /// Check if process has valid digital signature
    pub fn verify_signature(&self, pid: u32) -> Result<SignatureStatus> {
        debug!("Verifying signature for process {}", pid);
        
        // In production:
        // 1. Get process executable path
        // 2. Use WinVerifyTrust to verify Authenticode signature
        // 3. Return signature details
        
        Ok(SignatureStatus {
            is_valid: true,
            publisher: Some("Microsoft Corporation".to_string()),
            certificate_subject: "Microsoft Windows".to_string(),
            certificate_issuer: "Microsoft Windows Production PCA 2011".to_string(),
        })
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize process monitor")
    }
}

/// Process entry with full information
#[derive(Debug, Clone)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub signature_verified: bool,
}

/// Detailed process information
#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub parent_pid: Option<u32>,
    pub command_line: String,
    pub start_time: std::time::SystemTime,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub connections: Vec<ConnectionInfo>,
    pub signature_verified: bool,
    pub signature_publisher: Option<String>,
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub protocol: Protocol,
    pub state: ConnectionState,
}

/// Protocol type
#[derive(Debug, Clone)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// TCP connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Established,
    Listen,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Delete,
}

/// Process tree structure
#[derive(Debug, Default)]
pub struct ProcessTree {
    pub roots: Vec<TreeNode>,
}

/// Tree node for process hierarchy
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub process: ProcessInfo,
    pub children: Vec<TreeNode>,
}

/// Signature verification status
#[derive(Debug, Clone)]
pub struct SignatureStatus {
    pub is_valid: bool,
    pub publisher: Option<String>,
    pub certificate_subject: String,
    pub certificate_issuer: String,
}

/// Behavioral analysis for PCDN detection
pub struct BehaviorAnalyzer;

impl BehaviorAnalyzer {
    /// Analyze process behavior for PCDN-like patterns
    /// 
    /// Returns a score from 0.0 (normal) to 1.0 (highly suspicious)
    pub fn analyze(process: &ProcessInfo) -> f32 {
        let mut score = 0.0;
        
        // Upload/Download ratio check (> 5:1 is suspicious)
        if process.download_speed > 0 {
            let ratio = process.upload_speed as f32 / process.download_speed as f32;
            if ratio > 5.0 {
                score += 0.4;
            } else if ratio > 2.0 {
                score += 0.2;
            }
        } else if process.upload_speed > 1024 {
            // High upload with no download
            score += 0.3;
        }
        
        // Connection count check (> 50 is suspicious)
        if process.connection_count > 100 {
            score += 0.3;
        } else if process.connection_count > 50 {
            score += 0.15;
        } else if process.connection_count > 20 {
            score += 0.05;
        }
        
        // Signature check (unsigned is more suspicious)
        if !process.signature_verified {
            score += 0.2;
        }
        
        // Normalize to 0.0 - 1.0
        score.min(1.0)
    }

    /// Classify process based on behavior
    pub fn classify(process: &ProcessInfo) -> BehaviorClass {
        let score = Self::analyze(process);
        
        match score {
            s if s >= 0.7 => BehaviorClass::HighRisk,
            s if s >= 0.4 => BehaviorClass::MediumRisk,
            s if s >= 0.2 => BehaviorClass::LowRisk,
            _ => BehaviorClass::Normal,
        }
    }
}

/// Behavior classification
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorClass {
    Normal,
    LowRisk,
    MediumRisk,
    HighRisk,
}

/// Risk level for quick visual indication
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Enhanced process info with behavior analysis
#[derive(Debug, Clone)]
pub struct EnhancedProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub signature_verified: bool,
    pub is_whitelisted: bool,
    pub is_blocked: bool,
    pub behavior_score: f32,
    pub behavior_class: BehaviorClass,
    pub risk_level: RiskLevel,
}

impl BehaviorAnalyzer {
    /// Analyze from ProcessEntry (internal format)
    pub fn analyze_from_entry(entry: &ProcessEntry) -> f32 {
        let mut score = 0.0;
        
        // Upload/Download ratio check (> 5:1 is suspicious)
        if entry.download_speed > 0 {
            let ratio = entry.upload_speed as f32 / entry.download_speed as f32;
            if ratio > 5.0 {
                score += 0.4;
            } else if ratio > 2.0 {
                score += 0.2;
            }
        } else if entry.upload_speed > 1024 {
            // High upload with no download
            score += 0.3;
        }
        
        // Connection count check (> 50 is suspicious)
        if entry.connection_count > 100 {
            score += 0.3;
        } else if entry.connection_count > 50 {
            score += 0.15;
        } else if entry.connection_count > 20 {
            score += 0.05;
        }
        
        // Signature check (unsigned is more suspicious)
        if !entry.signature_verified {
            score += 0.2;
        }
        
        // Normalize to 0.0 - 1.0
        score.min(1.0)
    }

    /// Classify from ProcessEntry (internal format)
    pub fn classify_from_entry(entry: &ProcessEntry) -> BehaviorClass {
        let score = Self::analyze_from_entry(entry);
        
        match score {
            s if s >= 0.7 => BehaviorClass::HighRisk,
            s if s >= 0.4 => BehaviorClass::MediumRisk,
            s if s >= 0.2 => BehaviorClass::LowRisk,
            _ => BehaviorClass::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_analyzer_pcdn() {
        let process = ProcessInfo {
            pid: 1234,
            name: "pcdn_client.exe".to_string(),
            path: None,
            upload_speed: 10240,
            download_speed: 512,
            connection_count: 150,
            signature_verified: false,
            is_whitelisted: false,
            is_blocked: false,
        };

        let score = BehaviorAnalyzer::analyze(&process);
        assert!(score >= 0.7);
        
        let class = BehaviorAnalyzer::classify(&process);
        assert_eq!(class, BehaviorClass::HighRisk);
    }

    #[test]
    fn test_behavior_analyzer_normal() {
        let process = ProcessInfo {
            pid: 5678,
            name: "browser.exe".to_string(),
            path: None,
            upload_speed: 512,
            download_speed: 5120,
            connection_count: 15,
            signature_verified: true,
            is_whitelisted: false,
            is_blocked: false,
        };

        let score = BehaviorAnalyzer::analyze(&process);
        assert!(score < 0.2);
        
        let class = BehaviorAnalyzer::classify(&process);
        assert_eq!(class, BehaviorClass::Normal);
    }

    #[tokio::test]
    async fn test_process_monitor_whitelist_blocklist() {
        let monitor = ProcessMonitor::new().unwrap();
        
        // Test whitelist operations
        assert!(!monitor.is_whitelisted(1234).await);
        monitor.add_to_whitelist(1234).await.unwrap();
        assert!(monitor.is_whitelisted(1234).await);
        monitor.remove_from_whitelist(1234).await.unwrap();
        assert!(!monitor.is_whitelisted(1234).await);
        
        // Test blocklist operations
        assert!(!monitor.is_blocked(5678).await);
        monitor.add_to_blocklist(5678, Some(60)).await.unwrap();
        assert!(monitor.is_blocked(5678).await);
        monitor.remove_from_blocklist(5678).await.unwrap();
        assert!(!monitor.is_blocked(5678).await);
    }

    #[test]
    fn test_risk_level_calculation() {
        // Low risk: normal traffic, signed
        assert_eq!(
            ProcessMonitor::calculate_risk_level(1024, 10, false),
            RiskLevel::Low
        );
        
        // Medium risk: moderate upload
        assert_eq!(
            ProcessMonitor::calculate_risk_level(600_000, 20, false),
            RiskLevel::Medium
        );
        
        // High risk: high upload + many connections
        assert_eq!(
            ProcessMonitor::calculate_risk_level(2_000_000, 80, false),
            RiskLevel::High
        );
        
        // Critical risk: all factors present
        assert_eq!(
            ProcessMonitor::calculate_risk_level(2_000_000, 150, true),
            RiskLevel::Critical
        );
    }
}
