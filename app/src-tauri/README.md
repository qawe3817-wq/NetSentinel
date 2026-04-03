# Tauri Core Bridge for NetSentinel

This directory contains the Tauri bridge layer that connects the React frontend with the Rust core service.

## Architecture

```
┌─────────────────┐     IPC Commands      ┌──────────────────────┐
│  React Frontend │◄─────────────────────►│  Tauri Bridge (Rust) │
│   (TypeScript)  │                       │   (src-tauri/main.rs)│
└─────────────────┘                       └──────────┬───────────┘
                                                     │
                                                     │ NamedPipe/LocalSocket
                                                     ▼
                                          ┌──────────────────────┐
                                          │   Core Service       │
                                          │   (/workspace/core)  │
                                          └──────────────────────┘
```

## Available IPC Commands

### Process Management
- `get_processes()` - Get all processes with network activity
- `terminate_process(pid)` - Terminate a specific process
- `block_process(pid, duration_secs)` - Temporarily block a process
- `add_to_whitelist(process_path)` - Add process to whitelist

### Traffic Monitoring
- `get_traffic_stats()` - Get real-time traffic statistics
- `get_threats()` - Get recent threat events

### Rule Engine
- `get_rules()` - Get all rules
- `apply_rule(rule)` - Create or update a rule
- `delete_rule(rule_id)` - Delete a rule

### Protection Control
- `set_protection_mode(mode)` - Change protection mode (Silent/Blocking/Passthrough)
- `get_protection_mode()` - Get current protection mode
- `check_core_connection()` - Check if core service is connected

### System
- `initialize_bridge()` - Initialize bridge and connect to core service
- `refresh_mock_data()` - Refresh mock data (development only)

## Development

### Prerequisites
- Node.js 18+
- Rust 1.70+
- Tauri CLI: `npm install -g @tauri-apps/cli`

### Running in Development Mode

```bash
# From the /workspace/app directory
npm install
npm run tauri dev
```

### Building for Production

```bash
npm run tauri build
```

## Data Types

All TypeScript interfaces in `types.ts` mirror the Rust structs in `main.rs` to ensure type safety across the IPC boundary.

### Security

- All IPC messages are HMAC-SHA256 signed (production mode)
- Communication uses NamedPipe with access control (Windows)
- Core service runs as Windows Service with elevated privileges

## Production Notes

In production, the mock data functions will be replaced with actual calls to the core service via NamedPipe/LocalSocket IPC. The bridge handles:
- Connection management
- Message serialization/deserialization
- Error handling and retry logic
- HMAC signature verification

## File Structure

```
src-tauri/
├── Cargo.toml          # Rust dependencies
├── build.rs            # Tauri build script
├── types.ts            # TypeScript type definitions
└── src/
    └── main.rs         # Tauri commands and bridge logic
```
