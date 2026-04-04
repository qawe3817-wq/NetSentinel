# WFP Native API 实现指南

## 📋 概述

本文档详细说明 NetSentinel V2.0 的 Windows Filtering Platform (WFP) 原生 API 绑定实现。

---

## 🏗️ 架构设计

### 模块结构

```
core/src/
├── wfp.rs          # 高层 WFP 引擎（跨平台抽象）
├── wfp_native.rs   # 原生 Windows API 绑定（Windows only）
└── ...
```

### 平台适配策略

| 平台 | 实现方式 | 功能状态 |
| :--- | :--- | :--- |
| **Windows 10 20H2+** | 原生 WFP API | ✅ 完整功能 |
| **Windows 11** | 原生 WFP API | ✅ 完整功能 |
| **Linux / macOS** | Stub 模拟 | ⚠️ 仅开发测试 |

---

## 🔧 核心组件

### 1. NativeWfpEngine

**位置**: `core/src/wfp_native.rs`

**功能**:
- WFP 会话管理（打开/关闭）
- 子层（Sublayer）创建
- 过滤器（Filter）添加/删除
- 临时阻断（带过期时间）

**关键 API**:

```rust
// 初始化 WFP 引擎
let engine = NativeWfpEngine::new()?;

// 添加进程过滤器
let filter_id = engine.add_process_filter(
    process_id,      // u32: 目标进程 ID
    action,          // FilterAction::Block/Permit
    layer,           // WfpLayer::AleAuthConnectV4
    priority,        // u16: 过滤器权重
)?;

// 临时阻断连接
let temp_filter_id = engine.block_connection_with_timeout(
    process_id,      // u32: 目标进程 ID
    duration_secs,   // u64: 阻断时长（秒）
)?;

// 删除过滤器
engine.remove_filter(&filter_id)?;
```

---

## 🪟 Windows API 调用流程

### 步骤 1: 打开 WFP 会话

```rust
let mut session = FWPM_SESSION0 {
    displayData: FWPM_DISPLAY_DATA0 {
        name: to_wide_string("NetSentinel WFP Session"),
        description: to_wide_string("Network filtering session"),
    },
    flags: FWPM_SESSION_FLAG_DYNAMIC, // 会话结束时自动清理
    ..Default::default()
};

let mut engine_handle: HANDLE = HANDLE::default();
let status = FwpmEngineOpen0(
    None,               // 本地机器
    RPC_C_AUTHN_DEFAULT,
    None,               // 默认认证
    &mut session,
    &mut engine_handle,
);

if status != ERROR_SUCCESS.0 {
    return Err(anyhow!("WFP 引擎打开失败：HRESULT 0x{:X}", status));
}
```

### 步骤 2: 创建子层

```rust
let sublayer_key = generate_guid();
let mut sublayer = FWPM_SUBLAYER0 {
    subLayerKey: sublayer_key,
    displayData: FWPM_DISPLAY_DATA0 {
        name: to_wide_string("NetSentinel Sublayer"),
        description: to_wide_string("Main filtering sublayer"),
    },
    weight: 0x0000FFFF, // 高优先级
    ..Default::default()
};

FwpmSubLayerAdd0(engine_handle, &sublayer, None);
```

### 步骤 3: 添加过滤器

```rust
let filter_id = generate_guid();
let mut filter = FWPM_FILTER0 {
    displayData: FWPM_DISPLAY_DATA0 {
        name: to_wide_string(&format!("NetSentinel Filter PID {}", process_id)),
        description: to_wide_string("Process-based filter"),
    },
    layer_key: FWPM_LAYER_ALE_AUTH_CONNECT_V4,
    sub_layer_key: sublayer_key,
    weight: FWP_VALUE0 { type_: FWP_UINT16_TYPE, uint16: priority },
    num_filter_conditions: 1,
    filter_condition: &mut condition,
    action: FWPM_ACTION0 { type_: FWP_ACTION_BLOCK, ..Default::default() },
    filter_id,
    ..Default::default()
};

FwpmFilterAdd0(engine_handle, &filter, None, None);
```

### 步骤 4: 临时阻断（带过期）

```rust
let expire_time = unix_time_to_filetime(now + duration_secs);
let mut filter = FWPM_FILTER0 {
    // ... 其他字段
    expiration_time: expire_time, // Windows FILETIME 格式
    flags: FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHTS,
    ..Default::default()
};

FwpmFilterAdd0(engine_handle, &filter, None, None);
// WFP 会自动在过期时间删除此过滤器
```

---

## 🛡️ 安全与权限

### 必需权限

1. **管理员权限**: WFP API 需要 `SeLoadDriverPrivilege`
2. **服务账户**: 建议以 `LocalSystem` 账户运行 Windows Service

### 代码签名

**必须使用 EV Code Signing Certificate** 签名以防止误报：

```bash
# 使用 signtool 签名
signtool sign /v /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 \
    /f "EV_Certificate.pfx" /p "password" \
    target/release/netsentinel-core.exe
```

### 杀软白名单

提交以下文件到主流杀软白名单：
- `netsentinel-core.exe`
- `netsentinel-ui.exe`
- WFP 驱动相关文件

---

## 📦 依赖配置

### Cargo.toml

```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_NetworkManagement_WindowsFilteringPlatform",
    "Win32_NetworkManagement_IpHelper",
    "Win32_System_Threading",
    "Win32_Security",
    "Win32_System_Com",  # CoCreateGuid
]}

uuid = { version = "1.6", features = ["v4"] }
rand = "0.8"
```

---

## 🔍 错误处理

### 常见 HRESULT 错误码

| HRESULT | 含义 | 解决方案 |
| :--- | :--- | :--- |
| `0x80320009` | FWP_E_ALREADY_EXISTS | 子层已存在，可忽略 |
| `0x80070005` | E_ACCESSDENIED | 缺少管理员权限 |
| `0x8007042C` | WFP 服务未启动 | 启动 `BFE` (Base Filtering Engine) 服务 |
| `0x80070006` | INVALID_HANDLE | 会话已关闭，需重新初始化 |

### 错误映射示例

```rust
match status {
    ERROR_SUCCESS.0 => Ok(()),
    0x80320009 => {
        warn!("子层已存在，继续使用");
        Ok(())
    }
    0x80070005 => Err(anyhow!(
        "访问被拒绝：请以管理员身份运行"
    )),
    _ => Err(anyhow!("WFP 操作失败：HRESULT 0x{:X}", status)),
}
```

---

## 🧪 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // 需要管理员权限
    fn test_native_wfp_engine_creation() {
        let engine = NativeWfpEngine::new().unwrap();
        assert!(engine.is_valid());
    }

    #[test]
    #[ignore]
    fn test_add_and_remove_filter() {
        let engine = NativeWfpEngine::new().unwrap();
        let filter_id = engine.add_process_filter(
            1234,
            FilterAction::Block,
            WfpLayer::AleAuthConnectV4,
            100,
        ).unwrap();
        
        engine.remove_filter(&filter_id).unwrap();
    }
}
```

### 集成测试

```bash
# 以管理员身份运行测试
cargo test --features integration-tests -- --ignored
```

---

## 🚀 部署步骤

### 1. 编译 Release 版本

```bash
# 在 Windows 上执行
cargo build --release --target x86_64-pc-windows-msvc
```

### 2. 安装为 Windows Service

```powershell
# 使用 sc.exe 创建服务
sc create NetSentinelCore binPath= "C:\Program Files\NetSentinel\netsentinel-core.exe" start= auto
sc config NetSentinelCore obj= LocalSystem
sc start NetSentinelCore
```

### 3. 验证 WFP 会话

```powershell
# 使用 netsh 查看活动过滤器
netsh advfirewall monitor show all
```

---

## 📊 性能优化

### 过滤器权重策略

| 优先级范围 | 用途 |
| :--- | :--- |
| `0xFFFF` | 紧急阻断（PCDN 检测触发） |
| `0x8000 - 0xFFFE` | 用户自定义规则 |
| `0x0001 - 0x7FFF` | 白名单规则 |
| `0x0000` | 默认放行 |

### 批量操作

避免频繁添加/删除过滤器，使用批处理方式：

```rust
// 推荐：批量添加
let filters_to_add = vec![...];
for rule in filters_to_add {
    engine.add_filter(rule)?;
}

// 定期清理过期过滤器
engine.cleanup_expired_filters()?;
```

---

## 🔮 未来扩展

### 计划功能

1. **流量统计回调**: 注册 callout 获取实时流量数据
2. **深度包检测 (DPI)**: 识别 PCDN 协议特征
3. **多网卡策略**: 针对不同网络适配器应用不同规则
4. **云规则同步**: 从云端拉取最新 PCDN 特征库

### API 扩展点

```rust
// 未来可扩展的接口
pub trait WfpExtension {
    fn register_callout(&self, callback: Box<dyn Fn(PacketInfo)>) -> Result<()>;
    fn get_traffic_stats(&self, process_id: u32) -> Result<TrafficStats>;
    fn export_rules(&self) -> Result<Vec<RuleExport>>;
}
```

---

## 📞 故障排查

### 日志级别设置

```bash
# 设置详细日志
set RUST_LOG=netsentinel_core=debug,wfp=trace
```

### 常见问题

**Q: WFP 引擎初始化失败**
```
A: 检查以下几点：
   1. 是否以管理员身份运行
   2. BFE 服务是否启动 (services.msc -> Base Filtering Engine)
   3. Windows 版本是否为 10 20H2+ 或 Windows 11
```

**Q: 过滤器不生效**
```
A: 可能原因：
   1. 过滤器权重过低，被其他规则覆盖
   2. 条件匹配不正确（检查进程 ID/路径）
   3. 作用层选择错误（ALE_AUTH_CONNECT_V4 vs ALE_AUTH_RECV_ACCEPT_V4）
```

---

## 📚 参考资料

- [Microsoft WFP Documentation](https://docs.microsoft.com/en-us/windows/win32/fwp/)
- [Windows Filtering Platform Design Guide](https://github.com/microsoft/Windows-driver-samples/tree/master/network/wfp)
- [Rust windows crate](https://github.com/microsoft/windows-rs)

---

*文档版本：v2.0 | 最后更新：2023-10-27*
