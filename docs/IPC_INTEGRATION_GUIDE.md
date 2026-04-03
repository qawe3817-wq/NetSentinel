# UI 与 Core IPC 联调指南

## 概述

本文档描述如何将 React 前端 (UI 层) 与 Rust 核心服务 (Core 层) 通过 Tauri IPC 进行真实联调。

## 架构概览

```
┌─────────────────────┐    Tauri IPC (invoke/listen)    ┌──────────────────────┐
│   UI Layer (React)  │◄──────────────────────────────►│   Core Layer (Rust)  │
│                     │                                 │                      │
│  - useProcessMonitor│                                 │  - main.rs (cmds)    │
│  - useTrafficWave   │                                 │  - process.rs        │
│  - useRuleEngine    │                                 │  - wfp.rs            │
│  - Dashboard        │                                 │  - rules.rs          │
│  - ProcessMonitor   │                                 │  - ipc.rs            │
└─────────────────────┘                                 └──────────────────────┘
```

## 已完成的 Hook 更新

### 1. useProcessMonitor.ts

**新增 IPC 调用:**
- `get_processes()` - 获取进程列表
- `block_process(pid, durationSecs)` - 阻断进程
- `kill_process(pid)` - 结束进程

**状态管理:**
- `loading` - 加载状态
- `error` - 错误信息

### 2. useTrafficWave.ts

**新增 IPC 调用:**
- `get_traffic_stats()` - 获取实时流量统计

**功能:**
- Canvas 60FPS 波形渲染
- 平滑插值动画
- 霓虹光效

## Rust Core 命令实现

### 在 core/src/main.rs 中添加 Tauri 命令

```rust
use tauri::command;
use crate::{process::ProcessMonitor, wfp::WfpEngine, rules::Rule};

// 全局状态 (实际应使用 tauri::State)
static mut PROCESS_MONITOR: Option<ProcessMonitor> = None;
static mut WFP_ENGINE: Option<WfpEngine> = None;

/// 获取进程列表
#[command]
pub async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    let monitor = unsafe { PROCESS_MONITOR.as_ref() }
        .ok_or("Process monitor not initialized")?;
    
    monitor.get_all_processes()
        .map_err(|e| e.to_string())
}

/// 阻断指定进程的网络连接
#[command]
pub async fn block_process(pid: u32, duration_secs: u64) -> Result<(), String> {
    let engine = unsafe { WFP_ENGINE.as_ref() }
        .ok_or("WFP engine not initialized")?;
    
    engine.block_connection(pid, duration_secs)
        .map_err(|e| e.to_string())
}

/// 结束进程
#[command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    let monitor = unsafe { PROCESS_MONITOR.as_ref() }
        .ok_or("Process monitor not initialized")?;
    
    monitor.terminate_process(pid)
        .map_err(|e| e.to_string())
}

/// 获取流量统计
#[command]
pub async fn get_traffic_stats() -> Result<TrafficStats, String> {
    let engine = unsafe { WFP_ENGINE.as_ref() }
        .ok_or("WFP engine not initialized")?;
    
    engine.get_stats()
        .map(|s| TrafficStats {
            total_upload: s.upload_bytes,
            total_download: s.download_bytes,
            blocked_count: s.blocked_connections,
            current_upload_speed: s.upload_speed_bps(),
            current_download_speed: s.download_speed_bps(),
        })
        .map_err(|e| e.to_string())
}

/// 添加规则
#[command]
pub async fn add_rule(rule: Rule) -> Result<(), String> {
    // TODO: 实现规则添加
    Ok(())
}

/// 获取所有规则
#[command]
pub async fn get_rules() -> Result<Vec<Rule>, String> {
    // TODO: 实现规则获取
    Ok(vec![])
}
```

### 在 tauri.conf.json 中注册命令

```json
{
  "tauri": {
    "systemTray": { ... },
    "allowlist": {
      "all": true
    }
  },
  "build": {
    "beforeBuildCommand": "...",
    "beforeDevCommand": "...",
    "devPath": "http://localhost:5173",
    "distDir": "../app/dist"
  }
}
```

### 在 src-tauri/src/lib.rs 中初始化

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_processes,
            block_process,
            kill_process,
            get_traffic_stats,
            add_rule,
            get_rules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 数据类型映射

### TypeScript → Rust

| TypeScript | Rust | 说明 |
|-----------|------|------|
| `number` | `u32`/`u64` | 根据数值范围选择 |
| `string` | `String` | UTF-8 字符串 |
| `boolean` | `bool` | 布尔值 |
| `null` | `Option<T>` | 可选类型 |
| `Array<T>` | `Vec<T>` | 数组 |
| `object` | `struct` | 对象结构体 |

### ProcessInfo 映射示例

```typescript
// TypeScript (app/src/hooks/useProcessMonitor.ts)
export interface ProcessInfo {
  pid: number
  name: string
  path: string
  uploadSpeed: number
  downloadSpeed: number
  connections: number
  status: 'normal' | 'suspicious' | 'blocked' | 'whitelisted'
  signature?: {
    verified: boolean
    publisher?: string
  }
}
```

```rust
// Rust (core/src/process.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: u32,
    pub status: ProcessStatus,
    pub signature: Option<SignatureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    Normal,
    Suspicious,
    Blocked,
    Whitelisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub verified: bool,
    pub publisher: Option<String>,
}
```

## 错误处理

### Rust 端错误转换

```rust
#[command]
pub async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    match monitor.get_all_processes() {
        Ok(processes) => Ok(processes),
        Err(e) => Err(format!("Failed to get processes: {}", e)),
    }
}
```

### TypeScript 端错误捕获

```typescript
try {
  const result = await invoke<ProcessInfo[]>('get_processes')
  setProcesses(result)
  setError(null)
} catch (err) {
  console.error('Failed to fetch processes:', err)
  setError(err instanceof Error ? err.message : 'Unknown error')
}
```

## 测试步骤

### 1. 开发模式测试

```bash
# 终端 1: 启动 Rust 后端
cd /workspace/core
cargo build

# 终端 2: 启动 Tauri 应用
cd /workspace
npm install
npm run tauri dev
```

### 2. 验证 IPC 通信

打开浏览器开发者工具，查看 Console:

```javascript
// 应该看到类似输出
Blocking processes: [1234, 5678]
Killing process: 9012
```

### 3. 性能测试

- [ ] 进程列表刷新 < 100ms
- [ ] 流量波形 60FPS 稳定
- [ ] 内存占用 < 50MB (空闲)

## 下一步

1. **实现真实的 WFP 拦截** - 完成 `wfp.rs` 中的 Windows API 调用
2. **添加事件订阅** - 使用 Tauri Event System 实现实时推送
3. **优化数据序列化** - 使用 MessagePack 替代 JSON 提升性能
4. **安全加固** - 实现 HMAC 消息认证

## 常见问题

### Q: invoke 返回 Promise 不解析？
A: 确保 Rust 命令函数标记为 `async` 并返回 `Result<T, String>`

### Q: 类型不匹配错误？
A: 检查 TypeScript 接口与 Rust struct 的字段名和类型是否一致

### Q: Windows 特定 API 如何在 Linux/macOS 开发？
A: 使用 `#[cfg(target_os = "windows")]` 条件编译，开发时使用 Mock 实现
