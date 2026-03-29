# 主题颜色和透明度修复设计文档

**日期**: 2026-03-29  
**版本**: 1.0  
**状态**: 设计审查中

## 概述

本设计文档描述了翻译应用中主题颜色和透明度功能的修复方案。当前实现存在顶栏透明度问题，导致文字和按钮也变透明，影响可读性。本方案将修复此问题，确保只有背景透明，文字和按钮保持清晰。

## 需求

### 功能需求

1. **主题颜色应用范围**
   - 翻译窗口顶栏使用主题颜色
   - 设置页面顶栏使用主题颜色
   - （当前已实现，无需修改）

2. **透明度设置**
   - 透明度只影响翻译窗口（main window）
   - 设置窗口（settings window）保持完全不透明
   - 文本显示区域：透明度范围 0%-100%
   - 翻译窗口顶栏背景：透明度范围 50%-100%（当用户设置为 0% 时为 50%，设置为 100% 时为 100%）
   - 顶栏文字、图标、按钮：始终完全不透明（opacity: 1.0）

### 非功能需求

1. **可维护性**: 代码清晰，易于理解
2. **性能**: 颜色转换计算开销可忽略
3. **兼容性**: 与现有主题系统无缝集成

## 当前实现分析

### 问题诊断

**文件**: `src/components/Titlebar.tsx` (第 67-69 行)

```tsx
<div
  style={{ 
    backgroundColor: theme.themeColor,
    opacity: `var(--titlebar-opacity)`,  // ❌ 问题：整个 div 包括子元素都变透明
  }}
>
```

**问题**: 使用 `opacity` 属性会让整个元素及其所有子元素（文字、图标、按钮）都变透明，导致可读性下降。

**期望行为**: 只有背景透明，文字和按钮完全不透明。

### 现有架构

**状态管理**: `src/contexts/ThemeContext.tsx`
- `ThemeSettings` 接口定义：`themeColor`, `bgColor`, `textColor`, `transparency`
- 从 Tauri 后端加载设置
- 应用 CSS 变量到 `document.documentElement`

**CSS 变量** (当前):
- `--theme-color`: 主题颜色（十六进制）
- `--bg-color`: 背景颜色（十六进制）
- `--text-color`: 文本颜色（十六进制）
- `--transparency`: 文本区域透明度（0-1，映射自用户设置 0-100）
- `--titlebar-opacity`: 顶栏透明度（0-0.5，将被移除）

## 设计方案

### 架构决策

**选择方案 A**: 使用 RGBA 背景色替代 `opacity` 属性

**理由**:
1. 只影响背景色，不影响子元素
2. 代码修改最少（2 个文件）
3. 用户体验最佳
4. 不需要复杂的 CSS 变量转换

### 组件层级

```
翻译窗口 (Main Window)
├── Titlebar
│   ├── 背景: RGBA(theme.themeColor, alpha)  ← 半透明
│   └── 内容: 文字、图标、按钮  ← 完全不透明
└── TranslationView
    ├── 背景: theme.bgColor + opacity  ← 透明度 0%-100%
    └── 文字: theme.textColor

设置窗口 (Settings Window)
└── SettingsModal
    ├── Header
    │   └── 背景: theme.themeColor  ← 完全不透明
    └── Content  ← 完全不透明
```

### 实现细节

#### 1. 移除 CSS 变量

**文件**: `src/contexts/ThemeContext.tsx` 和 `src/lib/theme.ts`

**修改**: 移除 `--titlebar-opacity` CSS 变量设置

**修改前**:
```typescript
function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));
  root.style.setProperty('--titlebar-opacity', String(settings.transparency / 200)); // ❌ 删除
}
```

**修改后**:
```typescript
function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));
  // --titlebar-opacity 已移除，顶栏组件内部计算
}
```

#### 2. 添加颜色转换工具函数

**文件**: `src/lib/utils.ts`

**新增函数**:
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

**边界情况处理**:
- 如果 `hex` 不是有效格式，`parseInt` 返回 `NaN`，RGBA 将无效但不会崩溃
- `alpha` 被钳制在 0-1 范围内

#### 3. 修改顶栏组件

**文件**: `src/components/Titlebar.tsx`

**修改**:
```typescript
import { hexToRgba } from '@/lib/utils'; // 新增导入

export function Titlebar({ ... }: TitlebarProps) {
  const { theme } = useTheme();

  // 计算顶栏背景透明度
  // 用户设置 0% → alpha = 0.5 (50% 透明)
  // 用户设置 100% → alpha = 1.0 (完全不透明)
  const titlebarBgAlpha = 0.5 + theme.transparency / 200;
  const titlebarBgColor = hexToRgba(theme.themeColor, titlebarBgAlpha);

  return (
    <div
      className="h-10 flex items-center px-3 select-none relative cursor-default"
      style={{ 
        backgroundColor: titlebarBgColor, // ✅ 使用 RGBA 背景色
        // opacity 属性已移除
      }}
    >
      {/* 内容保持不变 */}
    </div>
  );
}
```

**透明度计算公式**:
- 公式: `alpha = 0.5 + transparency / 200`
- 映射关系:
  - `transparency = 0` → `alpha = 0.5` (50% 透明，50% 不透明)
  - `transparency = 50` → `alpha = 0.75` (25% 透明，75% 不透明)
  - `transparency = 100` → `alpha = 1.0` (完全不透明)

#### 4. 确认文本区域保持不变

**文件**: `src/components/TranslationView.tsx`

**当前实现**（无需修改）:
```typescript
<ScrollArea 
  style={{ 
    backgroundColor: theme.bgColor,
    opacity: `var(--transparency)`, // ✅ 保持使用 opacity
  }}
>
```

**说明**: 文本区域继续使用 `opacity` 属性，因为整个区域（包括文字）都需要透明。

#### 5. 确认设置窗口顶栏

**文件**: `src/components/SettingsModal.tsx`

**当前实现**（无需修改）:
```typescript
<div
  style={{ backgroundColor: theme.themeColor }} // ✅ 完全不透明
>
```

**说明**: 设置窗口顶栏不使用任何透明度，保持完全不透明。

## 数据流

```
用户调整透明度滑块 (0-100)
    ↓
ThemeContext.setTheme({ ...theme, transparency: newValue })
    ↓
保存到 Tauri 设置 (invoke 'set_settings')
    ↓
应用 CSS 变量 (applyTheme)
    ↓
组件重新渲染
    ↓
Titlebar: 计算 hexToRgba(themeColor, 0.5 + transparency/200)
TranslationView: 应用 var(--transparency)
```

## 测试计划

### 手动测试用例

| 测试用例 | 操作步骤 | 预期结果 |
|---------|---------|---------|
| TC1: 透明度最小值 | 设置透明度为 0% | 文本区域完全透明，顶栏背景 50% 透明，顶栏文字完全清晰 |
| TC2: 透明度中间值 | 设置透明度为 50% | 文本区域 50% 透明，顶栏背景 75% 不透明，顶栏文字完全清晰 |
| TC3: 透明度最大值 | 设置透明度为 100% | 文本区域完全不透明，顶栏背景完全不透明，顶栏文字完全清晰 |
| TC4: 主题颜色变更 | 修改主题颜色为红色 | 翻译窗口顶栏和设置窗口顶栏都变为红色 |
| TC5: 设置窗口独立性 | 打开设置窗口，调整透明度 | 设置窗口本身保持完全不透明 |
| TC6: 颜色格式边界 | 使用极端颜色 (#000000, #FFFFFF) | 正确显示黑色和白色 |

### 边界情况

1. **无效颜色值**: 如果 `themeColor` 不是有效的十六进制颜色，`hexToRgba` 会生成无效的 RGBA，浏览器会忽略该样式，回退到默认样式。
2. **透明度超出范围**: `ThemeSettings.transparency` 应在 0-100 范围内，由设置 UI 的滑块控件保证。
3. **性能**: 每次渲染时调用 `hexToRgba` 的开销可忽略（简单的字符串解析和数学运算）。

## 文件修改清单

| 文件 | 修改类型 | 说明 |
|-----|---------|------|
| `src/lib/utils.ts` | 新增 | 添加 `hexToRgba` 函数 |
| `src/contexts/ThemeContext.tsx` | 修改 | 移除 `--titlebar-opacity` CSS 变量 |
| `src/lib/theme.ts` | 修改 | 移除 `--titlebar-opacity` CSS 变量 |
| `src/components/Titlebar.tsx` | 修改 | 使用 `hexToRgba` 计算背景色，移除 `opacity` 属性 |

**未修改文件**:
- `src/components/TranslationView.tsx` - 保持使用 `opacity`
- `src/components/SettingsModal.tsx` - 已正确实现
- Tauri 配置文件 - 无需修改

## 风险和缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|-----|------|------|---------|
| 颜色转换计算错误 | 显示异常颜色 | 低 | 添加单元测试验证 `hexToRgba` 函数 |
| 透明度公式错误 | 透明度范围不符合预期 | 低 | 手动测试边界值 (0%, 50%, 100%) |
| 性能影响 | 渲染性能下降 | 极低 | 颜色转换为轻量级计算 |

## 未来考虑

1. **颜色格式验证**: 添加输入验证确保 `themeColor` 是有效的十六进制颜色
2. **性能优化**: 如果 `themeColor` 和 `transparency` 频繁变化，可以使用 `useMemo` 缓存计算结果
3. **可访问性**: 考虑在高透明度时自动调整文字颜色以保持对比度

## 总结

本设计通过使用 RGBA 背景色替代 `opacity` 属性，解决了顶栏文字透明的问题。修改最少（4 个文件），风险低，用户体验显著提升。实现后，顶栏背景将正确地半透明，而文字和按钮保持完全清晰可读。
