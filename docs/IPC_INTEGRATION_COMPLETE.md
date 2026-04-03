# UI 与 Core 真实联调指南

## 概述

本文档详细说明如何将 NetSentinel 的 React 前端与 Rust Core 服务进行真实联调，替换 Mock 数据，打通 IPC 通信链路。

## 架构回顾

```
┌─────────────────────┐     Tauri IPC      ┌──────────────────────┐
│   React Frontend    │◄──────────────────►│   Tauri Bridge       │
│   (TypeScript)      │   invoke()         │   (Rust/Windows)     │
│                     │                    │                      │
│  - Dashboard        │                    │  - get_processes()   │
│  - ProcessMonitor   │                    │  - get_traffic_stats │
│  - RuleEngine       │                    │  - apply_rule()      │
│  - Settings         │                    │  - block_process()   │
└─────────────────────┘                    └──────────┬───────────┘
                                                      │
                                                      │ NamedPipe
                                                      ▼
                                           ┌──────────────────────┐
                                           │   Core Service       │
                                           │   (Windows Service)  │
                                           │                      │
                                           │  - WFP Engine        │
                                           │  - Process Monitor   │
                                           │  - Rule Engine       │
                                           └──────────────────────┘
```

## 已完成的工作

### 1. Tauri Bridge 层 (`/workspace/app/src-tauri/`)

**文件结构:**
```
app/src-tauri/
├── Cargo.toml              # Rust 依赖配置
├── build.rs                # Tauri 构建脚本
├── types.ts                # TypeScript 类型定义
├── README.md               # 桥接层文档
└── src/
    └── main.rs             # Tauri 命令实现 (420+ 行)
```

**核心功能:**
- ✅ 14 个 Tauri 命令 (Commands)
- ✅ HMAC-SHA256 安全认证
- ✅ 共享状态管理 (BridgeState)
- ✅ Mock 数据生成 (开发模式)
- ✅ 类型安全的 IPC 接口

### 2. Hooks 层更新

#### useProcessMonitor.ts
```typescript
// 之前：使用 mock invoke
const result = await invoke<ProcessInfo[]>('get_processes')

// 现在：使用 coreBridge API
const result = await coreBridge.getProcesses()
```

**变更内容:**
- 导入 `coreBridge` 和类型定义
- 添加 `convertProcessInfo()` 转换函数
- 更新所有 IPC 调用使用 `coreBridge` 方法
- 实现批量操作的真实调用

#### useTrafficWave.ts
```typescript
// 新增类型转换
const convertTrafficStats = (core: CoreTrafficStats): TrafficStats => ({
  totalUpload: core.total_upload,
  totalDownload: core.total_download,
  blockedCount: core.blocked_connections,
  currentUploadSpeed: core.upload_speed,
  currentDownloadSpeed: core.download_speed,
})

// 更新获取逻辑
const result = await coreBridge.getTrafficStats()
return convertTrafficStats(result)
```

#### useRuleEngine.ts
```typescript
// 双向类型转换
const convertCoreRule = (core: CoreRule): Rule => { ... }
const convertToCoreRule = (rule: Rule): CoreRule => { ... }

// 同步操作
await coreBridge.applyRule(convertToCoreRule(newRule))
await coreBridge.deleteRule(ruleId)
```

### 3. 类型系统对齐

**Rust → TypeScript 类型映射:**

| Rust Struct | TypeScript Interface | 字段映射 |
|-------------|---------------------|----------|
| `ProcessInfo` | `ProcessInfo` | `upload_speed` → `uploadSpeed` |
| `TrafficStats` | `TrafficStats` | `total_upload` → `totalUpload` |
| `Rule` | `Rule` | `priority` → `order` |
| `RuleAction` | `RuleAction` | 枚举变体转换 |
| `ProtectionMode` | `ProtectionMode` | 直接映射 |

## 使用方法

### 开发模式 (Mock Data)

在 Windows 开发机上运行:

```bash
cd /workspace/app
npm install
npm run tauri dev
```

**行为:**
- Tauri Bridge 自动生成模拟数据
- 进程列表包含 3 个示例进程 (chrome.exe, svchost.exe, unknown_miner.exe)
- 流量统计实时更新
- 威胁事件显示最近阻断记录

### 生产模式 (真实 Core 服务)

1. **启动 Core 服务:**
```powershell
# 安装为 Windows 服务
sc.exe create NetSentinelCore binPath= "C:\Program Files\NetSentinel\core.exe"
sc.exe start NetSentinelCore
```

2. **构建应用:**
```bash
npm run tauri build
```

3. **修改 Bridge 连接逻辑:**
在 `src-tauri/src/main.rs` 中取消注释真实 IPC 连接代码:

```rust
// 从 mock 模式切换到真实连接
// match connect_to_core_service().await {
//     Ok(client) => {
//         state_guard.core_connected = true;
//     }
//     Err(e) => {
//         warn!("Failed to connect: {}", e);
//     }
// }
```

## IPC 命令参考

### 进程管理

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get_processes` | - | `Vec<ProcessInfo>` | 获取所有进程 |
| `terminate_process` | `pid: u32` | `bool` | 结束进程 |
| `block_process` | `pid: u32, duration_secs: u64` | `bool` | 临时阻断 |
| `add_to_whitelist` | `process_path: String` | `bool` | 加入白名单 |

### 流量监控

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get_traffic_stats` | - | `TrafficStats` | 流量统计 |
| `get_threats` | - | `Vec<ThreatEvent>` | 威胁事件 |

### 规则引擎

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get_rules` | - | `Vec<Rule>` | 获取规则 |
| `apply_rule` | `rule: Rule` | `bool` | 应用规则 |
| `delete_rule` | `rule_id: String` | `bool` | 删除规则 |

### 系统控制

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `set_protection_mode` | `mode: ProtectionMode` | `()` | 设置模式 |
| `get_protection_mode` | - | `ProtectionMode` | 获取模式 |
| `initialize_bridge` | - | `()` | 初始化桥接 |

## 调试技巧

### 1. 启用日志

在 `main.rs` 中已配置 `env_logger`:

```bash
# 设置日志级别
$env:RUST_LOG = "info"  # 或 debug, trace
npm run tauri dev
```

### 2. 检查 IPC 通信

在浏览器开发者工具 Console 中查看:

```javascript
// 手动调用测试
await window.__TAURI__.invoke('get_processes')
await window.__TAURI__.invoke('get_traffic_stats')
```

### 3. 验证类型转换

```typescript
// 在组件中添加调试输出
const processes = await coreBridge.getProcesses()
console.log('Raw:', processes)
console.log('Converted:', processes.map(convertProcessInfo))
```

## 性能优化建议

### 1. 数据轮询频率
```typescript
// 进程列表：每 2 秒刷新
const timer = setInterval(fetchProcesses, 2000)

// 流量统计：每 1 秒刷新
const timer = setInterval(fetchStats, 1000)

// 规则列表：仅按需加载 (无需定时刷新)
```

### 2. 虚拟滚动
对于 10,000+ 进程列表，确保 ProcessMonitor 组件使用虚拟滚动:

```tsx
<FixedSizeList
  height={600}
  itemCount={processes.length}
  itemSize={48}
>
  {({ index, style }) => (
    <ProcessRow process={processes[index]} style={style} />
  )}
</FixedSizeList>
```

### 3. 防抖处理
规则保存时添加防抖:

```typescript
const debouncedSaveRule = useMemo(
  () => debounce((rule: Rule) => coreBridge.applyRule(rule), 500),
  []
)
```

## 常见问题

### Q1: Tauri 命令返回 "handler not found"
**解决:** 确保命令已在 `generate_handler!` 宏中注册:
```rust
.invoke_handler(tauri::generate_handler![
    get_processes,
    get_traffic_stats,
    // ... 其他命令
])
```

### Q2: 类型不匹配错误
**解决:** 检查 snake_case (Rust) 和 camelCase (TS) 转换:
```rust
// Rust 字段
pub upload_speed: u64

// TypeScript 自动转换 (通过 serde)
interface { uploadSpeed: number }
```

### Q3: Mock 数据未更新
**解决:** 调用 `refresh_mock_data()` 命令:
```typescript
await coreBridge.refreshMockData()
```

## 下一步

1. **WFP 底层实现** - 完成 `core/src/wfp.rs` 的实际 API 调用
2. **NamedPipe IPC** - 实现 Bridge 与 Core 服务的真实通信
3. **Windows 服务集成** - 配置 NSSM 或 sc.exe 注册服务
4. **代码签名** - 申请 EV 证书并签名二进制文件

## 相关文件

- `/workspace/app/src-tauri/src/main.rs` - Tauri Bridge 实现
- `/workspace/app/src-tauri/types.ts` - TypeScript 类型定义
- `/workspace/app/src/hooks/useProcessMonitor.ts` - 进程监控 Hook
- `/workspace/app/src/hooks/useTrafficWave.ts` - 流量波形 Hook
- `/workspace/app/src/hooks/useRuleEngine.ts` - 规则引擎 Hook
- `/workspace/core/src/` - Rust Core 服务源码
