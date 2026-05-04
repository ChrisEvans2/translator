# Feature

当前版本更新记录（版本迭代后清空）

## 2026-05-02

### 设置界面弹窗（Tooltip）优化

**弹窗定位重构：**
- 所有引擎设置弹窗从 `position: absolute` 改为 `position: fixed`，避免被 `overflow-y-auto` 滚动容器截断
- 新增 `handleTooltipEnter`/`handleTooltipLeave` 辅助函数，通过 `getBoundingClientRect()` 计算弹窗位置
- 弹窗使用 `z-[9999]` 确保显示在最顶层

**自动向上弹出：**
- 当检测到下方空间不足时，弹窗自动向上弹出（`top = rect.top - 4 - tooltipHeight`）
- 当检测到右侧溢出时，弹窗自动左移

**长文本拆行显示：**
- 百度 APP ID/密钥、谷歌镜像源/官方 URL、大模型API 地址/模型等长文本弹窗拆成两行显示
- 移除 `whitespace-nowrap`，改用 `max-width` + `<br />` 控制换行

**仅举例输入框显示弹窗：**
- 需要举例的输入框（URL、模型等）保留弹窗提示
- 不需要举例的输入框（API Key、VLM 模型等）移除弹窗

**溢出修复：**
- `settings.html` 的 `html, body, #root` 添加 `overflow: visible`
- 设置窗口根容器从 `overflow-hidden` 改为 `overflow-visible`

### 大模型API URL 可配置

- 新增 `llmapi_url` 设置字段，不再硬编码 SiliconFlow 端点
- 默认值: `https://api.siliconflow.cn/v1/chat/completions`
- 支持任何 OpenAI 兼容 API 提供商（DeepSeek、Moonshot、智谱、Groq、Together、LM Studio、vLLM 等）
- 设置界面「引擎 → 大模型API」新增 URL 输入框

## 2026-04-21

### 设置界面 UI 优化

**通用设置下拉框焦点动画：**
- 为 `<select>` 元素新增 `.settings-select-wrapper` 包裹层
- 点击下拉框时，底部白色线条变为主题颜色从中间向两边展开的动画，与引擎设置输入框动画一致

**导航标签滑动指示器：**
- 移除通用/外观/引擎按钮的 `bg-white/15` 高亮背景
- 改为在按钮左右两侧显示主题颜色竖线指示当前选中项
- 切换标签时，竖线指示器平滑滑动到目标按钮（`transition: top 0.25s ease`）
