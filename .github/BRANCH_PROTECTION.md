# GitHub 分支保护配置指南

本文档指导如何在 GitHub 上配置分支保护规则，确保代码质量。

## 配置步骤

### 1. 进入仓库设置

1. 打开 https://github.com/hanxueyuan/searchEverything
2. 点击 **Settings** 标签
3. 在左侧菜单选择 **Branches**
4. 点击 **Add branch protection rule**

### 2. 配置 main 分支保护

**Branch name pattern:** `main`

**保护规则:**

#### 必需的检查 (Required status checks)
- ✅ **Require status checks to pass before merging**
- ✅ **Require branches to be up to date before merging**

选择以下必需的状态检查：
- [x] Lint (Format & Clippy)
- [x] Test (Linux)
- [x] Test (Windows)
- [x] Test (macOS)
- [x] Build (ubuntu-latest)
- [x] Build (windows-latest)
- [x] Build (macos-latest)
- [x] Quality Gate
- [x] Code Coverage

#### Pull Request 要求 (Pull Request reviews)
- ✅ **Require a pull request before merging**
- ✅ **Require approvals**
  - Required number of approvals before merging: `1`
- ✅ **Dismiss stale pull request approvals when new commits are pushed**
- ✅ **Require review from Code Owners** (如果配置了 CODEOWNERS)

#### 合并选项 (Merge options)
- ✅ **Require conversation resolution before merging**
- ✅ **Require linear history** (可选，保持提交历史清晰)

#### 其他限制 (Additional settings)
- ❌ **Do not allow bypassing the above settings** (谨慎启用，可能影响管理员)
- ✅ **Restrict who can push to matching branches** (可选，限制推送权限)
- ✅ **Allow force pushes** (通常禁用，除非有特殊需求)
- ✅ **Allow deletions** (通常禁用)

### 3. 配置 develop 分支保护 (可选)

**Branch name pattern:** `develop`

**保护规则:**
- ✅ **Require status checks to pass before merging**
  - 选择相同的 CI 检查
- ✅ **Require branches to be up to date before merging**

### 4. 配置 CODEOWNERS (可选但推荐)

创建 `.github/CODEOWNERS` 文件：

```
# 默认代码所有者
* @hanxueyuan

# 特定目录的所有者
/src/ @hanxueyuan
/.github/ @hanxueyuan
```

## 验证配置

### 测试分支保护

1. 创建一个新分支并做修改
2. 尝试直接推送到 main 分支 → 应该被拒绝
3. 创建 Pull Request
4. 等待 CI 检查完成
5. 尝试在没有批准的情况下合并 → 应该被阻止
6. 获得批准后合并 → 应该成功

### 检查状态

在 Pull Request 页面，你应该看到：
- ✅ 所有必需的检查通过
- ✅ 所需的批准数量满足
- ✅ 合并按钮可用 (绿色)

## 故障排除

### CI 检查未出现

1. 检查 GitHub Actions 是否启用
2. 检查工作流文件是否正确
3. 查看 Actions 标签页是否有运行记录

### 检查未标记为必需

1. 等待 CI 运行完成
2. 刷新分支保护设置页面
3. 新完成的检查会出现在可选列表中

### 管理员被阻止

如果需要管理员绕过权限：
1. 在分支保护设置中启用 **Allow administrators to bypass**
2. 或临时禁用保护规则

## 最佳实践

### 1. 保持 CI 快速
- 优化测试速度
- 使用缓存
- 并行运行独立任务

### 2. 清晰的提交信息
- 使用约定式提交 (Conventional Commits)
- 关联 Issue 和 PR

### 3. 定期审查
- 每月审查一次保护规则
- 根据团队规模调整批准要求
- 更新 CI 检查列表

### 4. 文档化
- 在 README 中说明提交流程
- 维护贡献指南
- 记录常见问题

## 相关资源

- [GitHub 分支保护文档](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/about-protected-branches)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [CODEOWNERS 文档](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners)
