# 翻译应用自动化工作流指南

## 概览

本项目使用 **方案 B（Worktree + 审查循环）**，实现全自动代码审查和重做循环。

## 架构

```
你 (Founder) 说："实现 XXX 功能"
    ↓
Sisyphus (主协调者)
    ↓
并行启动 background agents：
    ├─ Hephaestus (background) - 实现代码
    ├─ Momus (background) - 准备审查
    └─ Librarian (background) - 研究最佳实践
    ↓
Hephaestus 完成 → 自动触发审查
    ↓
如果不通过：
    ├─ 记录问题到审查报告
    ├─ 重新启动 Hephaestus 修复
    └─ 循环直到通过
    ↓
如果通过：
    ├─ 更新 CHANGELOG.md
    ├─ commit (如果要求)
    └─ 完成
```

## Git 分支策略

- **main** - 稳定版本，已经测试和审查通过
- **dev** - 开发分支，日常开发
- **opencode/feature-*** - Worktree 自动创建的功能分支

## 版本管理

版本号格式：`0.0.x`
- x 从 0 开始
- 每次功能更新递增 1
- 当前版本：`0.0.0`

每次更新需要：
1. 修改代码
2. 通过审查
3. 更新 CHANGELOG.md
4. 版本号 +1

## Worktree 位置

⚠️ **重要说明**：插件硬编码路径，worktree 存储在：
- `D:\VScode\Work_code\translate_app\.opencode\worktrees\<分支名>/`

**不是** `D:\opencode_project`（无法自定义基础路径）

## 工作流使用

### 1. 日常开发（手动）

```bash
# 在 dev 分支工作
git checkout dev
# 改代码...
git add .
git commit -m "feat: 新功能描述"
# 审查通过后
git checkout main
git merge dev
# 更新版本号到 0.0.1
```

### 2. 自动化工作流（推荐）

在 OpenCode 中对我（Sisyphus）说：

```
实现用户认证功能
```

我会自动执行：
1. 并行启动 Hephaestus、Momus、Librarian
2. Hephaestus 在 worktree 实现功能
3. Momus 审查代码
4. 如果不通过 → 自动重启 Hephaestus 修复
5. 循环直到通过
6. 更新 CHANGELOG
7. 询问是否 commit

### 3. 使用 Worktree 插件

插件自动管理 worktree 生命周期：

**创建 Worktree**：
```bash
# 会提示输入分支名
opencode
# 输入：feature-auth
# 自动创建 opencode/feature-auth 分支
# 自动创建 .opencode/worktrees/feature-auth/ 目录
# 自动在新终端打开 OpenCode
```

**删除 Worktree**（session 结束时自动）：
```bash
# 退出 OpenCode
exit
# 插件自动：
# 1. git add .
# 2. AI 生成 commit message
# 3. git commit
# 4. git push origin opencode/feature-auth
# 5. 删除 worktree 目录
```

## 审查标准

Momus 审查检查项：
1. **正确性**：逻辑、边缘情况、错误处理
2. **质量**：可读性、可维护性、测试覆盖率
3. **架构**：SOLID 原则、设计模式、关注点分离
4. **安全性**：漏洞、认证、数据验证
5. **类型安全**：无 `as any`、`@ts-ignore`

## 常见命令

```bash
# 查看当前分支
git branch

# 查看所有 worktree
git worktree list

# 手动清理 worktree
git worktree remove .opencode/worktrees/<分支名>
git worktree prune

# 查看版本
cat package.json | grep version
```

## 目录结构

```
translate_app/
├── .opencode/
│   ├── opencode.json                      # Oh My OpenCode 配置
│   ├── opencode-worktree-session-config.json  # Worktree 插件配置
│   └── worktrees/                         # Worktree 存储目录
│       ├── feature-auth/                  # 功能分支 worktree
│       └── feature-payment/               # 另一个功能分支
├── .sisyphus/                             # Oh My OpenCode 工作记录
├── src/                                   # 源代码
├── CHANGELOG.md                           # 版本更新日志
└── package.json                           # 项目配置（包含版本号）
```

## 配置文件

### .opencode/opencode.json
```json
{
  "plugin": ["@tmegit/opencode-worktree-session"],
  "background_task": {
    "defaultConcurrency": 5,
    "providerConcurrency": {
      "anthropic": 3,
      "openai": 3
    }
  }
}
```

### .opencode/opencode-worktree-session-config.json
```json
{
  "terminal": {
    "mode": "default"
  },
  "postWorktree": {
    "cmd": "",
    "args": ""
  },
  "configToolsAvailable": false
}
```

## 故障排除

### Worktree 没有被删除
```bash
git worktree list
git worktree remove .opencode/worktrees/<分支名> --force
git worktree prune
```

### 分支冲突
```bash
git checkout main
git pull
git checkout dev
git merge main
# 解决冲突...
git commit
```

### 插件不工作
```bash
# 重新安装
pnpm remove @tmegit/opencode-worktree-session
pnpm add -D @tmegit/opencode-worktree-session
# 重启 OpenCode
```

## 联系方式

如有问题，请在 OpenCode 中直接问我（Sisyphus）。
