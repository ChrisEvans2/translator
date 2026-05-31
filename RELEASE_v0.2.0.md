# v0.2.0 Release Notes

## 新功能

### Windows 自动划词改用 UI Automation
- 自动划词模式通过 Windows UI Automation 的 `TextPattern.GetSelection()` 只读获取选中文本
- 自动模式不再模拟 `Ctrl+C` 或修改剪贴板
- 快捷键触发模式继续保留复制兜底，兼容不暴露 UIA 文本选区的应用

## 优化

- 鼠标自动触发仅在检测到有效拖拽后执行，普通点击不会触发划词检测
- UIA 父级查找深度扩大，提升 Outlook 等富文本/WebView 控件的兼容性
- UIA 读取失败时短时间重试，适配选区状态延迟暴露的应用
- 设置页文案明确区分自动划词与快捷键复制模式

## 修复

- 修复自动划词误发 `Ctrl+C` 导致 PowerShell、cmd 和终端 TUI 被中断的问题
- 修复 Lightroom 等应用中拖动图片时误触发复制动作的问题
- 修复重复划选相同文本后无法再次弹出翻译的问题

## 安装说明

### Windows
下载对应架构的安装程序：
- `translate_app_0.2.0_x64-setup.exe` — NSIS 安装程序（推荐）
- `translate_app_0.2.0_x64_en-US.msi` — MSI 安装程序

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

## 补充修复

- 标题栏右键退出时会终止整个应用，避免主窗口关闭后后台进程继续运行。
