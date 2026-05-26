# Translator

一款基于 Tauri v2 + React 19 的 Windows 桌面翻译应用，监听剪贴板内容自动翻译，支持 LaTeX 公式渲染。

## 功能特性

### 核心翻译
- **剪贴板自动翻译** — 复制任意文本后，翻译结果实时显示在悬浮窗口中，无需手动操作
- **多翻译引擎** — 支持百度翻译、Google 翻译、大模型 API（OpenAI 兼容）、本地 Ollama
- **LaTeX 公式渲染** — 自动识别并渲染 `$...$`（行内）和 `$$...$$`（块级）公式，适合学术场景
- **图片翻译（VLM）** — 支持视觉语言模型图片翻译（仅 LLM 引擎）

### 划词翻译
- **划词自动翻译** — 选中文本后自动翻译，支持鼠标拖选触发
- **快捷键触发** — 默认 `Alt+Q`，可自定义
- **剪贴板保护** — 翻译前后自动保存/恢复剪贴板内容
- **悬浮弹窗** — 翻译结果在鼠标附近弹出，支持复制、关闭、ESC 快捷键

### 界面与设置
- **主题自定义** — 可调整窗口背景色、主题色、文字颜色、透明度
- **多语言互译** — 支持中文、英文、日文、韩文及自动检测源语言
- **窗口置顶** — 翻译窗口始终显示在最前（可关闭）
- **自动显示** — 翻译完成后自动弹出窗口（可关闭）
- **设置自动保存** — 修改即时生效，关键设置（引擎/语言）立即保存

## 安装使用

### 直接使用（推荐）

从 [Releases](https://github.com/ChrisEvans2/translator/releases) 下载最新安装包：

| 安装包 | 说明 |
|--------|------|
| `translate_app_x.x.x_x64-setup.exe` | NSIS 安装程序（推荐） |
| `translate_app_x.x.x_x64_en-US.msi` | MSI 安装程序 |

运行安装程序，按照提示完成安装即可。

### 从源码构建

**环境要求：**
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) 1.70+
- [Tauri 前置依赖](https://tauri.app/start/prerequisites/)（Windows 需要 WebView2）

```bash
git clone https://github.com/ChrisEvans2/translator.git
cd translator
pnpm install
pnpm tauri build
```

构建产物在 `src-tauri/target/release/bundle/` 目录下。

## 配置翻译引擎

首次启动后点击右上角设置图标，按需配置：

### 百度翻译
1. 前往 [百度翻译开放平台](https://fanyi-api.baidu.com/) 注册开发者账号
2. 创建应用，获取 **APP ID** 和**密钥**
3. 填入设置中的对应字段

### Google 翻译
- **镜像源**（推荐）：填入可用的 Google 翻译镜像 URL
- **官方 API**：填入官方 URL + API Key（需要 Google Cloud 账号）

### 大模型 API
1. 支持任何 OpenAI 兼容接口（SiliconFlow、DeepSeek、Moonshot、智谱、Groq、Together、LM Studio、vLLM 等）
2. 填入 API 地址、API Key 和模型名
3. 默认使用 SiliconFlow 的 `deepseek-ai/DeepSeek-V3`

### Ollama（本地模型）
1. 安装并启动 [Ollama](https://ollama.com/)
2. 拉取模型：`ollama pull llama2`（或其他模型）
3. 填入服务地址（默认 `http://localhost:11434`）和模型名

## 使用方法

1. 启动应用，翻译窗口会出现在屏幕上
2. 在设置中配置好翻译引擎的凭证
3. 选择源语言和目标语言（支持自动检测）
4. 复制任意文本，翻译结果自动显示
5. 点击翻译结果可复制到剪贴板

### 划词翻译

1. 在设置中启用「划词翻译」
2. 选择模式：
   - **自动翻译**（默认）：选中文本后自动翻译
   - **快捷键触发**：按 `Alt+Q`（可自定义）翻译选中文本
3. 翻译结果在鼠标附近弹出，点击复制按钮可复制结果

## 版本历史

### v0.0.7（2026-05-25）
- 新增划词翻译功能：独立悬浮窗口、划词设置页、鼠标拖选触发
- 修复划词翻译弹窗显示问题

### v0.0.6（2026-05-02）
- 设置界面弹窗（Tooltip）重构：fixed 定位、自动向上弹出
- 大模型 API URL 可配置，支持任意 OpenAI 兼容接口

### v0.0.5（2026-05-01）
- 新增最顶层显示、自动显示开关
- 新增图片翻译（VLM）支持
- 设置文件路径迁移至 `~/translate_app.json`
- 修复翻译卡死、设置反序列化失败等问题

### v0.0.4（2026-04-15）
- LLM 提示词优化：语言代码映射为自然语言名称
- 重命名硅基流动引擎为大模型API
- 修复 LaTeX 公式显示问题

### v0.0.3（2026-03-31）
- Google 翻译双 URL 支持（镜像源 + 官方 API）
- 设置自动保存、跨窗口设置同步
- 错误提示优化

### v0.0.2（2026-03-30）
- 设置界面改进：可折叠手风琴、输入框样式优化
- 百度翻译增加 URL 输入框

### v0.0.1（2026-03-30）
- 跨窗口主题同步、窗口透明度修复

### v0.0.0（2026-03-29）
- 初始化项目，实现基础翻译功能

## 技术栈

- **前端**: React 19 + TypeScript + Tailwind CSS v4 + Radix UI
- **后端**: Rust + Tauri v2
- **LaTeX 渲染**: react-katex
- **构建工具**: Vite + pnpm

## License

MIT
