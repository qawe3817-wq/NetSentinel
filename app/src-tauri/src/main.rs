//! NetSentinel Tauri Core Bridge
//! 
//! This module provides IPC commands that the UI can call to interact with the core service.
//! It acts as a bridge between the React frontend and the Rust core service running as a Windows Service.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, warn, error};

type HmacSha256 = Hmac<Sha256>;

/// HMAC secret key for IPC security (in production, this should be dynamically generated)
const IPC_SECRET_KEY: &[u8] = b"netsentinel_ipc_secret_key_2024";

/// Process information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub is_signed: bool,
    pub risk_score: f32,
    pub parent_pid: Option<u32>,
}

/// Traffic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub total_upload: u64,
    pub total_download: u64,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub active_connections: u32,
    pub blocked_connections: u32,
}

/// Rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: Vec<RuleCondition>,
    pub action: RuleAction,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,
    pub operator: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Block { duration_secs: Option<u64> },
    Limit { upload_kbps: u32, download_kbps: u32 },
    Warn,
    Allow,
}

/// Threat event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEvent {
    pub timestamp: u64,
    pub process_name: String,
    pub pid: u32,
    pub target_ip: String,
    pub target_port: u16,
    pub reason: String,
    pub action_taken: String,
}

/// Shared state for the bridge
pub struct BridgeState {
    pub core_connected: bool,
    pub processes: Vec<ProcessInfo>,
    pub traffic_stats: TrafficStats,
    pub rules: Vec<Rule>,
    pub threats: Vec<ThreatEvent>,
    pub protection_mode: ProtectionMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProtectionMode {
    Silent,
    Blocking,
    Passthrough,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            core_connected: false,
            processes: Vec::new(),
            traffic_stats: TrafficStats {
                total_upload: 0,
                total_download: 0,
                upload_speed: 0,
                download_speed: 0,
                active_connections: 0,
                blocked_connections: 0,
            },
            rules: Vec::new(),
            threats: Vec::new(),
            protection_mode: ProtectionMode::Silent,
        }
    }
}

/// Compute HMAC-SHA256 signature for IPC message authentication
fn compute_hmac(data: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(IPC_SECRET_KEY).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Verify HMAC signature
fn verify_hmac(data: &str, signature: &str) -> bool {
    let expected = compute_hmac(data);
    expected == signature
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get all processes with network activity
#[tauri::command]
pub async fn get_processes(state: Arc<Mutex<BridgeState>>) -> Result<Vec<ProcessInfo>, String> {
    info!("get_processes called");
    let state_guard = state.lock().await;
    
    // In production, this would call the core service via NamedPipe/LocalSocket
    // For now, return mock data for development
    Ok(state_guard.processes.clone())
}

/// Get real-time traffic statistics
#[tauri::command]
pub async fn get_traffic_stats(state: Arc<Mutex<BridgeState>>) -> Result<TrafficStats, String> {
    info!("get_traffic_stats called");
    let state_guard = state.lock().await;
    Ok(state_guard.traffic_stats.clone())
}

/// Get all rules
#[tauri::command]
pub async fn get_rules(state: Arc<Mutex<BridgeState>>) -> Result<Vec<Rule>, String> {
    info!("get_rules called");
    let state_guard = state.lock().await;
    Ok(state_guard.rules.clone())
}

/// Create or update a rule
#[tauri::command]
pub async fn apply_rule(
    rule: Rule,
    state: Arc<Mutex<BridgeState>>,
) -> Result<bool, String> {
    info!("apply_rule called for rule: {}", rule.name);
    
    let mut state_guard = state.lock().await;
    
    // Check if rule exists
    if let Some(existing) = state_guard.rules.iter_mut().find(|r| r.id == rule.id) {
        *existing = rule;
        info!("Rule updated");
    } else {
        state_guard.rules.push(rule);
        info!("Rule created");
    }
    
    // In production, send rule to core service via IPC
    // core_client.apply_rule(&rule).await?;
    
    Ok(true)
}

/// Delete a rule
#[tauri::command]
pub async fn delete_rule(
    rule_id: String,
    state: Arc<Mutex<BridgeState>>,
) -> Result<bool, String> {
    info!("delete_rule called for id: {}", rule_id);
    
    let mut state_guard = state.lock().await;
    let initial_len = state_guard.rules.len();
    state_guard.rules.retain(|r| r.id != rule_id);
    
    Ok(state_guard.rules.len() < initial_len)
}

/// Get recent threat events
#[tauri::command]
pub async fn get_threats(state: Arc<Mutex<BridgeState>>) -> Result<Vec<ThreatEvent>, String> {
    info!("get_threats called");
    let state_guard = state.lock().await;
    Ok(state_guard.threats.clone())
}

/// Change protection mode
#[tauri::command]
pub async fn set_protection_mode(
    mode: ProtectionMode,
    state: Arc<Mutex<BridgeState>>,
) -> Result<(), String> {
    info!("set_protection_mode called: {:?}", mode);
    
    let mut state_guard = state.lock().await;
    state_guard.protection_mode = mode;
    
    // In production, notify core service
    // core_client.set_protection_mode(mode).await?;
    
    Ok(())
}

/// Get current protection mode
#[tauri::command]
pub async fn get_protection_mode(state: Arc<Mutex<BridgeState>>) -> Result<ProtectionMode, String> {
    let state_guard = state.lock().await;
    Ok(state_guard.protection_mode)
}

/// Terminate a process
#[tauri::command]
pub async fn terminate_process(pid: u32) -> Result<bool, String> {
    info!("terminate_process called for pid: {}", pid);
    
    // In production, call core service to terminate process
    // core_client.terminate_process(pid).await
    
    // Mock implementation
    Ok(true)
}

/// Add process to whitelist
#[tauri::command]
pub async fn add_to_whitelist(process_path: String) -> Result<bool, String> {
    info!("add_to_whitelist called for: {}", process_path);
    
    // In production, update whitelist in core service
    // core_client.add_to_whitelist(&process_path).await
    
    Ok(true)
}

/// Block a process temporarily
#[tauri::command]
pub async fn block_process(
    pid: u32,
    duration_secs: u64,
) -> Result<bool, String> {
    info!("block_process called for pid: {} duration: {}s", pid, duration_secs);
    
    // In production, call core service
    // core_client.block_process(pid, duration_secs).await
    
    Ok(true)
}

/// Check core service connection status
#[tauri::command]
pub async fn check_core_connection(state: Arc<Mutex<BridgeState>>) -> Result<bool, String> {
    let state_guard = state.lock().await;
    Ok(state_guard.core_connected)
}

/// Initialize bridge and attempt to connect to core service
#[tauri::command]
pub async fn initialize_bridge(state: Arc<Mutex<BridgeState>>) -> Result<(), String> {
    info!("initialize_bridge called");
    
    let mut state_guard = state.lock().await;
    
    // In production, attempt to connect to core service via NamedPipe
    // match connect_to_core_service().await {
    //     Ok(client) => {
    //         state_guard.core_connected = true;
    //         info!("Connected to core service");
    //     }
    //     Err(e) => {
    //         warn!("Failed to connect to core service: {}", e);
    //         state_guard.core_connected = false;
    //     }
    // }
    
    // Mock: simulate connection after delay
    state_guard.core_connected = true;
    info!("Bridge initialized (mock mode)");
    
    Ok(())
}

/// Simulate mock data for development (will be removed in production)
#[tauri::command]
pub async fn refresh_mock_data(state: Arc<Mutex<BridgeState>>) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let mut state_guard = state.lock().await;
    
    // Generate mock processes
    state_guard.processes = vec![
        ProcessInfo {
            pid: 1234,
            name: "chrome.exe".to_string(),
            path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(),
            upload_speed: 125000,
            download_speed: 2500000,
            connection_count: 45,
            is_signed: true,
            risk_score: 0.1,
            parent_pid: Some(5678),
        },
        ProcessInfo {
            pid: 5678,
            name: "svchost.exe".to_string(),
            path: "C:\\Windows\\System32\\svchost.exe".to_string(),
            upload_speed: 50000,
            download_speed: 100000,
            connection_count: 120,
            is_signed: true,
            risk_score: 0.2,
            parent_pid: None,
        },
        ProcessInfo {
            pid: 9012,
            name: "unknown_miner.exe".to_string(),
            path: "C:\\Users\\AppData\\Local\\Temp\\unknown_miner.exe".to_string(),
            upload_speed: 850000,
            download_speed: 50000,
            connection_count: 250,
            is_signed: false,
            risk_score: 0.95,
            parent_pid: None,
        },
    ];
    
    // Generate mock traffic stats
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    state_guard.traffic_stats = TrafficStats {
        total_upload: timestamp * 1000,
        total_download: timestamp * 5000,
        upload_speed: 1025000,
        download_speed: 2650000,
        active_connections: 415,
        blocked_connections: 23,
    };
    
    // Generate mock threats
    state_guard.threats = vec![
        ThreatEvent {
            timestamp: timestamp - 60,
            process_name: "unknown_miner.exe".to_string(),
            pid: 9012,
            target_ip: "185.234.72.15".to_string(),
            target_port: 8333,
            reason: "High upload ratio detected (PCDN behavior)".to_string(),
            action_taken: "Blocked".to_string(),
        },
        ThreatEvent {
            timestamp: timestamp - 300,
            process_name: "suspicious_p2p.exe".to_string(),
            pid: 3456,
            target_ip: "91.234.56.78".to_string(),
            target_port: 6881,
            reason: "BitTorrent protocol detected".to_string(),
            action_taken: "Blocked".to_string(),
        },
    ];
    
    info!("Mock data refreshed");
    Ok(())
}

// ============================================================================
// Tauri Application Setup
// ============================================================================

pub fn run() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let bridge_state = Arc::new(Mutex::new(BridgeState::default()));
    
    tauri::Builder::default()
        .manage(bridge_state)
        .invoke_handler(tauri::generate_handler![
            get_processes,
            get_traffic_stats,
            get_rules,
            apply_rule,
            delete_rule,
            get_threats,
            set_protection_mode,
            get_protection_mode,
            terminate_process,
            add_to_whitelist,
            block_process,
            check_core_connection,
            initialize_bridge,
            refresh_mock_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
