# 跨窗口主题同步系统设计

## 问题背景

用户报告两个问题：

1. **主题颜色修改需要重启才能生效**：在设置窗口修改主题颜色或背景颜色后，只有设置页面顶栏实时更新，翻译窗口需要关闭应用重新打开才能看到变化。
2. **透明度滑动不生效**：移动透明度滑块时，翻译窗口的背景透明度没有任何变化。

### 根本原因

**问题 1 根源**：
- 设置窗口和翻译窗口是两个独立的 Tauri 窗口
- 每个窗口有独立的 React 应用实例和独立的 ThemeContext
- 设置窗口修改主题时：
  - 调用 `setTheme()` 更新本窗口的 React Context ✅
  - 调用 `set_settings()` 保存到磁盘 ✅
  - 但翻译窗口的 ThemeProvider 只在启动时加载一次主题 ❌
  - 两个窗口的状态互相隔离，无跨窗口通信机制 ❌

**问题 2 根源**：
- `TranslationView.tsx` 第 19 行使用了错误的透明度应用方式：
  ```tsx
  opacity: `var(--transparency)`,  // ❌ 整个元素透明（包括文字）
  ```
- `opacity` 属性作用于整个 ScrollArea 元素及其所有子元素
- 导致文字也变透明，无法清晰显示
- 应该只让**背景**透明，文字保持不透明

---

## 设计目标

1. **实时同步**：设置窗口修改主题后，翻译窗口立即更新，无需重启
2. **透明度修复**：透明度只影响翻译窗口背景，文字始终清晰可见
3. **架构清晰**：使用 Tauri 事件系统进行跨窗口通信，符合框架最佳实践
4. **边界情况处理**：处理多窗口、快速修改、错误情况

---

## 架构设计

### 数据流

```
设置窗口修改主题
    ↓
1. 更新本窗口 React Context (setTheme)
    ↓
2. 保存到磁盘 (set_settings)
    ↓
3. Rust 发送全局事件 (theme-changed)
    ↓
4. 所有窗口收到事件通知
    ↓
5. 重新加载主题 (get_settings)
    ↓
6. 更新 React Context (setTheme)
    ↓
7. 应用 CSS 变量 (applyTheme)
    ↓
8. React 重渲染 UI
```

### 关键组件

1. **Rust 后端 (settings.rs)**：
   - `set_settings` 命令保存设置后发送全局事件
   - 使用 `app.emit_all()` 广播到所有窗口

2. **ThemeContext (ThemeContext.tsx)**：
   - 监听 `theme-changed` 事件
   - 收到事件时重新调用 `get_settings()` 并更新状态
   - cleanup 时取消监听

3. **TranslationView (TranslationView.tsx)**：
   - 使用 `hexToRgba()` 计算带透明度的背景色
   - 移除 `opacity` 属性，改用 `backgroundColor` 的 alpha 通道

---

## 详细实现

### 1. Rust 后端修改 (settings.rs)

**修改 `set_settings` 命令签名和实现**：

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

**关键点**：
- 新增参数 `app: tauri::AppHandle`（Tauri 自动注入）
- `emit_all("theme-changed", ())`：广播事件到所有窗口
- 事件 payload 为空 `()`，因为窗口会自己调用 `get_settings()` 获取最新值
- 事件发送失败返回错误（不影响设置保存）

---

### 2. ThemeContext 修改 (ThemeContext.tsx)

**导入 Tauri 事件 API**：
```tsx
import { listen } from '@tauri-apps/api/event';
```

**添加事件监听 useEffect**：
```tsx
export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<ThemeSettings>(defaultTheme);

  // 初始加载（保持不变）
  useEffect(() => {
    invoke<any>('get_settings')
      .then((settings) => {
        setTheme({
          themeColor: settings.theme_color || defaultTheme.themeColor,
          bgColor: settings.bg_color || defaultTheme.bgColor,
          textColor: settings.text_color || defaultTheme.textColor,
          transparency: settings.transparency ?? defaultTheme.transparency,
        });
      })
      .catch(console.error);
  }, []);

  // 监听主题变更事件（新增）
  useEffect(() => {
    const unlisten = listen('theme-changed', () => {
      invoke<any>('get_settings')
        .then((settings) => {
          setTheme({
            themeColor: settings.theme_color || defaultTheme.themeColor,
            bgColor: settings.bg_color || defaultTheme.bgColor,
            textColor: settings.text_color || defaultTheme.textColor,
            transparency: settings.transparency ?? defaultTheme.transparency,
          });
        })
        .catch(console.error);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  // 应用主题（保持不变）
  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}
```

**关键点**：
- `listen()` 返回 `Promise<UnlistenFn>`，需要在 cleanup 中调用
- 事件处理器重新调用 `get_settings()` 从磁盘读取最新值
- 错误处理：打印到 console，不阻塞 UI
- 两个独立的 useEffect：一个初始加载，一个监听事件

---

### 3. TranslationView 透明度修复 (TranslationView.tsx)

**导入工具函数**：
```tsx
import { hexToRgba } from '@/lib/utils';
```

**修改组件实现**：
```tsx
export function TranslationView({ translatedText, isLoading, error }: TranslationViewProps) {
  const { theme } = useTheme();
  
  // 计算带透明度的背景色
  const bgAlpha = 1 - theme.transparency / 100;
  const bgColorWithAlpha = hexToRgba(theme.bgColor, bgAlpha);

  return (
    <ScrollArea 
      className="h-[calc(100vh-2.5rem)]"
      style={{ 
        backgroundColor: bgColorWithAlpha,  // ✅ 只有背景透明
      }}
    >
      <div className="p-4 min-h-full" style={{ color: theme.textColor }}>
        {/* ... 内容保持不变 ... */}
      </div>
    </ScrollArea>
  );
}
```

**关键点**：
- 移除 `opacity` 属性
- 使用 `hexToRgba()` 计算 RGBA 背景色
- `bgAlpha` 计算：
  - 0% 透明度 → alpha = 1.0（完全不透明）
  - 100% 透明度 → alpha = 0.0（完全透明）
- 文字颜色不受影响，始终清晰

---

### 4. 移除冗余 CSS 变量 (ThemeContext.tsx)

**当前 `applyTheme` 函数**：
```tsx
function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  // 文本区域透明度：0-100 映射到 0-1
  root.style.setProperty('--transparency', String(1 - settings.transparency / 100));  // 移除这行
}
```

**修改后**：
```tsx
function applyTheme(settings: ThemeSettings) {
  const root = document.documentElement;
  root.style.setProperty('--theme-color', settings.themeColor);
  root.style.setProperty('--bg-color', settings.bgColor);
  root.style.setProperty('--text-color', settings.textColor);
  // --transparency 不再需要，TranslationView 直接使用 theme.transparency
}
```

**理由**：
- TranslationView 现在直接使用 `theme.transparency` 计算 alpha
- CSS 变量 `--transparency` 不再被使用
- 保留其他 CSS 变量（`--theme-color` 等）供其他组件使用

---

## 边界情况处理

### Rust 端

1. **事件发送失败**：
   - `emit_all()` 失败时返回错误
   - 设置已保存到磁盘，不影响数据一致性
   - 前端会在下次启动时加载最新设置

2. **无窗口监听**：
   - 如果没有窗口监听 `theme-changed` 事件，事件被忽略
   - 这是正常行为（例如只有设置窗口打开）

### 前端

1. **事件监听失败**：
   - `listen()` 失败时打印错误但不阻塞 UI
   - 窗口保留旧主题，用户重启后会加载最新值

2. **快速修改主题**：
   - 每次修改都触发事件和重新加载
   - 设置修改不频繁，性能影响可忽略
   - React 状态更新会自动批处理

3. **网络/磁盘错误**：
   - `get_settings()` 失败时保留当前主题
   - 错误打印到 console 供调试

4. **多窗口场景**：
   - 同时打开多个翻译窗口
   - 所有窗口都会收到事件并同步更新
   - 每个窗口独立调用 `get_settings()`（磁盘读取开销小）

---

## 测试验证点

### 功能测试

1. **实时同步 - 主题颜色**：
   - 打开设置窗口和翻译窗口
   - 修改主题颜色
   - 验证：设置窗口顶栏立即更新 ✅
   - 验证：翻译窗口顶栏立即更新 ✅（无需重启）

2. **实时同步 - 背景颜色**：
   - 修改背景颜色
   - 验证：翻译窗口背景立即更新 ✅

3. **实时同步 - 文本颜色**：
   - 修改文本颜色
   - 验证：翻译窗口文本颜色立即更新 ✅

4. **透明度功能**：
   - 滑动透明度滑块到 0%
   - 验证：翻译窗口背景完全不透明 ✅
   - 滑动到 100%
   - 验证：翻译窗口背景完全透明 ✅
   - 验证：文字始终清晰可见 ✅

5. **多窗口同步**：
   - 同时打开 2 个翻译窗口
   - 修改主题
   - 验证：两个翻译窗口同时更新 ✅

### 边界测试

1. **只打开设置窗口**：
   - 修改主题
   - 验证：不报错，事件正常发送 ✅

2. **快速修改主题**：
   - 连续快速修改主题颜色 5 次
   - 验证：所有窗口正确同步到最终值 ✅
   - 验证：无性能问题 ✅

3. **重启应用**：
   - 修改主题并关闭应用
   - 重新启动应用
   - 验证：所有窗口加载最新主题 ✅

---

## 文件修改清单

### 修改文件

1. **src-tauri/src/settings.rs**
   - 修改 `set_settings` 函数签名，添加 `app: tauri::AppHandle` 参数
   - 添加 `app.emit_all("theme-changed", ())` 调用

2. **src/contexts/ThemeContext.tsx**
   - 导入 `listen` from `@tauri-apps/api/event`
   - 添加监听 `theme-changed` 事件的 useEffect
   - 移除 `applyTheme` 中的 `--transparency` CSS 变量设置

3. **src/components/TranslationView.tsx**
   - 导入 `hexToRgba` from `@/lib/utils`
   - 计算 `bgColorWithAlpha`
   - 移除 `opacity` 属性，改用 `backgroundColor: bgColorWithAlpha`

### 无需修改

- **src/lib/utils.ts**：`hexToRgba` 函数已存在
- **src/components/Titlebar.tsx**：已正确使用 `hexToRgba`
- **src/components/SettingsModal.tsx**：无需修改

---

## 技术决策

### 为什么选择事件系统而非轮询？

**事件系统优势**：
- ✅ 实时响应（0 延迟）
- ✅ 无性能开销（按需触发）
- ✅ 符合 Tauri 最佳实践
- ✅ 扩展性强（未来可用于其他跨窗口通信）

**轮询方案缺点**：
- ❌ 有延迟（轮询间隔）
- ❌ 持续性能开销（定时器和文件读取）
- ❌ 不符合框架惯例

### 为什么用 RGBA 而非 opacity？

**RGBA 背景色**：
- ✅ 只影响背景，文字保持清晰
- ✅ 符合 CSS 最佳实践
- ✅ 性能更好（浏览器优化）

**opacity 属性**：
- ❌ 影响整个元素树（包括文字）
- ❌ 文字变透明不可读

### 为什么让窗口自己调用 get_settings？

**优点**：
- ✅ 事件 payload 为空，简化序列化
- ✅ 磁盘读取开销小（配置文件很小）
- ✅ 避免事件携带大量数据
- ✅ 单一数据源（磁盘），避免不一致

**替代方案**：事件携带完整设置对象
- ❌ 需要序列化/反序列化
- ❌ 事件 payload 大
- ❌ Rust 和 TypeScript 类型同步复杂

---

## 安全性考虑

1. **事件命名冲突**：
   - 使用明确的事件名 `"theme-changed"`
   - 避免与 Tauri 内置事件冲突

2. **错误处理**：
   - 所有异步操作都有 `.catch()` 处理
   - 错误不会导致应用崩溃

3. **资源清理**：
   - 事件监听器在组件卸载时清理
   - 避免内存泄漏

4. **并发修改**：
   - 设置保存是原子操作（文件写入）
   - React 状态更新自动批处理
   - 无需额外的锁机制

---

## 后续优化空间

1. **节流优化**（如果需要）：
   - 如果用户连续快速修改主题，可以在前端添加节流
   - 当前设计已足够（设置修改不频繁）

2. **事件去重**（如果需要）：
   - 检查新主题是否与当前主题相同
   - 相同则跳过重渲染
   - 当前设计已足够（React 优化已处理）

3. **持久化改进**（未来）：
   - 使用 IndexedDB 或 Tauri Store 替代文件系统
   - 当前方案已满足需求

---

## 总结

本设计通过 Tauri 事件系统实现跨窗口主题同步，修复透明度应用方式，解决用户报告的两个问题：

1. ✅ 主题修改实时生效，无需重启
2. ✅ 透明度只影响背景，文字保持清晰

架构简洁、符合框架最佳实践、边界情况处理完善。
