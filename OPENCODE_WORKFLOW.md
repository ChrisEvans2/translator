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
    ├─ 更新 Feature.md（记录本次小更新）
    └─ 等待用户指示是否 commit

你 (Founder) 说:"commit"
    ↓
Sisyphus 执行:
    ├─ 将 Feature.md 总结写入 CHANGELOG.md
    ├─ 清空 Feature.md
    ├─ 更新版本号 (0.0.x → 0.0.x+1)
    └─ git commit
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

**版本迭代流程**：
1. 日常开发：更新 Feature.md（记录小更新）
2. 用户说 "commit"：
   - 将 Feature.md 总结写入 CHANGELOG.md
   - 清空 Feature.md
   - 版本号 +1
   - git commit

**文件说明**：
- `Feature.md`：记录当前版本的小更新，版本迭代后清空
- `CHANGELOG.md`：记录所有版本的更新历史

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
6. 更新 Feature.md（记录本次更新）
7. 等待你指示是否 commit

当你决定 commit 时，我会：
1. 将 Feature.md 总结写入 CHANGELOG.md
2. 清空 Feature.md
3. 版本号 +1
4. git commit

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
│   ├── opencode.json                      # OpenCode 配置
│   └── skills/                            # Agent 技能文件
│       ├── hephaestus/SKILL.md            # 代码实现 agent
│       ├── momus/SKILL.md                 # 代码审查 agent
│       └── librarian/SKILL.md             # 研究 agent
├── .sisyphus/                             # 工作记录
├── src/                                   # 源代码
├── Feature.md                             # 当前版本小更新记录
├── CHANGELOG.md                           # 版本更新历史
└── package.json                           # 项目配置（包含版本号）
```

## 配置文件

### .opencode/opencode.json
```json
{
  "$schema": "https://opencode.ai/config.json"
}
```

当前配置为空，使用 OpenCode 默认设置。

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

