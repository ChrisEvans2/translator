# CHANGELOG

所有重要的更改都会记录在这个文件中。

版本号格式：`0.0.x`
- 每次功能更新递增 x
- main 分支包含稳定版本
- dev 分支用于开发

---

## [0.0.0] - 2026-03-29

### 添加
- 初始化 Tauri + React 翻译应用项目
- 实现基础翻译功能（百度、Google、Ollama、SiliconFlow）
- 添加 shadcn/ui 组件库
- 实现剪贴板监听功能
- 添加主题切换（亮色/暗色）
- 实现设置界面（引擎配置、快捷键、主题等）
- 添加 LaTeX 数学公式渲染支持

### 技术栈
- **前端**: React 19 + TypeScript + Tailwind CSS
- **后端**: Rust (Tauri 2)
- **UI 组件**: shadcn/ui (Radix UI)
- **构建工具**: Vite

### 开发工具
- Git 分支管理：main（稳定）、dev（开发）
- OpenCode 插件：
  - `@tmegit/opencode-worktree-session` - Git worktree 自动化管理
  - Oh My OpenCode - 后台任务并发管理
