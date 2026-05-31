# Feature

当前版本更新记录（版本迭代后清空）

- 修复自动划词在普通点击终端/VS Code 内置终端时误触发复制的问题：鼠标自动触发改为必须检测到有效拖拽距离后才尝试复制翻译。
- 终端前台窗口复制保护：检测到 cmd/PowerShell/Windows Terminal 等终端时改用 `Ctrl+Shift+C`，并在剪贴板未变化时跳过翻译，避免中断 TUI。
- Windows 自动划词模式改为 UI Automation-only：拖拽后通过 `TextPattern.GetSelection()` 只读获取选中文本，不再模拟复制快捷键；快捷键触发模式仍保留复制兜底。
- 提升 Outlook 等富文本/WebView 控件的自动划词稳定性：扩大 UIA 父级查找深度，增加短时间重试，并允许间隔后再次翻译相同文本。
