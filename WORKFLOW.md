# 翻译应用自动化工作流指南

## 概览

本项目使用 **自动代码审查和重做循环** 实现高质量代码交付。

## 架构

```
你 (Founder) 说:"实现 XXX 功能"
    ↓
Sisyphus (主协调者)
    ↓
并行启动 background agents:
    ├─ Hephaestus (background) - 实现代码
    ├─ Momus (background) - 准备审查
    └─ Librarian (background) - 研究最佳实践
    ↓
Hephaestus 完成 → 自动触发审查
    ↓
如果不通过:
    ├─ 记录问题到审查报告
    ├─ 重新启动 Hephaestus 修复
    └─ 循环直到通过
    ↓
如果通过:
    ├─ 更新 CHANGELOG.md
    ├─ commit (如果要求)
    └─ 完成
```

## Git 分支策略

- **main** - 稳定版本，已经测试和审查通过
- **dev** - 开发分支，日常开发
- **feature-*** - 功能分支

## 版本管理

版本号格式：`0.0.x`
- x 从 0 开始
- 每次功能更新递增 1
- 当前版本：`0.1.0`

每次更新需要:
1. 修改代码
2. 通过审查
3. 更新 CHANGELOG.md
4. 版本号 +1

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
# 更新版本号
```

### 2. 自动化工作流（推荐）

在 OpenCode 中对我（Sisyphus）说:

```
实现用户认证功能
```

我会自动执行:
1. 并行启动 Hephaestus、Momus、Librarian
2. Hephaestus 实现功能
3. Momus 审查代码
4. 如果不通过 → 自动重启 Hephaestus 修复
5. 循环直到通过
6. 更新 CHANGELOG
7. 询问是否 commit

## 审查标准

Momus 审查检查项:
1. **正确性**：逻辑、边缘情况、错误处理
2. **质量**：可读性、可维护性、测试覆盖率
3. **架构**：SOLID 原则、设计模式、关注点分离
4. **安全性**：漏洞、认证、数据验证
5. **类型安全**：无 `as any`、`@ts-ignore`

## 常见命令

```bash
# 查看当前分支
git branch

# 查看版本
cat package.json | grep version
```

## 目录结构

```
translate_app/
├── .opencode/
│   └── opencode.json                      # Oh My OpenCode 配置
├── .sisyphus/                             # Oh My OpenCode 工作记录
├── src/                                   # 源代码
├── CHANGELOG.md                           # 版本更新日志
└── package.json                           # 项目配置（包含版本号）
```

## 配置文件

### .opencode/opencode.json
```json
{
  "background_task": {
    "defaultConcurrency": 5,
    "providerConcurrency": {
      "anthropic": 3,
      "openai": 3
    }
  }
}
```

## 故障排除

### 分支冲突
```bash
git checkout main
git pull
git checkout dev
git merge main
# 解决冲突...
git commit
```

## 联系方式

如有问题，请在 OpenCode 中直接问我（Sisyphus）。

