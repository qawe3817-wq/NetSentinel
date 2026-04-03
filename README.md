# NetSentinel

> **Silent Guardian, Visible Control.** (静默守护，可视掌控)

内核级家庭网络隐私防御工具 - 不仅阻断 PCDN，更提供系统级的网络透视能力。

## 🌟 特性

- **零感知运行**: 后台静默运行，内存占用 < 50MB
- **电影级 UI**: 现代桌面美学，支持深色/浅色模式
- **军工级稳定**: UI 崩溃不影响核心服务，支持看门狗自愈
- **智能识别**: 基于行为指纹的 PCDN 检测算法

## 🏗️ 架构

采用分离式微内核架构：

```
UI 层 (React + TypeScript + Tauri) <--IPC--> 核心服务 (Rust + WFP)
```

## 📁 目录结构

```
NetSentinel/
├── app/                    # UI 源码 (React/TypeScript)
│   ├── components/         # 原子组件
│   ├── hooks/              # 逻辑复用
│   ├── styles/             # 全局样式变量
│   └── views/              # 页面视图
├── core/                   # 核心服务 (Rust)
│   ├── src/
│   ├── Cargo.toml
│   └── tests/
├── installer/              # 安装包脚本
├── resources/              # 图标、字体、图片资源
├── scripts/                # 构建脚本
├── docs/                   # 开发文档
└── .github/workflows/      # CI/CD 配置
```

## 🚀 快速开始

### 前置要求

- Node.js 18+
- Rust 1.70+
- Windows 10 (20H2+) / Windows 11

### 安装依赖

```bash
# 安装前端依赖
cd app && npm install

# 构建核心服务
cd ../core && cargo build --release
```

### 开发模式

```bash
npm run dev
```

## 📄 许可证

[查看 LICENSE 文件](./LICENSE)

## 📞 联系

- 问题反馈：GitHub Issues
- 开发文档：`/docs` 目录
