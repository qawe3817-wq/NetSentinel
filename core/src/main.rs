//! NetSentinel Core Service
//! 
//! Kernel-level network privacy defense using Windows Filtering Platform (WFP)
//! 
//! Architecture: Separated Micro-Kernel
//! - UI Layer (React/TypeScript) <--IPC--> Core Service (Rust)

mod wfp;
mod wfp_native;
mod process;
mod rules;
mod ipc;
mod config;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{fmt, EnvFilter};

/// Core service entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("🛡️  NetSentinel Core v2.0 starting...");

    // Load configuration
    let cfg = config::Config::load()?;
    info!("Configuration loaded");

    // Initialize IPC server (NamedPipe for Windows)
    let ipc_server = ipc::IpcServer::new(&cfg)?;
    info!("IPC server initialized");

    // Initialize WFP engine
    let wfp_engine = wfp::WfpEngine::new()?;
    info!("WFP engine initialized");

    // Start watchdog for self-healing
    let watchdog = wfp::Watchdog::spawn()?;
    info!("Watchdog started");

    // Main event loop
    tokio::select! {
        result = ipc_server.run() => {
            if let Err(e) = result {
                error!("IPC server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    // Cleanup
    drop(wfp_engine);
    drop(watchdog);

    info!("NetSentinel Core shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "2.0.0");
    }
}
