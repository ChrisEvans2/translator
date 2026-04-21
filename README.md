# Translator

一款基于 Tauri v2 + React 19 的 Windows 桌面翻译应用，监听剪贴板内容自动翻译，支持 LaTeX 公式渲染。

## 功能特性

- **剪贴板自动翻译** — 复制任意文本后，翻译结果实时显示在悬浮窗口中，无需手动操作
- **多翻译引擎** — 支持百度翻译、Google 翻译、大模型 API（SiliconFlow/OpenAI 兼容）、本地 Ollama
- **LaTeX 公式渲染** — 自动识别并渲染 `$...$`（行内）和 `$$...$$`（块级）公式，适合学术场景
- **主题自定义** — 可调整窗口背景色、主题色、文字颜色、透明度
- **多语言互译** — 支持中文、英文、日文、韩文及自动检测源语言
- **窗口置顶** — 翻译窗口始终显示在最前，不影响其他操作

## 截图

> 复制文本 → 翻译结果自动出现在悬浮窗口

## 安装使用

### 直接使用（推荐）

从 [Releases](https://github.com/ChrisEvans2/translator/releases) 下载最新安装包，运行即可。

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

### 大模型 API（默认 SiliconFlow）
1. 前往 [SiliconFlow](https://siliconflow.cn/) 注册并获取 API Key
2. 填入 API Key，模型默认为 `deepseek-ai/DeepSeek-V3`
3. 也可填入任意 OpenAI 兼容接口的 API Key + 模型名

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

## 技术栈

- **前端**: React 19 + TypeScript + Tailwind CSS v4 + Radix UI
- **后端**: Rust + Tauri v2
- **LaTeX 渲染**: react-katex
- **构建工具**: Vite + pnpm

## License

MIT
