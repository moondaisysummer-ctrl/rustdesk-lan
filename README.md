# rustdesk-lan

基于 [RustDesk](https://github.com/rustdesk/rustdesk) 开源项目修改的**局域网专用精简版**。仅支持同一局域网内通过裸 IP 直连的远程桌面与文件传输，无需自建或连接任何注册/中继服务器，开箱即用。

本项目与上游 RustDesk 的关系：仅在上游代码基础上做裁剪与优化，核心协议与架构保持不变。许可证与上游一致，沿用 [AGPL-3.0](LICENCE)。

## 相对上游的改动与优化

### 1. 仅限局域网直连

- 删除上游 rendezvous/relay 全流程（ID 注册、NAT 打洞、中继转发），新增 `src/lan_server.rs` 提供局域网 TCP 直连监听
- 主控端直接输入被控端局域网 IP 即可发起连接，ID 方式连接已彻底禁用，不产生任何外部网络依赖
- 连接为局域网内明文 TCP，无密钥交换与中继转发开销，安全性依赖局域网环境
- 同步移除依赖服务器的配套功能：账号登录、2FA、通讯录/群组、OIDC、自定义服务器、自动更新

### 2. 传输通道精简

- 移除 KCP（UDP 可靠传输）通道及其连接维护逻辑，仅保留 TCP 直连

### 3. 端口迁移

为避免与上游 RustDesk 及其服务器部署冲突，全部端口整体迁移：

| 用途 | 上游端口 | 本项目端口 |
|---|---|---|
| 基座服务（UDP/TCP） | 21116 | 31116 |
| 中继（保留定义，本版未使用） | 21117 | 31117 |
| 直连 TCP（服务端监听 / 客户端目标） | 21118 | 31118 |
| UDP 广播发现 | 21119 | 31119 |

两端保持一致：被控端监听 31118/TCP 与 31119/UDP，主控端裸 IP 直连默认目标 31118。

### 4. 认证简化：仅固定密码

- 去除"一次性密码""同时使用两种密码"等选项，UI 与认证逻辑只保留**固定密码**
- 默认固定密码：`12qwaszx`（首次启动自动写入，可在设置页修改）
- 若上游旧配置残留了其它验证方式，会自动归一为固定密码

### 5. 文件传输提速

- 修复稳态传输时每毫秒一次的全量任务序列化 + 跨进程 IPC + Flutter 事件推送（两方向合计约 2000 事件/秒）导致的 UI 线程争抢与速率下降
- 进度日志节流为 200ms 一次（5 次/秒），进度条刷新肉眼无感；传输完成/失败/取消状态仍实时推送，不受节流影响
- 数据通道本身保持上游设计：128KB 分块、zstd-3 压缩、1ms 驱动，理论上限高于千兆网线速

### 6. 体积精简

- 移除 Android/iOS 移动端工程及全部移动端构建脚本
- 界面语言由上游 40+ 种精简为中文/英文
- 删除上游社区文档（docs/ 下的多语言 README、贡献指南等）

### 7. 命名与打包

- 全部产物统一命名 `rustdesk-lan`，不带版本号
- 提供两种交付形态：
  - `rustdesk-lan.deb`（Debian 系安装包）
  - `rustdesk-lan.AppImage`（通用免安装格式，x86_64）

## 使用

1. 被控端安装并运行 rustdesk-lan，保持服务在线
2. 主控端在 IP 输入框直接填被控端局域网 IP，输入固定密码即可连接
3. 文件传输在连接工具栏选择"文件传输"，左右两侧分别对应本机与远端目录

## 构建（Linux x86_64）

依赖 Rust、Flutter 3.24.x 与 Node 侧工具链同上游要求：

```bash
# 1. 编译 Rust 库
cargo build --lib --release --features flutter

# 2. 编译 Flutter 应用（会自动带入 liblibrustdesk.so 并改名为 librustdesk.so）
cd flutter && flutter build linux --release && cd ..

# 3. 组装 deb
dpkg-deb --build --root-owner-group /tmp/rddeb rustdesk-lan.deb

# 4. 构建 AppImage
cd appimage
PATH=/tmp/aib-bin:$HOME/.local/bin:$PATH appimage-builder --recipe AppImageBuilder-x86_64.yml --skip-test
mv rustdesk-lan-1.5.0-x86_64.AppImage rustdesk-lan.AppImage
```

## 项目结构

保留上游主要布局（详见上游仓库）：

- `src/` Rust 主程序（`src/server/` 网络/服务，`src/platform/` 平台相关，`src/lan_server.rs` 本项目新增的局域网直连服务）
- `flutter/` 当前 UI（`lib/desktop/` 桌面端、`lib/common/` 共享组件）
- `libs/hbb_common/` 与服务端共享的基础库（协议、`Config`）
- `libs/base/` 客户端基础库（选项键、文件传输 `fs.rs` 等）
- `libs/scrap/` 屏幕捕获；`libs/enigo/` 输入控制；`libs/clipboard/` 剪贴板
- `appimage/` AppImage/deb 打包配置

仓库内保留的辅助文档：

- `性能优化.md` — 性能瓶颈分析与潜在优化点清单（尚未实施，供后续决策参考）
- `AGENTS.md` / `CLAUDE.md` — 开发规范

## 许可证

本项目基于 [RustDesk](https://github.com/rustdesk/rustdesk) 修改，遵循其原始许可证 [GNU AGPL-3.0](LICENCE)。
