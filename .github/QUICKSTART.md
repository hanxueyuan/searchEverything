# CI/CD 快速开始指南

## 🚀 5 分钟快速配置

### 步骤 1: 推送配置到 GitHub

```bash
cd /workspace/projects/searchEverything

# 添加所有新文件
git add .github/
git add scripts/verify-cicd.sh
git add src/tests/integration_tests.rs
git add src/tests/mod.rs
git add CI_CD_SUMMARY.md

# 提交
git commit -m "feat: add complete CI/CD workflow

- GitHub Actions workflows (ci, coverage, release, pr-validation)
- Integration tests for core functionality
- Code coverage with 80% threshold
- Multi-platform build support (Linux/Windows/macOS)
- Automated release process
- Branch protection guidelines"

# 推送
git push origin main
```

### 步骤 2: 验证 GitHub Actions

1. 打开 https://github.com/hanxueyuan/searchEverything/actions
2. 查看 CI 工作流是否自动触发
3. 等待所有检查完成 (约 5-10 分钟)

### 步骤 3: 配置分支保护

1. 进入 Settings → Branches
2. 点击 **Add branch protection rule**
3. Branch name pattern: `main`
4. 勾选以下选项:
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Require a pull request before merging
   - ✅ Require approvals (1 approval)
5. 选择所有 CI 检查作为必需状态检查
6. 点击 **Create**

### 步骤 4: 测试 CI/CD

创建一个测试 PR:

```bash
# 创建新分支
git checkout -b test-ci-cd

# 做个小修改
echo "# CI/CD Test" >> README.md

# 提交并推送
git add README.md
git commit -m "test: trigger CI/CD"
git push -u origin test-ci-cd
```

然后在 GitHub 上:
1. 创建 Pull Request 到 main 分支
2. 观察 CI 检查自动运行
3. 验证所有检查通过

## ✅ 验证清单

完成配置后，确认以下项目:

- [ ] GitHub Actions 已启用
- [ ] CI 工作流自动触发
- [ ] 所有测试通过
- [ ] 覆盖率报告生成
- [ ] 分支保护规则已配置
- [ ] PR 需要 CI 通过才能合并

## 📋 常用命令

### 本地测试

```bash
# 运行所有测试
cargo test --all-features

# 代码检查
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# 生成覆盖率报告
cargo tarpaulin --all-features --out Html
```

### 发布新版本

```bash
# 更新版本号 (Cargo.toml)
# 然后:
git tag v0.1.1
git push origin v0.1.1
```

## 🔍 故障排除

### CI 没有触发

1. 检查 Actions 是否启用: Settings → Actions
2. 查看工作流文件是否正确
3. 检查 `.github/workflows/` 目录

### 测试失败

1. 本地运行 `cargo test` 复现问题
2. 查看 GitHub Actions 日志
3. 修复后重新推送

### 覆盖率不足

1. 运行 `cargo tarpaulin` 查看哪些代码未覆盖
2. 添加更多测试用例
3. 或调整 `THRESHOLD` (不推荐)

## 📚 更多信息

- [CI_CD_SUMMARY.md](../CI_CD_SUMMARY.md) - 完整实施总结
- [CI_CD.md](CI_CD.md) - 详细配置文档
- [BRANCH_PROTECTION.md](BRANCH_PROTECTION.md) - 分支保护配置

## 🎯 下一步

配置完成后:

1. 开始使用 PR 流程进行所有代码变更
2. 监控覆盖率趋势
3. 定期更新依赖和工具链
4. 根据团队需求调整 CI/CD 配置

---

**需要帮助？** 查看 [CI_CD_SUMMARY.md](../CI_CD_SUMMARY.md) 或联系 @hanxueyuan
