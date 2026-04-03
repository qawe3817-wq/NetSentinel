# NetSentinel Core Service - Rust 实现指南

## 架构概述

NetSentinel 采用**分离式微内核架构**，核心服务（Core）与 UI 层完全解耦，通过 IPC（命名管道）进行安全通信。

```
┌─────────────────┐     IPC (NamedPipe)    ┌──────────────────┐
│   UI Layer      │◄──────────────────────►│   Core Service   │
│  (React/Tauri)  │     HMAC 验证消息       │     (Rust)       │
└─────────────────┘                        └────────┬─────────┘
                                                    │
                                                    ▼
                                          ┌──────────────────┐
                                          │  Windows Kernel  │
                                          │   (WFP API)      │
                                          └──────────────────┘
```

## 核心模块说明

### 1. `main.rs` - 服务入口

- **功能**: 初始化日志、配置、IPC 服务器、WFP 引擎和看门狗
- **关键特性**:
  - 异步运行时 (Tokio)
  - 优雅关闭处理
  - 模块化设计

### 2. `wfp.rs` - Windows Filtering Platform 引擎

- **功能**: 内核级网络过滤
- **关键 API**:
  - `FwpmEngineOpen0`: 打开 WFP 会话
  - `FwpmSubLayerAdd0`: 添加子层
  - `FwpmFilterAdd0`: 添加过滤器
  - `FwpmCalloutAdd0`: 注册回调
- **生产环境实现要点**:
  ```rust
  // 示例：打开 WFP 会话
  let mut engine_handle: FWPM_ENGINE_HANDLE = std::ptr::null_mut();
  let session = FWPM_SESSION0 {
      sessionKey: GUID::new(),
      displayData: FWPM_DISPLAY_DATA0 {
          name: L"NetSentinel",
          description: L"Network privacy defense",
      },
      flags: FWPM_SESSION_FLAG_DYNAMIC, // 进程终止后自动清理
      ..Default::default()
  };
  unsafe {
      FwpmEngineOpen0(
          std::ptr::null(),
          RPC_C_AUTHN_DEFAULT,
          std::ptr::null(),
          &session,
          &mut engine_handle,
      )
  };
  ```

### 3. `process.rs` - 进程监控模块

- **功能**: 实时进程网络活动监控
- **关键 API**:
  - `CreateToolhelp32Snapshot`: 枚举进程
  - `GetExtendedTcpTable`/`GetExtendedUdpTable`: 获取网络连接
  - `WinVerifyTrust`: 验证数字签名
- **行为指纹分析**:
  - PCDN 特征：上传/下载比 > 5:1，高并发连接 (>50)，目标 IP 分散
  - 正常流量：突发流量，低并发，目标 IP 集中 (CDN)

### 4. `rules.rs` - 规则引擎

- **功能**: 可视化规则编辑器后端支持
- **条件块结构**:
  ```rust
  Rule {
      name: "Block PCDN",
      conditions: vec![
          Condition { field: ProcessName, operator: Contains, value: "video" },
          Condition { field: UploadSpeed, operator: GreaterThan, value: "500KB/s" },
      ],
      action: Block { duration_secs: 300 },
  }
  ```
- **冲突检测**: 保存时分析规则优先级和条件重叠

### 5. `ipc.rs` - IPC 通信模块

- **功能**: UI 与 Core 的安全通信
- **协议**: 基于 NamedPipe 的 JSON-RPC
- **安全机制**:
  - HMAC-SHA256 消息认证
  - ACL 限制仅允许 SYSTEM 和管理员访问
  - 消息重放保护（时间戳 + 随机数）
- **消息类型**:
  - `GetProcessList` / `ProcessList`
  - `AddRule` / `RemoveRule` / `GetRules` / `RulesList`
  - `BlockProcess`
  - `GetStats` / `Stats`

### 6. `config.rs` - 配置管理

- **功能**: 持久化配置和自愈
- **配置项**:
  - IPC 管道名称和密钥
  - 看门狗设置
  - 带宽阈值
  - 默认阻断时长
- **自愈机制**: 配置损坏时自动回滚到 `config.backup.json`

## 编译与构建

### 开发环境要求

- Rust 1.75+ (edition 2021)
- Windows SDK 10.0.19041+
- Visual Studio Build Tools 2019+

### 编译命令

```bash
# 开发模式
cd core
cargo build

# 发布模式 (优化 + 裁剪)
cargo build --release

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

### 交叉编译 (可选)

```bash
# 添加目标
rustup target add x86_64-pc-windows-msvc

# 交叉编译
cargo build --release --target x86_64-pc-windows-msvc
```

## 部署为 Windows 服务

### 使用 NSSM (Non-Sucking Service Manager)

```batch
nssm install NetSentinelCore "C:\Program Files\NetSentinel\netsentinel-core.exe"
nssm set NetSentinelCore Start SERVICE_AUTO_START
nssm set NetSentinelCore DisplayName "NetSentinel Core Service"
nssm set NetSentinelCore Description "Kernel-level network privacy defense service"
nssm start NetSentinelCore
```

### 使用 sc.exe (原生工具)

```batch
sc create NetSentinelCore binPath= "C:\Program Files\NetSentinel\netsentinel-core.exe" start= auto
sc description NetSentinelCore "Kernel-level network privacy defense service"
sc start NetSentinelCore
```

## 代码签名

为防止 Windows Defender 误报，必须使用 EV Code Signing Certificate 签名：

```batch
# 使用 SignTool (Windows SDK)
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
    /f "certificate.pfx" /p "password" netsentinel-core.exe
```

## 性能优化建议

1. **WFP 会话优化**:
   - 使用 `FWPM_SESSION_FLAG_DYNAMIC` 标志，进程崩溃后自动清理
   - 合理设置子层权重，避免与其他安全软件冲突

2. **内存管理**:
   - 使用 `Vec::with_capacity()` 预分配容量
   - 避免不必要的字符串克隆

3. **异步 I/O**:
   - IPC 通信使用 Tokio 异步运行时
   - 批量处理进程列表更新（每 500ms）

4. **发布构建优化** (`Cargo.toml`):
   ```toml
   [profile.release]
   opt-level = 3
   lto = true
   strip = true
   codegen-units = 1
   ```

## 调试技巧

### 启用详细日志

```bash
# 设置环境变量
set RUST_LOG=netsentinel_core=debug
netsentinel-core.exe
```

### 使用 Windows Performance Recorder

```batch
# 录制 WFP 性能数据
wpr -start CPU
# ... 运行应用 ...
wpr -stop netsentinel.etl
```

### 查看 WFP 过滤器

```powershell
# 使用 netsh 查看活动过滤器
netsh wfp show filters
```

## 常见问题

### Q: 如何处理权限不足？

A: 核心服务应以 Windows 服务形式运行，无需用户手动管理员权限。安装程序负责注册服务。

### Q: 如何避免与杀毒软件冲突？

A: 
1. 使用动态 WFP 会话 (`FWPM_SESSION_FLAG_DYNAMIC`)
2. 合理设置子层权重（建议在 65535 附近）
3. 提交白名单申请给主流杀软厂商

### Q: 配置文件在哪里？

A: `%APPDATA%\NetSentinel\config.json`

## 下一步开发计划

1. **Phase 1**: 完成 WFP 基础过滤功能
2. **Phase 2**: 实现进程监控和行为分析
3. **Phase 3**: 完善规则引擎和冲突检测
4. **Phase 4**: 代码签名和杀软白名单提交

---

*文档版本：v2.0 | 最后更新：2023-10-27*
