# v0.0.7 Release Notes

## 新功能

### 划词翻译
- 新增划词翻译功能，支持鼠标拖选文本后自动翻译
- 独立 `selection-popup` 悬浮窗口，翻译结果在鼠标附近弹出
- 支持快捷键触发（默认 `Alt+Q`，可自定义）
- 支持自动翻译模式（默认开启）和快捷键模式
- 剪贴板保护：翻译前后自动保存/恢复剪贴板内容
- 支持复制翻译结果、ESC 关闭弹窗

### 划词翻译设置
- 新增「划词」设置页面
- 可配置选项：
  - 划词翻译开关
  - 自动翻译模式
  - 快捷键（默认 Alt+Q）
  - 恢复剪贴板
  - 自动关闭时间（毫秒）

## 修复

- 修复开启划词翻译后没有弹窗的问题
- 修复弹窗首次创建时可能错过划词事件的问题
- 修复自动划词鼠标监听只处理一次消息后退出的问题
- 修复弹窗窗口边框显示问题（Windows WebView2 阴影）
- 修复弹窗内容不显示的问题（事件发送时序）
- 修复划词触发不响应的问题（模拟 Ctrl+C 改用 SendInput）

## 安装说明

### Windows
下载对应架构的安装程序：
- `translate_app_0.0.7_x64-setup.exe` — NSIS 安装程序（推荐）
- `translate_app_0.0.7_x64_en-US.msi` — MSI 安装程序

运行安装程序，按照提示完成安装。

### 从源码构建
```bash
git clone https://github.com/ChrisEvans2/translator.git
cd translator
pnpm install
pnpm tauri build
```

## 系统要求

- Windows 10/11（需要 WebView2）
- 从源码构建需要：Node.js 18+、pnpm、Rust 1.70+

## 完整更新日志

详见 [CHANGELOG.md](https://github.com/ChrisEvans2/translator/blob/main/CHANGELOG.md)
