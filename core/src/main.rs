//! NetSentinel Core Service
//! 
//! Kernel-level network privacy defense using Windows Filtering Platform (WFP)
//! 
//! # Architecture
//! 
//! Separated Micro-Kernel design:
//! - UI Layer (React/TypeScript/Tauri) <--IPC--> Core Service (Rust)
//! 
//! # Modules
//! 
//! - `config`: Configuration management
//! - `ipc`: Inter-process communication via NamedPipe
//! - `process`: Process monitoring and management
//! - `rules`: Rule engine with condition-based filtering
//! - `wfp`: Windows Filtering Platform engine abstraction
//! - `wfp_native`: Native WFP API bindings

mod config;
mod ipc;
mod process;
mod rules;
mod wfp;
mod wfp_native;

use anyhow::{Context, Result};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

/// Application version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core service entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!("🛡️  NetSentinel Core v{} starting...", VERSION);

    // Load configuration with error context
    let cfg = config::Config::load()
        .context("Failed to load configuration")?;
    info!("Configuration loaded from: {}", cfg.config_path.display());

    // Initialize IPC server (NamedPipe for Windows)
    let ipc_server = ipc::IpcServer::new(&cfg)
        .context("Failed to initialize IPC server")?;
    info!("IPC server initialized on pipe: {}", cfg.pipe_name);

    // Initialize WFP engine
    let wfp_engine = wfp::WfpEngine::new()
        .context("Failed to initialize WFP engine")?;
    info!("WFP engine initialized successfully");

    // Start watchdog for self-healing
    let watchdog = wfp::Watchdog::spawn()
        .context("Failed to spawn watchdog")?;
    info!("Watchdog started - will restart service within 2s on crash");

    // Main event loop with graceful shutdown
    tokio::select! {
        result = ipc_server.run() => {
            if let Err(e) = result {
                error!("IPC server error: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    // Cleanup in reverse order of initialization
    drop(watchdog);
    drop(wfp_engine);
    drop(ipc_server);

    info!("NetSentinel Core shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_format() {
        assert!(!VERSION.is_empty(), "Version should not be empty");
        assert!(VERSION.chars().all(|c| c.is_ascii_digit() || c == '.'), 
                "Version should contain only digits and dots");
    }
}
