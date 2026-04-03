# NetSentinel - Project Structure & Setup Guide

## 📁 Complete Directory Structure

```
NetSentinel/
├── .github/
│   └── workflows/
│       ├── ci.yml                 # Continuous Integration
│       └── release.yml            # Release automation
├── .gitignore                     # Git ignore rules
├── .env.example                   # Environment variables template
├── tauri.conf.json               # Tauri configuration
├── package.json                   # Root package (workspaces)
├── README.md                      # Project overview
├── LICENSE                        # MIT License
│
├── app/                           # UI Layer (React + TypeScript + Vite)
│   ├── index.html                 # HTML entry point
│   ├── package.json               # Frontend dependencies
│   ├── tsconfig.json              # TypeScript config
│   ├── tsconfig.node.json         # Node TS config
│   ├── vite.config.ts             # Vite bundler config
│   ├── tailwind.config.js         # Tailwind CSS config
│   ├── postcss.config.cjs         # PostCSS config
│   └── src/
│       ├── main.tsx               # React entry point
│       ├── App.tsx                # Main app component with routing
│       ├── components/
│       │   ├── Layout.tsx         # Main layout wrapper
│       │   ├── Sidebar.tsx        # Floating sidebar navigation
│       │   ├── TopBar.tsx         # Transparent top bar
│       │   └── ui.tsx             # Atomic UI components (Button, Card, etc.)
│       ├── hooks/
│       │   ├── index.ts           # Hook exports
│       │   ├── useTrafficWave.ts  # Real-time traffic chart hook
│       │   ├── useProcessMonitor.ts # Process monitoring hook
│       │   ├── useRuleEngine.ts   # Rule management hook
│       │   └── useSystemSettings.ts # Settings management hook
│       ├── views/
│       │   ├── Dashboard.tsx      # Command center dashboard
│       │   ├── ProcessMonitor.tsx # Process monitoring grid
│       │   ├── RuleEngine.tsx     # Visual rule editor
│       │   └── Settings.tsx       # System settings page
│       └── styles/
│           └── index.css          # Global styles + Design Tokens
│
├── core/                          # Core Service (Rust + WFP)
│   ├── Cargo.toml                 # Rust dependencies
│   ├── Cargo.lock                 # Locked dependencies
│   └── src/
│       ├── main.rs                # Core service entry
│       ├── wfp.rs                 # Windows Filtering Platform engine
│       ├── process.rs             # Process enumeration & monitoring
│       ├── rules.rs               # Rule engine with condition blocks
│       ├── ipc.rs                 # IPC via NamedPipe with HMAC
│       └── config.rs              # Configuration management
│
├── installer/                     # Installation Scripts
│   ├── installer.iss              # Inno Setup script
│   └── wix/
│       └── main.wxs               # WiX Toolset config
│
├── resources/                     # Static Assets
│   ├── icons/                     # SVG/PNG icons
│   │   ├── icon.ico               # Windows icon
│   │   ├── icon.png               # App icon
│   │   └── *.svg                  # Vector icons
│   ├── fonts/                     # Embedded fonts
│   │   └── HarmonyOS_Sans_SC/     # Primary font family
│   └── images/                    # Illustrations & backgrounds
│       ├── mesh-gradient.svg      # Dashboard background
│       └── logo.svg               # Brand logo
│
├── scripts/                       # Build & Utility Scripts
│   ├── build.ps1                  # Windows build script
│   ├── build.sh                   # Unix build script
│   ├── sign.ps1                   # Code signing script
│   └── generate-icons.ps1         # Icon generation
│
├── docs/                          # Documentation
│   ├── ARCHITECTURE.md            # Architecture deep-dive
│   ├── API.md                     # IPC API documentation
│   ├── USER_GUIDE.md              # User manual
│   ├── PRIVACY_POLICY.md          # Privacy policy
│   └── CONTRIBUTING.md            # Contribution guidelines
│
└── tests/                         # End-to-End Tests
    ├── e2e/                       # Playwright/Electron tests
    └── fixtures/                  # Test fixtures
```

## 🚀 Quick Start Guide

### Prerequisites

1. **Node.js 18+** - [Download](https://nodejs.org/)
2. **Rust 1.70+** - [Install via rustup](https://rustup.rs/)
3. **Windows 10 (20H2+) / Windows 11**
4. **Visual Studio Build Tools** (for native modules)

### Step 1: Clone Repository

```bash
git clone https://github.com/your-org/netsentinel.git
cd netsentinel
```

### Step 2: Install Dependencies

```bash
# Install root workspace dependencies
npm install

# Verify app dependencies are installed
cd app && npm install
cd ..

# Verify core builds
cd core && cargo check
cd ..
```

### Step 3: Development Mode

```bash
# Run in development mode (starts both UI and watches core)
npm run dev
```

This will:
- Start Vite dev server on `http://localhost:1420`
- Launch Tauri window with hot-reload
- Watch for changes in both UI and Rust code

### Step 4: Build Production Version

```bash
# Build for production
npm run build

# Output will be in:
# - app/dist/ (frontend bundle)
# - core/target/release/netsentinel-core.exe (core service)
# - src-tauri/target/release/NetSentinel.exe (bundled app)
```

## 🔧 Development Guidelines

### Design Token System

All colors, spacing, and animations are defined as CSS variables in `app/src/styles/index.css`:

```css
:root {
  --bg-base: #F0F2F5;
  --surface-card: rgba(255, 255, 255, 0.85);
  --primary: #3B82F6;
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

### IPC Communication

UI communicates with Core via Tauri Commands:

```typescript
// In React component
import { invoke } from '@tauri-apps/api'

const processes = await invoke('get_process_list')
await invoke('block_process', { pid: 1234, duration_secs: 600 })
```

### Adding New Pages

1. Create view component in `app/src/views/`
2. Add route in `App.tsx`
3. Add navigation item in `Sidebar.tsx`

### Adding Core Features

1. Implement logic in `core/src/`
2. Expose via IPC in `core/src/ipc.rs`
3. Create Tauri command in `src-tauri/src/main.rs`
4. Call from UI using `invoke()`

## 📊 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold Start | < 1.5s | From launch to interactive UI |
| Idle Memory (UI) | < 40MB | Task Manager - Private Working Set |
| Idle Memory (Core) | < 20MB | Task Manager - Private Working Set |
| CPU (Idle) | < 1% | Average over 60 seconds |
| FPS (Animations) | 60fps | Chrome DevTools Performance |

## 🔒 Security Checklist

- [ ] EV Code Signing Certificate applied
- [ ] HMAC validation for all IPC messages
- [ ] No user data uploaded to cloud
- [ ] Windows Defender whitelist submitted
- [ ] Firewall rules configured correctly
- [ ] NamedPipe ACL restricts access

## 🐛 Troubleshooting

### "Failed to connect to IPC"

Ensure core service is running:
```bash
# Check if pipe exists
\\.\pipe\netsentinel-core
```

### Build fails with Rust errors

Update Rust toolchain:
```bash
rustup update
```

### UI doesn't reflect theme change

Check that `data-theme` attribute is set on `<html>` element.

## 📝 Next Steps

After setup completion:
1. Review `docs/ARCHITECTURE.md` for system design
2. Read `docs/API.md` for IPC interface details
3. Follow `docs/CONTRIBUTING.md` before submitting PRs
4. Test with `npm run lint` and `npm run typecheck`

---

**Version**: 2.0.0  
**Last Updated**: 2023-10-27  
**Status**: Draft
