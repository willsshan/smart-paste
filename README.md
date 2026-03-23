# AI Paste

一个面向 Windows 的轻量级剪贴板历史与快速粘贴工具，基于 `Rust + Tauri 2 + 原生 Windows API` 构建。

AI Paste 的目标不是做一个庞杂的“剪贴板大全”，而是把最常用的一条链路做顺：

- 复制后自动记录历史
- `Ctrl+Shift+V` 唤起候选面板
- `↑ / ↓` 键切换候选项
- `Enter` 确认并粘贴
- `Esc` 关闭面板

当前项目仍处于早期开发阶段，但主链路已经跑通，适合继续迭代成一个真正可用的 Windows 效率工具。

## 特性

- Windows 全局热键：`Ctrl+Shift+V`
- 自动采集文本剪贴板历史
- 候选面板键盘导航
- 收藏 / 置顶历史项
- 无边框、常驻顶部的轻量浮层窗口
- Tauri 前后端通信与本地状态管理
- 基于 `windows-rs` 的原生剪贴板读写和按键注入

## 当前状态

当前仓库更接近一个可持续迭代的 MVP，而不是已经完成的正式版本。

已实现：

- 文本剪贴板轮询与去重入库
- `Ctrl+Shift+V` 唤起历史面板
- 历史列表刷新、搜索、收藏
- `Esc` 关闭面板
- `↑ / ↓` 选择历史项
- `Enter` 触发选中项粘贴流程
- 前端静态资源开发 / 构建脚本

已知限制：

- 当前仅支持文本历史，不支持图片、文件和 HTML
- 粘贴逻辑仍需继续打磨，某些输入框场景可能存在兼容性差异
- 暂无托盘、开机自启、设置页和持久化数据库
- 暂未加入隐私黑名单、敏感内容过滤等安全策略

## 技术栈

- 桌面框架：`Tauri 2`
- 核心语言：`Rust`
- 前端：原生 `HTML + CSS + JavaScript`
- Windows 集成：`windows-rs`
- 构建脚本：`Node.js`

## 项目结构

```text
.
├─ src/                    # 前端界面与交互逻辑
├─ src-tauri/              # Tauri / Rust 核心逻辑
│  ├─ src/clipboard/       # 剪贴板采集、读取与内存管理
│  ├─ src/commands/        # Tauri 命令
│  └─ src/state/           # 应用状态
├─ scripts/                # 开发与构建脚本
├─ dist/                   # 静态构建输出（默认忽略）
└─ PROJECT_PLAN.md         # 项目规划与产品方向
```

## 环境要求

建议在 Windows 11 上开发和运行。

- Node.js 18+
- Rust stable
- Cargo
- Tauri 2 开发环境
- WebView2 Runtime

如果本机还没配置 Tauri 开发环境，可以参考 Tauri 官方文档安装 Rust、Visual Studio C++ Build Tools 和 WebView2。

## 本地开发

安装 JavaScript 侧依赖（如果后续新增依赖）：

```powershell
npm install
```

启动 Tauri 开发模式：

```powershell
cargo tauri dev
```

单独启动前端静态开发服务：

```powershell
node .\scripts\dev-server.mjs
```

构建前端静态资源：

```powershell
node .\scripts\build-static.mjs
```

检查 Rust 代码是否可编译：

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
```

## 使用方式

1. 正常复制一段文本。
2. 按 `Ctrl+Shift+V` 打开历史候选面板。
3. 使用 `↑ / ↓` 选择目标项。
4. 按 `Enter` 尝试将选中项粘贴到当前光标位置。
5. 按 `Esc` 关闭面板。

## 设计方向

这个项目的长期方向是做一个“更轻、更快、更适合键盘用户”的 Windows 剪贴板工具。

计划中的后续能力包括：

- 图片 / 文件 / HTML 历史支持
- 更稳定的跨应用粘贴兼容性
- 系统托盘与开机自启
- SQLite 持久化
- 隐私黑名单与敏感内容过滤
- 搜索增强、片段收藏、纯文本粘贴

更完整的产品设想可以查看 [PROJECT_PLAN.md](./PROJECT_PLAN.md)。

## 贡献

欢迎 issue、讨论和 PR。

如果你想参与这个项目，比较适合从下面几类问题入手：

- Windows 输入框兼容性与粘贴稳定性
- 剪贴板历史持久化
- 图片 / 文件剪贴板支持
- 搜索和键盘交互体验
- 隐私保护与安全策略
- UI 细节与 Fluent 风格打磨

提交 PR 前，建议至少确认：

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
node .\scripts\build-static.mjs
```

## 开源许可

本项目使用 [MIT License](./LICENSE)。
