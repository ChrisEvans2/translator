# 跨窗口主题同步系统 实现计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复两个问题：1) 主题颜色修改需要重启才能生效；2) 透明度滑动不生效。

**Architecture:** 使用 Tauri Event System 实现跨窗口通信，设置窗口修改主题后发送全局事件，翻译窗口监听事件并重新加载主题。修复透明度使用 RGBA 背景色替代 opacity 属性。

**Tech Stack:** Tauri 2 (Rust), React 18, TypeScript, @tauri-apps/api/event

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `src-tauri/src/settings.rs` | 保存设置后发送全局事件 |
| `src/contexts/ThemeContext.tsx` | 监听事件并重新加载主题 |
| `src/components/TranslationView.tsx` | 使用 RGBA 背景色修复透明度 |
| `src/lib/utils.ts` | hexToRgba 工具函数（已有，无需修改） |

---

## Task 1: Rust 后端 - 添加事件发送

**Files:**
- Modify: `D:\VScode\Work_code\translate_app\src-tauri\src\settings.rs:72-76`

- [ ] **Step 1: 修改 set_settings 函数签名，添加 AppHandle 参数**

当前代码：
```rust
#[tauri::command]
pub fn set_settings(settings: Settings) -> Result<(), String> {
```

修改为：
```rust
#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
```

- [ ] **Step 2: 在文件保存后添加事件发送**

在 `fs::write` 之后、`Ok(())` 之前添加：
```rust
    // 发送全局事件通知所有窗口
    app.emit_all("theme-changed", ()).map_err(|e| e.to_string())?;
```

完整修改后的函数：
```rust
#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    
    // 发送全局事件通知所有窗口
    app.emit_all("theme-changed", ()).map_err(|e| e.to_string())?;
    
    Ok(())
}
```

- [ ] **Step 3: LSP 诊断验证**

Run: `lsp_diagnostics` on `src-tauri/src/settings.rs`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat: emit theme-changed event after saving settings"
```

---

## Task 2: ThemeContext - 添加事件监听

**Files:**
- Modify: `D:\VScode\Work_code\translate_app\src\contexts\ThemeContext.tsx`

- [ ] **Step 1: 添加 import 语句**

在文件顶部添加：
```tsx
import { listen } from '@tauri-apps/api/event';
```

完整 import 行：
```tsx
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
```

- [ ] **Step 2: 提取主题加载逻辑为辅助函数**

在 `applyTheme` 函数之后添加：
```tsx
async function loadTheme(): Promise<ThemeSettings> {
  const settings = await invoke<any>('get_settings');
  return {
    themeColor: settings.theme_color || defaultTheme.themeColor,
    bgColor: settings.bg_color || defaultTheme.bgColor,
    textColor: settings.text_color || defaultTheme.textColor,
    transparency: settings.transparency ?? defaultTheme.transparency,
  };
}
```

- [ ] **Step 3: 重构初始加载 useEffect**

修改初始加载 useEffect 使用新的 `loadTheme` 函数：
```tsx
  // 从 Tauri 设置加载主题
  useEffect(() => {
    loadTheme().then(setTheme).catch(console.error);
  }, []);
```

- [ ] **Step 4: 添加事件监听 useEffect**

在初始加载 useEffect 之后添加：
```tsx
  // 监听主题变更事件
  useEffect(() => {
    const unlisten = listen('theme-changed', () => {
      loadTheme().then(setTheme).catch(console.error);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);
```

- [ ] **Step 5: 移除冗余 CSS 变量**

修改 `applyTheme` 函数，删除 `--transparency` 那行：
```tsx
function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  // --transparency 不再需要，TranslationView 直接使用 theme.transparency
}
```

- [ ] **Step 6: LSP 诊断验证**

Run: `lsp_diagnostics` on `src/contexts/ThemeContext.tsx`
Expected: No errors

- [ ] **Step 7: Commit**

```bash
git add src/contexts/ThemeContext.tsx
git commit -m "feat: listen for theme-changed events in ThemeProvider"
```

---

## Task 3: TranslationView - 修复透明度

**Files:**
- Modify: `D:\VScode\Work_code\translate_app\src\components\TranslationView.tsx`

- [ ] **Step 1: 添加 import 语句**

在文件顶部添加：
```tsx
import { hexToRgba } from '@/lib/utils';
```

完整 import 行：
```tsx
import { ScrollArea } from '@/components/ui/scroll-area';
import { useTheme } from '@/contexts/ThemeContext';
import { LatexRenderer } from '@/components/LatexRenderer';
import { hexToRgba } from '@/lib/utils';
```

- [ ] **Step 2: 计算带透明度的背景色**

在组件函数内部，`useTheme` 之后添加：
```tsx
  // 计算带透明度的背景色
  const bgAlpha = 1 - theme.transparency / 100;
  const bgColorWithAlpha = hexToRgba(theme.bgColor, bgAlpha);
```

- [ ] **Step 3: 修改 ScrollArea 的 style 属性**

当前代码：
```tsx
    <ScrollArea 
      className="h-[calc(100vh-2.5rem)]"
      style={{ 
        backgroundColor: theme.bgColor,
        opacity: `var(--transparency)`,
      }}
    >
```

修改为：
```tsx
    <ScrollArea 
      className="h-[calc(100vh-2.5rem)]"
      style={{ 
        backgroundColor: bgColorWithAlpha,
      }}
    >
```

- [ ] **Step 4: LSP 诊断验证**

Run: `lsp_diagnostics` on `src/components/TranslationView.tsx`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/components/TranslationView.tsx
git commit -m "fix: use RGBA background instead of opacity for transparency"
```

---

## Task 4: 构建验证

- [ ] **Step 1: TypeScript 类型检查**

Run: `pnpm tsc --noEmit`
Expected: No errors

- [ ] **Step 2: 前端构建**

Run: `pnpm build`
Expected: Build succeeds

- [ ] **Step 3: Rust 编译检查**

Run: `pnpm tauri build --debug`
Expected: No errors (或在 dev 模式下验证)
