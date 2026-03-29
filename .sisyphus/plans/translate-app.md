# 翻译软件 (Clipboard Translation App)

## TL;DR

> **Quick Summary**: 基于 Tauri v2 + React + shadcn/ui 的桌面翻译工具，核心功能是监听剪切板自动翻译并正确渲染 LaTeX 公式。支持百度翻译、谷歌翻译、硅基流动 API 和 Ollama 本地大模型四种翻译引擎。
> 
> **Deliverables**:
> - 可运行的桌面应用 (Windows .exe)
> - 自定义深色主题 UI (shadcn/ui)
> - 剪切板监听 + 自动翻译功能
> - LaTeX 公式渲染 (KaTeX)
> - 设置面板 (通用/外观/引擎配置)
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 → Task 4 → Task 6 → Task 10 → Task 14 → Task 18 → F1-F4

---

## Context

### Original Request
开发一个翻译软件：
1. 核心功能：监听剪切板自动翻译，正确显示 LaTeX 公式
2. 翻译引擎：百度翻译、谷歌翻译、硅基流动 API、Ollama 本地
3. 百度翻译优先 APP ID + API Key 接入
4. 谷歌翻译优先 URL 镜像接入
5. UI：顶栏 #426666 可拖动，左侧设置按钮，右侧引擎选择/复制/最小化关闭
6. 翻译界面 #3f3f3f
7. 设置页面 #212121，三标签页（通用、外观、引擎）
8. 外观设置：主题色、背景色、文本色、透明度滑块
9. 技术栈：Rust + Tauri + React + Vite + Tailwind CSS + shadcn/ui

### Interview Summary
**Key Discussions**:
- 顶栏颜色确认为 #426666
- LaTeX 渲染使用 KaTeX（轻量快速）
- 最小化到任务栏（非系统托盘）
- UI 语言仅中文（目标用户为中国人）
- 仅显示译文，不显示原文
- 不需要历史记录功能
- 不需要全局快捷键
- 不需要自动化测试（依赖 Agent QA 场景验证）

**Research Findings**:
- Tauri v2 项目结构：`src/` (前端) + `src-tauri/` (Rust后端)
- 剪切板监听：推荐 `tauri-plugin-clipboard-x` 事件驱动，备选 `arboard` 轮询
- 百度翻译：MD5 签名鉴权，5 QPS 限制
- 谷歌翻译：v2 API Key 或镜像 URL
- SiliconFlow：Bearer Token，OpenAI 兼容格式
- Ollama：本地 `http://localhost:11434/api/chat`

### Metis Review
**Identified Gaps** (addressed):
- LaTeX 翻译策略：先提取 LaTeX → 翻译周围文本 → 重新插入
- 剪切板防抖：300ms 防抖避免频繁触发
- 错误处理：网络错误、API 凭证错误、LaTeX 解析错误的 UI 反馈
- 窗口行为：无系统装饰（frameless），自定义拖动区域
- 重复检测：跳过与上次相同的剪切板内容

---

## Work Objectives

### Core Objective
构建一个桌面翻译工具，实现剪切板监听、多引擎翻译、LaTeX 渲染和深色主题自定义界面。

### Concrete Deliverables
- `src-tauri/`: Rust 后端代码（剪切板监听、翻译引擎调用、设置持久化）
- `src/`: React 前端代码（UI 组件、主题系统）
- `src-tauri/tauri.conf.json`: Tauri 配置（无边框窗口、插件）
- `src/components/ui/`: shadcn/ui 组件

### Definition of Done
- [ ] `pnpm tauri dev` 成功启动应用
- [ ] `pnpm tauri build` 成功生成 Windows 可执行文件
- [ ] 复制文本后 2 秒内显示翻译结果
- [ ] LaTeX 公式正确渲染（KaTeX 样式可见）
- [ ] 四个翻译引擎均可配置和切换
- [ ] 设置持久化（重启后保留）

### Must Have
- 剪切板监听（事件驱动或轮询）
- 四个翻译引擎（百度、谷歌、硅基流动、Ollama）
- LaTeX 公式渲染（KaTeX）
- 自定义标题栏（可拖动）
- 设置面板三标签页
- CSS 变量主题系统

### Must NOT Have (Guardrails)
- ❌ 翻译历史记录
- ❌ 全局快捷键
- ❌ 原文显示（仅显示译文）
- ❌ 系统托盘功能
- ❌ 自动化单元测试文件
- ❌ 除四个引擎外的其他翻译引擎
- ❌ 字体大小/字体族自定义（仅颜色和透明度）
- ❌ 多窗口/标签页

### Future Roadmap (TODO — 当前版本不实现)
> 以下功能预留为未来开发计划，当前版本仅完善纯文本翻译。

- 📋 **OCR / 图片翻译**
  - 截图或从剪切板读取图片
  - 使用 OCR 引擎（如 Tesseract、PaddleOCR 或云端 API）识别文字
  - 将识别结果送入翻译流程
  - 技术考量：需要额外的 Rust crate 或调用外部进程

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (新项目)
- **Automated tests**: None（依赖 Agent QA 场景验证）
- **Framework**: N/A

### QA Policy
每个任务必须包含 Agent 执行的 QA 场景，证据保存到 `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`。

- **Frontend/UI**: Playwright — 导航、交互、断言 DOM、截图
- **TUI/CLI**: interactive_bash (tmux) — 运行命令、验证输出
- **API/Backend**: Bash (curl) — 发送请求、断言状态码和响应字段

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — foundation):
├── Task 1: 项目脚手架初始化 [quick]
├── Task 2: Tailwind + shadcn/ui 配置 [quick]
├── Task 3: 无边框窗口配置 [quick]
└── Task 4: 设置存储模块 (Rust) [unspecified-high]

Wave 2 (After Wave 1 — core modules):
├── Task 5: CSS 变量主题系统 [quick]
├── Task 6: 翻译引擎接口定义 [quick]
├── Task 7: 百度翻译引擎实现 [unspecified-high]
├── Task 8: 谷歌翻译引擎实现 [unspecified-high]
├── Task 9: 硅基流动引擎实现 [unspecified-high]
└── Task 10: Ollama 引擎实现 [unspecified-high]

Wave 3 (After Wave 2 — clipboard & LaTeX):
├── Task 11: 剪切板监听模块 [deep]
├── Task 12: LaTeX 检测与提取 [unspecified-high]
└── Task 13: LaTeX 渲染组件 [visual-engineering]

Wave 4 (After Wave 3 — UI components):
├── Task 14: 自定义标题栏组件 [visual-engineering]
├── Task 15: 翻译主视图组件 [visual-engineering]
├── Task 16: 设置弹窗 - 通用标签页 [visual-engineering]
├── Task 17: 设置弹窗 - 外观标签页 [visual-engineering]
├── Task 18: 设置弹窗 - 引擎标签页 [visual-engineering]
└── Task 19: 引擎选择下拉框 [visual-engineering]

Wave 5 (After Wave 4 — integration):
├── Task 20: 剪切板→翻译→渲染 Pipeline [deep]
├── Task 21: 复制译文功能 [quick]
└── Task 22: 最小化/关闭按钮行为 [quick]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── F1: 计划符合性审计 (oracle)
├── F2: 代码质量审查 (unspecified-high)
├── F3: 真实 QA 验证 (unspecified-high)
└── F4: 范围保真检查 (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 4 → Task 6 → Task 7 → Task 11 → Task 13 → Task 20 → F1-F4
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 6 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|------------|--------|
| 1 | — | 2, 3, 4, 5, 6 |
| 2 | 1 | 13, 14, 15, 16, 17, 18, 19 |
| 3 | 1 | 14, 22 |
| 4 | 1 | 5, 7-10, 16, 17, 18, 20 |
| 5 | 4 | 13, 14, 15, 16, 17 |
| 6 | 1 | 7, 8, 9, 10, 20 |
| 7-10 | 6 | 20 |
| 11 | 1 | 20 |
| 12 | 1 | 13, 20 |
| 13 | 2, 5, 12 | 15, 20 |
| 14 | 2, 3, 5 | 22 |
| 15 | 2, 5, 13 | 20 |
| 16-18 | 2, 4, 5 | — |
| 19 | 2 | 14 |
| 20 | 4, 7-10, 11, 13, 15 | F1-F4 |
| 21 | 15 | — |
| 22 | 3, 14 | — |

### Agent Dispatch Summary

| Wave | Tasks | Categories |
|------|-------|------------|
| 1 | 4 | T1-T3 → `quick`, T4 → `unspecified-high` |
| 2 | 6 | T5-T6 → `quick`, T7-T10 → `unspecified-high` |
| 3 | 3 | T11 → `deep`, T12 → `unspecified-high`, T13 → `visual-engineering` |
| 4 | 6 | T14-T19 → `visual-engineering` |
| 5 | 3 | T20 → `deep`, T21-T22 → `quick` |
| FINAL | 4 | F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep` |

---

## TODOs

- [x] 1. 项目脚手架初始化

  **What to do**:
  - 使用 `npm create tauri-app@latest` 创建项目，选择 React + TypeScript + Vite 模板
  - 验证项目结构：`src/` (前端) + `src-tauri/` (Rust后端)
  - 运行 `pnpm install` 安装依赖
  - 运行 `pnpm tauri dev` 验证基础应用可启动

  **Must NOT do**:
  - 不要添加任何业务代码
  - 不要修改默认模板内容

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单一脚手架任务，标准化流程
  - **Skills**: `[]`
    - 无需特殊技能

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (Wave 1 起点)
  - **Blocks**: Tasks 2, 3, 4, 5, 6, 7
  - **Blocked By**: None

  **References**:
  - Official docs: https://v2.tauri.app/start/create-project/
  - Tauri v2 + React template pattern

  **Acceptance Criteria**:
  - [ ] `src/` 目录存在且包含 `main.tsx`, `App.tsx`
  - [ ] `src-tauri/` 目录存在且包含 `Cargo.toml`, `tauri.conf.json`
  - [ ] `pnpm tauri dev` 成功启动应用窗口

  **QA Scenarios**:
  ```
  Scenario: 项目初始化成功
    Tool: Bash
    Preconditions: 空目录
    Steps:
      1. 运行 ls -la src/ 检查前端目录
      2. 运行 ls -la src-tauri/ 检查后端目录
      3. 运行 cat src-tauri/Cargo.toml | grep "tauri"
    Expected Result: 目录存在，Cargo.toml 包含 tauri 依赖
    Evidence: .sisyphus/evidence/task-1-init-structure.txt

  Scenario: 应用可启动
    Tool: Bash
    Preconditions: 依赖已安装
    Steps:
      1. 运行 timeout 30s pnpm tauri dev 2>&1 | head -50
    Expected Result: 输出包含 "Running" 或 "Compiling" 无错误
    Evidence: .sisyphus/evidence/task-1-dev-startup.txt
  ```

  **Commit**: YES
  - Message: `chore: initialize Tauri v2 + React + Vite + TypeScript project`
  - Files: `package.json`, `src/`, `src-tauri/`, `vite.config.ts`, `tsconfig.json`

- [x] 2. Tailwind CSS + shadcn/ui 配置

  **What to do**:
  - 安装 Tailwind CSS: `pnpm add -D tailwindcss postcss autoprefixer`
  - 初始化 Tailwind: `pnpm dlx tailwindcss init -p`
  - 安装 shadcn/ui: `pnpm dlx shadcn@latest init`
    - 选择 TypeScript: Yes
    - 选择 CSS variables: Yes
    - 选择 Base color: Slate
    - 选择 Style: Default
  - 配置 `tailwind.config.js` 包含 `./src/**/*.{ts,tsx}`
  - 在 `src/index.css` 添加 Tailwind 指令
  - 添加基础 shadcn/ui 组件: `pnpm dlx shadcn@latest add button input select tabs slider switch dialog dropdown-menu`

  **Must NOT do**:
  - 不要添加不需要的组件
  - 不要修改默认 shadcn/ui 主题（后续任务处理）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 标准配置任务
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (与 Task 3, 4 并行)
  - **Blocks**: Tasks 14, 15, 16, 17, 18, 19, 20
  - **Blocked By**: Task 1

  **References**:
  - Official shadcn/ui docs: https://ui.shadcn.com/docs/installation/vite
  - Tailwind CSS Vite guide: https://tailwindcss.com/docs/guides/vite

  **Acceptance Criteria**:
  - [ ] `tailwind.config.js` 存在
  - [ ] `components.json` 存在 (shadcn/ui 配置)
  - [ ] `src/components/ui/button.tsx` 存在
  - [ ] `src/index.css` 包含 `@tailwind` 指令

  **QA Scenarios**:
  ```
  Scenario: Tailwind 配置正确
    Tool: Bash
    Preconditions: Task 2 完成
    Steps:
      1. 运行 cat tailwind.config.js | grep "content"
      2. 运行 cat src/index.css | grep "@tailwind"
    Expected Result: content 包含 src 路径，index.css 包含三个 @tailwind 指令
    Evidence: .sisyphus/evidence/task-2-tailwind-config.txt

  Scenario: shadcn/ui 组件已安装
    Tool: Bash
    Preconditions: Task 2 完成
    Steps:
      1. 运行 ls src/components/ui/
    Expected Result: 输出包含 button.tsx, input.tsx, select.tsx, tabs.tsx, slider.tsx
    Evidence: .sisyphus/evidence/task-2-shadcn-components.txt
  ```

  **Commit**: YES
  - Message: `chore: add Tailwind CSS and shadcn/ui setup`
  - Files: `tailwind.config.js`, `postcss.config.js`, `components.json`, `src/index.css`, `src/components/ui/`

- [x] 3. 无边框窗口配置

- [x] 4. 设置存储模块 (Rust)

  **What to do**:
  - 创建 `src-tauri/src/settings.rs` 模块
  - 定义 `Settings` 结构体:
    ```rust
    pub struct Settings {
        pub source_lang: String,       // "auto", "zh", "en", etc.
        pub target_lang: String,       // "zh", "en", etc.
        pub engine: String,            // "baidu", "google", "siliconflow", "ollama"
        pub clipboard_enabled: bool,
        pub theme_color: String,       // "#426666"
        pub bg_color: String,          // "#3f3f3f"
        pub text_color: String,        // "#ffffff"
        pub transparency: u8,          // 0-100
        pub locale: String,            // "zh-CN", "en-US"
        // Engine configs
        pub baidu_app_id: String,
        pub baidu_secret_key: String,
        pub google_url: String,
        pub google_api_key: String,
        pub siliconflow_api_key: String,
        pub siliconflow_model: String,
        pub ollama_url: String,
        pub ollama_model: String,
    }
    ```
  - 实现 `Settings::default()` 返回默认值
  - 实现 `load_settings()` 从 Tauri appConfigDir 读取 JSON
  - 实现 `save_settings()` 写入 JSON 到 appConfigDir
  - 在 `src-tauri/src/main.rs` 中注册 Tauri commands: `get_settings`, `set_settings`
  - 处理文件不存在/损坏情况（返回默认值）

  **Must NOT do**:
  - 不要实现加密存储（API key 明文存储）
  - 不要添加历史记录相关字段

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Rust 模块实现，需要文件 IO 和序列化
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (与 Task 2, 3 并行)
  - **Blocks**: Tasks 5, 6, 8-11, 17, 18, 19, 21
  - **Blocked By**: Task 1

  **References**:
  - Tauri State Management: https://v2.tauri.app/develop/state-management/
  - Tauri Commands: https://v2.tauri.app/develop/calling-rust/
  - serde_json crate for JSON serialization

  **Acceptance Criteria**:
  - [ ] `src-tauri/src/settings.rs` 存在
  - [ ] `Settings` 结构体包含所有必需字段
  - [ ] `get_settings` 和 `set_settings` 命令可从前端调用
  - [ ] 设置文件保存到 Tauri appConfigDir

  **QA Scenarios**:
  ```
  Scenario: 设置存储和加载
    Tool: Bash
    Preconditions: Task 4 完成，应用可运行
    Steps:
      1. 运行 cat src-tauri/src/settings.rs | grep "pub struct Settings"
      2. 运行 cargo check --manifest-path src-tauri/Cargo.toml
    Expected Result: Settings 结构体存在，Rust 编译无错误
    Evidence: .sisyphus/evidence/task-4-settings-struct.txt

  Scenario: Tauri 命令注册
    Tool: Bash
    Preconditions: Task 4 完成
    Steps:
      1. 运行 grep -r "get_settings\|set_settings" src-tauri/src/
    Expected Result: 两个命令都在 main.rs 中注册
    Evidence: .sisyphus/evidence/task-4-commands.txt
  ```

  **Commit**: YES
  - Message: `feat: add settings store with JSON persistence`
  - Files: `src-tauri/src/settings.rs`, `src-tauri/src/main.rs`, `src-tauri/Cargo.toml`

- [x] 5. CSS 变量主题系统

  **What to do**:
  - 在 `src/index.css` 中定义 CSS 变量:
    ```css
    :root {
      --theme-color: #426666;
      --bg-color: #3f3f3f;
      --text-color: #ffffff;
      --settings-bg: #212121;
      --transparency: 1;
      --titlebar-opacity: 0.5;
    }
    ```
  - 创建 `src/lib/theme.ts`:
    - `applyTheme(settings: Settings)` 函数，将设置值应用到 CSS 变量
    - 透明度映射：settings.transparency (0-100) → titlebar-opacity (0-0.5)
  - 使用 React Context 管理主题状态: `src/contexts/ThemeContext.tsx`
  - 在根组件中订阅设置变化并应用主题

  **Must NOT do**:
  - 不要添加多主题预设
  - 不要添加字体相关变量

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: CSS 变量和简单 Context
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 13, 14, 15, 16, 17
  - **Blocked By**: Task 4

  **References**:
  - CSS Custom Properties: https://developer.mozilla.org/en-US/docs/Web/CSS/--*
  - shadcn/ui theming: https://ui.shadcn.com/docs/theming

  **Acceptance Criteria**:
  - [ ] CSS 变量在 `:root` 中定义
  - [ ] `applyTheme()` 函数正确映射透明度
  - [ ] ThemeContext 导出 `useTheme` hook

  **QA Scenarios**:
  ```
  Scenario: CSS 变量定义
    Tool: Bash
    Preconditions: Task 5 完成
    Steps:
      1. 运行 cat src/index.css | grep -A10 ":root"
    Expected Result: 包含 --theme-color, --bg-color, --text-color 等变量
    Evidence: .sisyphus/evidence/task-5-css-vars.txt

  Scenario: 透明度映射
    Tool: Bash
    Preconditions: Task 5 完成
    Steps:
      1. 运行 cat src/lib/theme.ts | grep -A5 "transparency"
    Expected Result: 代码显示 0-100 → 0-0.5 的映射逻辑
    Evidence: .sisyphus/evidence/task-5-transparency-map.txt
  ```

  **Commit**: YES
  - Message: `feat: add theme system with CSS variables`
  - Files: `src/index.css`, `src/lib/theme.ts`, `src/contexts/ThemeContext.tsx`

- [x] 6. 翻译引擎接口定义

  **What to do**:
  - 创建 `src-tauri/src/engines/mod.rs`:
    ```rust
    pub trait TranslationEngine {
        fn name(&self) -> &'static str;
        async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, EngineError>;
    }
    
    pub enum EngineError {
        Network(String),
        Auth(String),
        RateLimit,
        Timeout,
        Parse(String),
    }
    ```
  - 创建 `src/types/engine.ts` 前端类型定义:
    ```typescript
    export interface TranslationResult {
      text: string;
      engine: string;
      error?: string;
    }
    ```
  - 在 `src-tauri/src/main.rs` 中注册 `translate` 命令（占位实现）

  **Must NOT do**:
  - 不要实现具体引擎（后续任务）
  - 不要添加多于四个引擎的枚举值

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 接口定义，无复杂逻辑
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 7, 8, 9, 10, 20
  - **Blocked By**: Task 1

  **References**:
  - Rust Async Traits: https://rust-lang.github.io/async-book/
  - Tauri IPC Types

  **Acceptance Criteria**:
  - [ ] `TranslationEngine` trait 定义存在
  - [ ] `EngineError` 枚举包含 Network, Auth, RateLimit, Timeout, Parse
  - [ ] 前端 `TranslationResult` 类型存在

  **QA Scenarios**:
  ```
  Scenario: Rust trait 定义
    Tool: Bash
    Preconditions: Task 6 完成
    Steps:
      1. 运行 cat src-tauri/src/engines/mod.rs | grep -A5 "trait TranslationEngine"
    Expected Result: trait 包含 name() 和 translate() 方法
    Evidence: .sisyphus/evidence/task-6-trait-def.txt
  ```

  **Commit**: YES
  - Message: `feat: add translation engine trait/interface`
  - Files: `src-tauri/src/engines/mod.rs`, `src/types/engine.ts`

- [x] 7. 百度翻译引擎实现

- [x] 8. 谷歌翻译引擎实现

- [x] 9. 硅基流动引擎实现

- [x] 10. Ollama 引擎实现

  **What to do**:
  - 创建 `src-tauri/src/engines/ollama.rs`
  - 实现 `OllamaEngine` 结构体:
    - 字段: `url` (默认 "http://localhost:11434"), `model` (默认 "llama2")
  - API 端点: `{url}/api/chat`
  - 请求格式:
    ```json
    {
      "model": "llama2",
      "messages": [
        {"role": "system", "content": "Translate to {target_lang}. Keep LaTeX formulas unchanged."},
        {"role": "user", "content": "{text}"}
      ],
      "stream": false
    }
    ```
  - 无认证（本地服务）
  - 处理连接失败错误（Ollama 未运行）

  **Must NOT do**:
  - 不要实现流式响应
  - 不要添加模型下载功能

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: HTTP 请求、本地服务连接
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (与 Tasks 7, 8, 9 并行)
  - **Blocks**: Task 20
  - **Blocked By**: Task 6

  **References**:
  - Ollama API docs: https://github.com/ollama/ollama/blob/main/docs/api.md
  - Ollama Chat endpoint: https://docs.ollama.com/api/chat

  **Acceptance Criteria**:
  - [ ] `OllamaEngine` 实现 `TranslationEngine` trait
  - [ ] `stream: false` 在请求中设置
  - [ ] 连接失败返回 `EngineError::Network`

  **QA Scenarios**:
  ```
  Scenario: Ollama 引擎配置
    Tool: Bash
    Preconditions: Task 10 完成
    Steps:
      1. 运行 cat src-tauri/src/engines/ollama.rs | grep -A10 "struct OllamaEngine"
      2. 运行 grep "stream.*false\|localhost:11434" src-tauri/src/engines/ollama.rs
    Expected Result: 结构体包含 url 和 model 字段，请求禁用流式
    Evidence: .sisyphus/evidence/task-10-ollama-impl.txt
  ```

  **Commit**: YES
  - Message: `feat: implement Ollama translation engine`
  - Files: `src-tauri/src/engines/ollama.rs`

- [x] 11. 剪切板监听模块

  **What to do**:
  - 添加 `tauri-plugin-clipboard-x` 到 `src-tauri/Cargo.toml`
  - 在 `tauri.conf.json` 中启用插件
  - 创建 `src-tauri/src/clipboard.rs`:
    - 启动剪切板监听
    - 发送 `clipboard-changed` 事件到前端
    - 实现 300ms 防抖，避免频繁触发
    - 实现重复检测，跳过与上次相同的内容
  - 在前端创建 `src/hooks/useClipboard.ts`:
    - 订阅 `clipboard-changed` 事件
    - 返回最新剪切板文本
  - 仅监听文本类型，忽略图片/文件

  **Must NOT do**:
  - 不要实现剪切板历史存储
  - 不要修改系统剪切板内容

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 事件驱动、Rust-JS 通信、防抖逻辑
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 20
  - **Blocked By**: Task 1

  **References**:
  - tauri-plugin-clipboard-x: https://github.com/ayangweb/tauri-plugin-clipboard-x
  - Tauri Events: https://v2.tauri.app/develop/calling-frontend/

  **Acceptance Criteria**:
  - [ ] 插件正确安装和配置
  - [ ] `clipboard-changed` 事件可被前端接收
  - [ ] 300ms 防抖实现
  - [ ] 重复内容检测实现

  **QA Scenarios**:
  ```
  Scenario: 剪切板模块存在
    Tool: Bash
    Preconditions: Task 11 完成
    Steps:
      1. 运行 cat src-tauri/src/clipboard.rs | head -30
      2. 运行 grep "clipboard" src-tauri/Cargo.toml
    Expected Result: clipboard.rs 存在，Cargo.toml 包含 clipboard 插件依赖
    Evidence: .sisyphus/evidence/task-11-clipboard-module.txt

  Scenario: 防抖逻辑
    Tool: Bash
    Preconditions: Task 11 完成
    Steps:
      1. 运行 grep -i "debounce\|300" src-tauri/src/clipboard.rs
    Expected Result: 代码包含 300ms 防抖相关逻辑
    Evidence: .sisyphus/evidence/task-11-debounce.txt
  ```

  **Commit**: YES
  - Message: `feat: add clipboard monitoring module`
  - Files: `src-tauri/src/clipboard.rs`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/hooks/useClipboard.ts`

- [x] 12. LaTeX 检测与提取

- [x] 13. LaTeX 渲染组件

  **What to do**:
  - 安装 KaTeX: `pnpm add katex react-katex`
  - 安装类型: `pnpm add -D @types/katex`
  - 在 `src/index.css` 中导入 KaTeX 样式: `@import 'katex/dist/katex.min.css';`
  - 创建 `src/components/LatexRenderer.tsx`:
    - Props: `text: string` (包含 LaTeX 的文本)
    - 使用 `detectLatex` 分离普通文本和 LaTeX
    - 普通文本直接渲染
    - LaTeX 使用 `<InlineMath>` 或 `<BlockMath>` 渲染
    - 错误处理: LaTeX 解析失败时显示原始文本
  - 使用 CSS 变量 `--text-color` 作为文本颜色

  **Must NOT do**:
  - 不要添加公式编辑功能
  - 不要添加复制公式为图片功能

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: React 组件、样式集成
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 15, 20
  - **Blocked By**: Tasks 2, 5, 12

  **References**:
  - react-katex: https://github.com/talyssonoc/react-katex
  - KaTeX options: https://katex.org/docs/options.html

  **Acceptance Criteria**:
  - [ ] KaTeX 样式正确加载
  - [ ] 内联公式使用 `<InlineMath>`
  - [ ] 块级公式使用 `<BlockMath>`
  - [ ] 解析错误时回退到原始文本

  **QA Scenarios**:
  ```
  Scenario: KaTeX 依赖安装
    Tool: Bash
    Preconditions: Task 13 完成
    Steps:
      1. 运行 grep "katex\|react-katex" package.json
    Expected Result: package.json 包含 katex 和 react-katex 依赖
    Evidence: .sisyphus/evidence/task-13-katex-deps.txt

  Scenario: 渲染组件结构
    Tool: Bash
    Preconditions: Task 13 完成
    Steps:
      1. 运行 cat src/components/LatexRenderer.tsx | grep -A5 "InlineMath\|BlockMath"
    Expected Result: 组件使用 InlineMath 和 BlockMath
    Evidence: .sisyphus/evidence/task-13-latex-component.txt
  ```

  **Commit**: YES
  - Message: `feat: add LaTeX rendering component with react-katex`
  - Files: `src/components/LatexRenderer.tsx`, `src/index.css`, `package.json`

- [x] 14. 自定义标题栏组件

- [x] 15. 翻译主视图组件

- [x] 16. 设置弹窗 - 通用标签页

- [x] 17. 设置弹窗 - 外观标签页

- [x] 18. 设置弹窗 - 引擎标签页

- [x] 19. 引擎选择下拉框

- [x] 20. 剪切板→翻译→渲染 Pipeline

- [x] 21. 复制译文功能

- [x] 22. 最小化/关闭按钮行为

  **What to do**:
  - 在 Titlebar 的最小化/关闭按钮上实现:
    - 左键点击 → 最小化到任务栏
    - 右键点击 → 关闭应用
  - 使用 Tauri window API:
    - `getCurrentWindow().minimize()` (左键)
    - `getCurrentWindow().close()` (右键)
  - 添加 `onContextMenu` 事件处理右键
  - 按钮 tooltip 说明操作方式

  **Must NOT do**:
  - 不要添加最小化到托盘功能
  - 不要添加关闭确认对话框

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 简单事件处理
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 5 (与 Task 21 并行)
  - **Blocks**: None
  - **Blocked By**: Tasks 3, 14

  **References**:
  - Tauri Window API: https://v2.tauri.app/reference/javascript/window/
  - `src/components/Titlebar.tsx` — 按钮位置

  **Acceptance Criteria**:
  - [ ] 左键最小化到任务栏
  - [ ] 右键关闭应用
  - [ ] 按钮有 tooltip 说明

  **QA Scenarios**:
  ```
  Scenario: 按钮事件绑定
    Tool: Bash
    Preconditions: Task 22 完成
    Steps:
      1. 运行 grep "onClick\|onContextMenu\|minimize\|close" src/components/Titlebar.tsx
    Expected Result: 按钮绑定 onClick (minimize) 和 onContextMenu (close) 事件
    Evidence: .sisyphus/evidence/task-22-button-events.txt

  Scenario: Tooltip 存在
    Tool: Bash
    Preconditions: Task 22 完成
    Steps:
      1. 运行 grep -i "title=\|tooltip" src/components/Titlebar.tsx
    Expected Result: 按钮包含 title 属性或 Tooltip 组件
    Evidence: .sisyphus/evidence/task-22-tooltip.txt
  ```

  **Commit**: YES
  - Message: `feat: wire up minimize/close button behaviors`
  - Files: `src/components/Titlebar.tsx`

---

## Final Verification Wave

- [x] F1. **Plan Compliance Audit** — `oracle`

- [x] F2. **Code Quality Review** — `unspecified-high`

- [x] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill)

- [x] F4. **Scope Fidelity Check** — `deep`
  对每个任务：读取 "What to do"，读取实际 diff (git log/diff)。验证 1:1 对应——规范中的所有内容都已构建（无遗漏），规范之外的内容未构建（无蔓延）。检查 "Must NOT do" 合规性。检测跨任务污染：Task N 触及 Task M 的文件。标记未说明的更改。
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Task(s) | Commit Message |
|---------|----------------|
| 1 | `chore: initialize Tauri v2 + React + Vite + TypeScript project` |
| 2 | `chore: add Tailwind CSS and shadcn/ui setup` |
| 3 | `chore: configure frameless window in tauri.conf.json` |
| 4 | `feat: add settings store with JSON persistence` |
| 5 | `feat: add theme system with CSS variables` |
| 6 | `feat: add translation engine trait/interface` |
| 7 | `feat: implement Baidu translation engine` |
| 8 | `feat: implement Google translation engine` |
| 9 | `feat: implement SiliconFlow translation engine` |
| 10 | `feat: implement Ollama translation engine` |
| 11 | `feat: add clipboard monitoring module` |
| 12 | `feat: add LaTeX detection and extraction utility` |
| 13 | `feat: add LaTeX rendering component with react-katex` |
| 14 | `feat: add custom titlebar with drag region` |
| 15 | `feat: add main translation view component` |
| 16 | `feat: add settings modal General tab` |
| 17 | `feat: add settings modal Appearance tab` |
| 18 | `feat: add settings modal Engines tab` |
| 19 | `feat: add engine selector dropdown` |
| 20 | `feat: integrate clipboard-translate-render pipeline` |
| 21 | `feat: add copy translated text functionality` |
| 22 | `feat: wire up minimize/close button behaviors` |

---

## Success Criteria

### Verification Commands
```bash
# 项目构建
pnpm tauri dev  # Expected: 应用窗口打开，无错误
pnpm tauri build  # Expected: 生成 .exe 文件

# TypeScript 类型检查
pnpm tsc --noEmit  # Expected: 无错误

# Rust 编译检查
cd src-tauri && cargo check  # Expected: 无错误
```

### Final Checklist
- [ ] 复制文本后自动翻译（< 2秒）
- [ ] LaTeX 公式正确渲染（KaTeX 类名可见）
- [ ] 四个翻译引擎可配置和切换
- [ ] 设置持久化（重启后保留）
- [ ] 自定义标题栏可拖动
- [ ] 左键最小化、右键关闭按钮正常工作
- [ ] 透明度滑块实时生效
- [ ] 所有 UI 标签为中文
- [ ] 无 "Must NOT Have" 功能被实现
