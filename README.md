# WhatsApp Tauri

A lightweight desktop wrapper for [WhatsApp Web](https://web.whatsapp.com) built with **[Tauri v2](https://tauri.app)** and **Rust**.

Unlike Electron-based alternatives, this app uses the **operating system's native WebView** (WebView2 on Windows 11) instead of bundling a full Chromium runtime.  
That single decision cuts idle RAM usage from ~200 MB (Electron) down to **~30–50 MB** and shrinks the installer from ~100 MB to under **5 MB**.

---

## Table of Contents

1. [Architecture overview](#architecture-overview)
2. [Key design choices](#key-design-choices)
3. [Security model](#security-model)
4. [Resource optimisations](#resource-optimisations)
5. [Prerequisites](#prerequisites)
6. [Getting started](#getting-started)
7. [Building for production](#building-for-production)
8. [Project structure](#project-structure)
9. [Extending the app](#extending-the-app)
10. [Troubleshooting](#troubleshooting)

---

## Architecture overview

```
┌─────────────────────────────────────────────────────┐
│                    Windows 11                        │
│                                                      │
│  ┌───────────────────────────────────────────────┐  │
│  │              Tauri v2 process                 │  │
│  │  ┌─────────────┐    ┌────────────────────┐   │  │
│  │  │  Rust core  │◄──►│  WebView2 (OS)     │   │  │
│  │  │  (tiny bin) │    │  web.whatsapp.com  │   │  │
│  │  └─────────────┘    └────────────────────┘   │  │
│  │        │                                      │  │
│  │  ┌─────▼──────────────────────────────────┐  │  │
│  │  │  System tray  ·  window management     │  │  │
│  │  └────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

The Rust core is intentionally minimal: it creates the window, sets the user-agent string, and wires up the system-tray icon.  
All WhatsApp logic runs inside the WebView, served directly by WhatsApp's servers.

---

## Key design choices

| Decision | Choice | Why |
|---|---|---|
| **Framework** | Tauri v2 | Native WebView, tiny binary, Rust safety |
| **Frontend runtime** | OS WebView2 (Windows) | No bundled Chromium → much less RAM/disk |
| **Content** | `https://web.whatsapp.com` loaded directly | No local relay server, no extra overhead |
| **User-agent** | Chrome 124 / Windows 11 | WhatsApp Web only supports Chromium-based browsers |
| **Tray icon** | Hide-to-tray on close | Background notifications without extra process |
| **IPC surface** | `core:default` only | Minimum attack surface; no custom JS↔Rust bridge needed |
| **Devtools** | Disabled in release | Prevents JS console access to Tauri internals |
| **Panic handler** | `panic = "abort"` | Smaller binary, faster failure |

---

## Security model

### What Tauri provides

* **Process isolation** – the WebView runs in a separate process from the Rust host; a compromised page cannot access Rust APIs unless you explicitly expose a Tauri command.
* **Capability system** – `src-tauri/capabilities/default.json` lists every IPC permission granted to the window.  The default is `core:default` only – no filesystem, shell, HTTP, or clipboard access from JavaScript.
* **`freezePrototype: true`** – prevents JS prototype-pollution attacks inside Tauri-served pages.
* **`devtools: false`** in release builds – hides the DevTools entry point from the production binary.

### What you should know

* The window loads an **external HTTPS URL**.  WhatsApp's own TLS, certificate-pinning and server-side security apply.
* Tauri does **not** inject any custom JavaScript into external URLs by default.
* WebView2 on Windows 11 enforces its own sandboxing and auto-updates through Windows Update.
* No end-to-end messages are processed locally by Rust; everything is handled by WhatsApp's web client inside the WebView.

### Hardening checklist

- [x] Disabled devtools in release (`"devtools": false`)
- [x] Frozen JS prototype (`"freezePrototype": true`)
- [x] Minimal IPC capability set (`core:default` only)
- [x] `windows_subsystem = "windows"` hides console window in release
- [x] `panic = "abort"` removes panic-unwinding machinery
- [ ] *(optional)* Enable Windows Defender Application Control policy
- [ ] *(optional)* Code-sign the installer with an EV certificate

---

## Resource optimisations

### Binary / installer size

| Setting | Effect |
|---|---|
| `opt-level = "s"` | Optimise for size over speed |
| `lto = true` | Dead-code elimination across crates |
| `codegen-units = 1` | Better inlining at link time |
| `strip = true` | Strip debug symbols from release binary |
| `panic = "abort"` | Remove panic-unwinding tables (~100 kB) |

### Runtime memory

| Technique | Savings |
|---|---|
| System WebView2 instead of bundled Chromium | ~150 MB RAM, ~90 MB disk |
| Single window, no plugins | Minimal Rust heap usage |
| Hide-to-tray instead of keeping a second process | No duplicate processes |
| Window starts hidden until page loads | No blank-frame allocation |

### Startup time

The window is created with `visible(false)` and becomes visible once the WhatsApp Web page has loaded, avoiding a visible white-flash on slow connections.

---

## Prerequisites

| Tool | Minimum version | Notes |
|---|---|---|
| [Rust + Cargo](https://rustup.rs) | 1.77 | Install via `rustup` |
| [Node.js](https://nodejs.org) | 18 LTS | Only needed for the Tauri CLI |
| [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) | Any | Pre-installed on Windows 11 |
| [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) | 2022 | C++ workload required for Rust on Windows |

Install the Tauri CLI once:

```bash
npm install
```

---

## Getting started

```bash
# Clone the repository
git clone https://github.com/Amendrap/whatsapp_tauri.git
cd whatsapp_tauri

# Install the Tauri CLI (only needed once)
npm install

# Start the app in development mode
npm run dev
```

`npm run dev` launches the app and opens WhatsApp Web inside the native window.  
DevTools are available in dev mode via **Right-click → Inspect** or `F12`.

---

## Building for production

```bash
npm run build
```

Tauri compiles the Rust binary with release optimisations and bundles an NSIS installer under `src-tauri/target/release/bundle/nsis/`.

> **Icon note:** Replace the placeholder icons in `src-tauri/icons/` with real artwork before building for distribution.  
> Run `npx tauri icon path/to/source-1024x1024.png` to auto-generate all required sizes.

---

## Project structure

```
whatsapp_tauri/
├── index.html                  # Fallback page (not shown; window loads WhatsApp Web)
├── package.json                # Tauri CLI dev-dependency
├── src-tauri/
│   ├── build.rs                # Tauri build script
│   ├── Cargo.toml              # Rust dependencies + release profile
│   ├── tauri.conf.json         # Window, security and bundle configuration
│   ├── capabilities/
│   │   └── default.json        # IPC permissions granted to the main window
│   ├── icons/                  # App icons (replace with real artwork)
│   └── src/
│       ├── main.rs             # Binary entry point (calls lib.rs)
│       └── lib.rs              # App setup: window, tray, user-agent, events
└── README.md
```

---

## Extending the app

### System notifications

WhatsApp Web fires Web Notification API requests.  WebView2 forwards them to the OS notification centre automatically — no extra Tauri plugin required.

### Auto-launch on login

Add the [`tauri-plugin-autostart`](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/autostart) plugin:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-autostart = "2"
```

```rust
// src-tauri/src/lib.rs  — inside Builder::default()
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    Some(vec!["--flag"]),
))
```

### Badge / unread count

Inject a small JS snippet via `webview.evaluate_script()` to observe the `<title>` element changes (`"(3) WhatsApp"`) and update a tray badge.

### Custom keyboard shortcuts

Use `tauri::GlobalShortcutManager` (available via `tauri-plugin-global-shortcut`) to register system-wide hotkeys such as `Ctrl+Shift+W` to toggle the window.

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| *"WhatsApp works only on Google Chrome"* | The release build sets a Chrome user-agent automatically; in dev mode open DevTools and verify the UA string. |
| White flash on startup | Expected in dev mode; the `visible(false)` trick applies in release. |
| Tray icon missing | Ensure your Windows theme is not hiding all tray icons — check *Settings → Taskbar → Other system tray icons*. |
| WebView2 not found | Install the [WebView2 Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). The installer handles this automatically via `downloadBootstrapper`. |
| Build fails: `LINK : fatal error` | Install **Desktop development with C++** workload in Visual Studio Build Tools. |
