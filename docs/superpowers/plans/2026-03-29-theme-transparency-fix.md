# 主题颜色和透明度修复实现计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复翻译窗口顶栏透明度问题，使顶栏背景半透明但文字和按钮保持完全清晰可读。

**Architecture:** 使用 RGBA 背景色替代 `opacity` 属性。在 `utils.ts` 添加 `hexToRgba` 工具函数，修改 `Titlebar` 组件在渲染时动态计算 RGBA 背景色，移除 CSS 变量 `--titlebar-opacity`。

**Tech Stack:** React, TypeScript, Tauri, CSS-in-JS

**Spec:** `docs/superpowers/specs/2026-03-29-theme-transparency-fix-design.md`

---

## Chunk 1: 工具函数和 CSS 变量清理

### Task 1: 添加 hexToRgba 工具函数

**Files:**
- Modify: `src/lib/utils.ts`

- [ ] **Step 1: 添加 hexToRgba 函数**

在 `src/lib/utils.ts` 文件末尾添加以下函数：

```typescript
/**
 * 将十六进制颜色转换为 RGBA 格式
 * @param hex - 十六进制颜色（如 "#426666"）
 * @param alpha - 透明度（0-1）
 * @returns RGBA 颜色字符串（如 "rgba(66, 102, 102, 0.75)"）
 */
export function hexToRgba(hex: string, alpha: number): string {
  // 移除 # 符号
  const cleanHex = hex.replace('#', '');
  
  // 解析 RGB 值
  const r = parseInt(cleanHex.slice(0, 2), 16);
  const g = parseInt(cleanHex.slice(2, 4), 16);
  const b = parseInt(cleanHex.slice(4, 6), 16);
  
  // 确保 alpha 在有效范围内
  const clampedAlpha = Math.max(0, Math.min(1, alpha));
  
  return `rgba(${r}, ${g}, ${b}, ${clampedAlpha})`;
}
```

- [ ] **Step 2: 验证函数逻辑**

手动验证：
- 输入: `hexToRgba("#426666", 0.75)`
- 预期输出: `"rgba(66, 102, 102, 0.75)"`
- 输入: `hexToRgba("#FFFFFF", 1.0)`
- 预期输出: `"rgba(255, 255, 255, 1)"`

- [ ] **Step 3: 检查 TypeScript 编译**

运行: `npm run build` 或 `tsc --noEmit`
预期: 无类型错误

- [ ] **Step 4: Commit**

```bash
git add src/lib/utils.ts
git commit -m "feat(utils): add hexToRgba color conversion function"
```

---

### Task 2: 移除 ThemeContext 中的 titlebar-opacity CSS 变量

**Files:**
- Modify: `src/contexts/ThemeContext.tsx:23-32`

- [ ] **Step 1: 修改 applyTheme 函数**

找到 `ThemeContext.tsx` 中的 `applyTheme` 函数（第 23-32 行），修改为：

```typescript
function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  // 文本区域透明度：0-100 映射到 0-1
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));
  // 移除 --titlebar-opacity，顶栏组件内部计算
}
```

**变更说明**: 删除第 31 行 `root.style.setProperty('--titlebar-opacity', String(settings.transparency / 200));`

- [ ] **Step 2: 检查 TypeScript 编译**

运行: `npm run build` 或 `tsc --noEmit`
预期: 无类型错误

- [ ] **Step 3: Commit**

```bash
git add src/contexts/ThemeContext.tsx
git commit -m "refactor(theme): remove titlebar-opacity CSS variable"
```

---

### Task 3: 移除 theme.ts 中的 titlebar-opacity CSS 变量

**Files:**
- Modify: `src/lib/theme.ts:10-17`

- [ ] **Step 1: 修改 applyTheme 函数**

找到 `theme.ts` 中的 `applyTheme` 函数（第 10-17 行），修改为：

```typescript
export function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));
  // 移除 --titlebar-opacity
}
```

**变更说明**: 删除第 16 行 `root.style.setProperty('--titlebar-opacity', String(settings.transparency / 200));`

- [ ] **Step 2: 检查 TypeScript 编译**

运行: `npm run build` 或 `tsc --noEmit`
预期: 无类型错误

- [ ] **Step 3: Commit**

```bash
git add src/lib/theme.ts
git commit -m "refactor(theme): remove titlebar-opacity from theme.ts"
```

---

## Chunk 2: Titlebar 组件修复

### Task 4: 修改 Titlebar 组件使用 RGBA 背景色

**Files:**
- Modify: `src/components/Titlebar.tsx:1,23,64-70`

- [ ] **Step 1: 导入 hexToRgba 函数**

在 `Titlebar.tsx` 顶部（第 1 行后）添加导入：

```typescript
import { hexToRgba } from '@/lib/utils';
```

完整的导入部分应该是：

```typescript
import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Settings, Copy, Minus, ChevronDown } from 'lucide-react';
import { useTheme } from '@/contexts/ThemeContext';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { hexToRgba } from '@/lib/utils';
```

- [ ] **Step 2: 计算顶栏背景 RGBA 颜色**

在 `Titlebar` 函数体内，`useTheme` 调用后（第 23 行后）添加计算逻辑：

```typescript
export function Titlebar({ onSettingsClick, onCopyClick, onEngineChange, currentEngine }: TitlebarProps) {
  const { theme } = useTheme();

  // 计算顶栏背景透明度
  // 用户设置 0% → alpha = 0.5 (50% 不透明)
  // 用户设置 100% → alpha = 1.0 (完全不透明)
  const titlebarBgAlpha = 0.5 + theme.transparency / 200;
  const titlebarBgColor = hexToRgba(theme.themeColor, titlebarBgAlpha);

  // ... 后续代码
```

- [ ] **Step 3: 修改顶栏 div 样式**

找到顶栏的 `<div>` 元素（第 64-70 行），修改 `style` 属性：

**修改前**:
```typescript
<div
  className="h-10 flex items-center px-3 select-none relative cursor-default"
  style={{ 
    backgroundColor: theme.themeColor,
    opacity: `var(--titlebar-opacity)`,
  }}
>
```

**修改后**:
```typescript
<div
  className="h-10 flex items-center px-3 select-none relative cursor-default"
  style={{ 
    backgroundColor: titlebarBgColor,
  }}
>
```

**变更说明**: 
- 将 `backgroundColor: theme.themeColor` 改为 `backgroundColor: titlebarBgColor`
- 删除 `opacity: var(--titlebar-opacity)` 行

- [ ] **Step 4: 检查 TypeScript 编译**

运行: `npm run build` 或 `tsc --noEmit`
预期: 无类型错误

- [ ] **Step 5: Commit**

```bash
git add src/components/Titlebar.tsx
git commit -m "fix(titlebar): use RGBA background color for proper transparency"
```

---

## Chunk 3: 验证和测试

### Task 5: 手动测试透明度功能

**Files:**
- None (manual testing)

- [ ] **Step 1: 启动应用**

运行: `npm run tauri dev`
预期: 应用成功启动，显示翻译窗口

- [ ] **Step 2: 测试透明度最小值 (0%)**

操作:
1. 打开设置窗口
2. 在"外观"标签下，将透明度滑块设置为 0%

预期结果:
- 翻译窗口文本区域背景完全不透明（因为公式 `1 - 0 / 100 = 1.0`）
- 翻译窗口顶栏背景 50% 不透明（alpha = 0.5）
- 顶栏文字、图标、按钮完全清晰可读

- [ ] **Step 3: 测试透明度中间值 (50%)**

操作:
1. 将透明度滑块设置为 50%

预期结果:
- 文本区域背景 50% 不透明（alpha = 0.5）
- 顶栏背景 75% 不透明（alpha = 0.75）
- 顶栏文字、图标、按钮完全清晰可读

- [ ] **Step 4: 测试透明度最大值 (100%)**

操作:
1. 将透明度滑块设置为 100%

预期结果:
- 文本区域背景完全透明（alpha = 0.0）
- 顶栏背景完全不透明（alpha = 1.0）
- 顶栏文字、图标、按钮完全清晰可读

- [ ] **Step 5: 测试主题颜色变更**

操作:
1. 在"外观"标签下，修改主题颜色为红色 (#FF0000)

预期结果:
- 翻译窗口顶栏背景变为红色（带透明度）
- 打开设置窗口，设置页面顶栏也变为红色（完全不透明）

- [ ] **Step 6: 测试设置窗口独立性**

操作:
1. 保持透明度滑块在 50%
2. 打开设置窗口

预期结果:
- 设置窗口本身完全不透明（不受透明度设置影响）
- 设置窗口顶栏使用主题颜色，完全不透明

- [ ] **Step 7: 测试极端颜色**

操作:
1. 测试黑色 (#000000)
2. 测试白色 (#FFFFFF)

预期结果:
- 两种颜色都正确显示，带相应的透明度

- [ ] **Step 8: 记录测试结果**

如果所有测试通过，继续下一步。
如果有任何测试失败，记录问题并修复。

---

### Task 6: 检查 LSP 诊断

**Files:**
- All modified files

- [ ] **Step 1: 运行 LSP 诊断**

检查所有修改的文件：
- `src/lib/utils.ts`
- `src/contexts/ThemeContext.tsx`
- `src/lib/theme.ts`
- `src/components/Titlebar.tsx`

预期: 无错误或警告

- [ ] **Step 2: 运行完整构建**

运行: `npm run build`
预期: 构建成功，无错误

- [ ] **Step 3: 最终 Commit**

```bash
git add -A
git commit -m "test: verify theme transparency fix"
```

---

## 完成清单

- [ ] 所有代码修改已完成
- [ ] TypeScript 编译无错误
- [ ] 手动测试全部通过
- [ ] LSP 诊断无问题
- [ ] 所有修改已提交到 Git

---

## 回归风险

如果修改后出现问题，检查：

1. **顶栏文字仍然透明**: 确认 `opacity` 属性已从 `<div>` 中完全移除
2. **颜色显示异常**: 检查 `hexToRgba` 函数的输入是否为有效的十六进制颜色
3. **透明度范围不正确**: 验证公式 `0.5 + transparency / 200` 是否正确应用
4. **设置窗口受影响**: 确认 `SettingsModal.tsx` 未被修改

---

## 未来改进

1. 为 `hexToRgba` 添加单元测试
2. 添加颜色格式验证
3. 使用 `useMemo` 缓存 RGBA 颜色计算结果（如果性能成为问题）
