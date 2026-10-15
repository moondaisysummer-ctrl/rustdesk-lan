# rustdesk-lan

**English** | [简体中文](README.zh-CN.md)

A **LAN-only slim fork** of the open-source [RustDesk](https://github.com/rustdesk/rustdesk) project. It supports remote desktop and file transfer only within the same local network, connecting directly by bare IP address — no rendezvous or relay server to run or connect to, works out of the box.

Relationship to upstream RustDesk: this project only trims and optimizes the upstream code base; the core protocol and architecture are unchanged. Licensing is identical to upstream, under [AGPL-3.0](LICENCE).

## Repositories and downloads

| Site | Repository | Installers (nightly builds) |
|---|---|---|
| GitHub | https://github.com/moondaisysummer-ctrl/rustdesk-lan | https://github.com/moondaisysummer-ctrl/rustdesk-lan/releases/tag/nightly |
| Gitee | https://gitee.com/moondaisy/rustdesk-lan | https://gitee.com/moondaisy/rustdesk-lan/releases/tag/nightly |

Available packages:

- Windows x86_64: `exe` / `msi`
- macOS aarch64 (Apple Silicon, requires macOS 12.3+): `dmg` (ad-hoc signed, not notarized; see the notes on the Release page for first-launch instructions)
- Linux x86_64 / aarch64: `deb` / `rpm` / `AppImage` / `pkg.tar.zst`
- Linux sciter slim build: x86_64 / armv7 `deb`

## Changes and optimizations vs. upstream

### 1. LAN-only direct connection

- Removed the entire upstream rendezvous/relay pipeline (ID registration, NAT hole punching, relay forwarding) and added `src/lan_server.rs` to provide a LAN TCP direct-connect listener
- The controller connects by entering the controlled machine's LAN IP directly; ID-based connections are fully disabled and no external network dependency is created
- Connections are plain TCP within the LAN: no key exchange and no relay overhead, with security relying on the LAN environment
- Server-dependent features removed accordingly: account login, 2FA, address book/groups, OIDC, custom server, auto-update

### 2. Simplified transport

- Removed the KCP (reliable UDP) channel and its connection maintenance logic; only TCP direct connection remains

### 3. Port migration

All ports have been shifted as a whole to avoid conflicts with upstream RustDesk and its server deployments. Since rendezvous/relay have been removed, **only 31118/TCP and 31119/UDP are in use at runtime**; the rest are constant definitions only:

| Purpose | Upstream port | This project | In use |
|---|---|---|---|
| Rendezvous service (UDP/TCP) | 21116 | 31116 | No |
| Relay | 21117 | 31117 | No |
| Direct TCP (server listen / client target) | 21118 | 31118 | **Yes** |
| UDP broadcast discovery | 21119 | 31119 | **Yes** |

Firewall and deployment docs only need to care about **31118/TCP** (all connections, including manual cross-subnet IP connections); add **31119/UDP** if LAN auto-discovery is wanted. 31116 / 31117 are never listened on or connected to, so they need no rule.

Both sides stay consistent: the controlled machine listens on 31118/TCP and 31119/UDP, and the controller's bare-IP direct connection defaults to target port 31118.

### 4. Simplified authentication: permanent password only

- Removed the "one-time password" and "use both passwords" options; both the UI and the auth logic keep only the **permanent password**
- Default permanent password: `12qwaszx` (written automatically on first launch, changeable in settings)
- If leftover upstream config specifies another verification method, it is normalized to the permanent password

### 5. Faster file transfer

- Fixed the per-millisecond full task serialization + cross-process IPC + Flutter event push during steady-state transfer (about 2000 events/second in both directions combined), which caused UI thread contention and throughput drops
- Progress logging is throttled to once per 200 ms (5 times/second); the progress bar refresh is visually unaffected, while completed/failed/canceled states are still pushed in real time and are not throttled
- The data channel itself keeps the upstream design: 128 KB chunks, zstd-3 compression, 1 ms drive loop — a theoretical ceiling above gigabit line rate

### 6. Smaller footprint

- Removed the Android/iOS mobile projects and all mobile build scripts
- UI languages reduced from 40+ upstream to Chinese/English
- Removed upstream community docs (multilingual READMEs, contribution guides, etc. under docs/)

### 7. Naming and packaging

- All artifacts are uniformly named `rustdesk-lan` (with version and platform suffix, e.g. `rustdesk-lan-1.5.0-x86_64.deb`)
- Covers Windows / macOS / Linux; see "Repositories and downloads" above for delivery forms and download links

### 8. Connectable while running in the background

- Disabled the upstream "only allow connections when the RustDesk window is open" restriction (forced to `N` at startup); connections no longer fail with `The main window is not open` when the main window is closed
- Combined with the tray resident mode and launch-at-login on all three platforms, the controlled machine works right after installation — no need to open the UI

## Usage

1. Install and run rustdesk-lan on the controlled machine and keep its service online (on Windows/Linux it starts at login by default, and all three platforms stay in the tray; **no need to open the main window to be connected**)
2. On the controller, enter the controlled machine's LAN IP into the IP field and connect with the permanent password
3. Other machines running rustdesk-lan on the same subnet appear automatically in the "Discovered peers" tab, shown by their LAN IP; click to connect directly (based on 31119/UDP broadcast discovery, which does not work across subnets)
4. For file transfer, choose "File transfer" in the session toolbar; the left and right panes are the local and remote directories respectively

## Build (Linux x86_64)

Rust, Flutter 3.24.x and the Node-side toolchain requirements are the same as upstream:

```bash
# 1. Build the Rust library
cargo build --lib --release --features flutter

# 2. Build the Flutter app (it bundles liblibrustdesk.so and renames it to librustdesk.so)
cd flutter && flutter build linux --release && cd ..

# 3. Assemble the deb
dpkg-deb --build --root-owner-group /tmp/rddeb rustdesk-lan.deb

# 4. Build the AppImage
cd appimage
PATH=/tmp/aib-bin:$HOME/.local/bin:$PATH appimage-builder --recipe AppImageBuilder-x86_64.yml --skip-test
mv rustdesk-lan-1.5.0-x86_64.AppImage rustdesk-lan.AppImage
```

## Project layout

The main upstream layout is preserved (see the upstream repository for details):

- `src/` Rust main program (`src/server/` networking/services, `src/platform/` platform code, `src/lan_server.rs` the LAN direct-connect service added by this project)
- `flutter/` current UI (`lib/desktop/` desktop, `lib/common/` shared widgets)
- `libs/hbb_common/` base library shared with the server (protocol, `Config`)
- `libs/base/` client base library (option keys, file transfer `fs.rs`, etc.)
- `libs/scrap/` screen capture; `libs/enigo/` input control; `libs/clipboard/` clipboard
- `appimage/` AppImage/deb packaging config

Auxiliary documents kept in the repository:

- `性能优化.md` — performance bottleneck analysis and list of potential optimizations (not yet implemented; for future decisions)
- `AGENTS.md` / `CLAUDE.md` — development guidelines

## License

This project is modified from [RustDesk](https://github.com/rustdesk/rustdesk) and follows its original license, [GNU AGPL-3.0](LICENCE).
