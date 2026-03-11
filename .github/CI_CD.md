# CI/CD 配置文档

本文档描述 searchEverything 项目的持续集成和持续部署 (CI/CD) 流程。

## 工作流程概览

```
┌─────────────────────────────────────────────────────────────────┐
│                        GitHub Events                             │
└─────────────┬──────────────────────┬────────────────────────────┘
              │                      │
              ▼                      ▼
    ┌─────────────────┐    ┌─────────────────┐
    │  Push to main   │    │  Pull Request   │
    │  or develop     │    │                 │
    └────────┬────────┘    └────────┬────────┘
             │                      │
             ▼                      ▼
    ┌─────────────────┐    ┌─────────────────┐
    │    ci.yml       │    │ pr-validation.yml│
    │  - Lint         │    │  - PR Checks     │
    │  - Test         │    │  - Description   │
    │  - Build        │    │  - Security      │
    │  - Coverage     │    │  - Docs          │
    └────────┬────────┘    └─────────────────┘
             │
             ▼
    ┌─────────────────┐
    │  coverage.yml   │
    │  - Coverage     │
    │  - Threshold    │
    │  - PR Comment   │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │   Tag Push      │
    │   (v*.*.*)      │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │  release.yml    │
    │  - Build All    │
    │  - Upload       │
    │  - Publish      │
    └─────────────────┘
```

## 工作流程详情

### 1. CI 主流程 (ci.yml)

**触发条件:**
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 分支

**执行步骤:**

| 任务 | 平台 | 描述 |
|------|------|------|
| `lint` | Ubuntu | 代码格式化检查和 Clippy 静态分析 |
| `test-linux` | Ubuntu | Linux 平台单元测试 |
| `test-windows` | Windows | Windows 平台单元测试 |
| `test-macos` | macOS | macOS 平台单元测试 |
| `build` | 所有平台 | Debug 和 Release 构建验证 |
| `integration-test` | Ubuntu | 集成测试和 OpenClaw Skill 验证 |
| `quality-gate` | Ubuntu | 汇总所有检查结果 |

**质量门禁要求:**
- ✅ 所有测试通过
- ✅ 代码格式化检查通过
- ✅ Clippy 检查无警告
- ✅ 所有平台构建成功

### 2. 覆盖率报告 (coverage.yml)

**触发条件:**
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 分支

**功能:**
- 使用 `cargo-tarpaulin` 生成代码覆盖率报告
- 生成 HTML 和 XML 格式报告
- 检查覆盖率阈值 (≥80%)
- PR 自动评论覆盖率结果

**覆盖率阈值:**
```yaml
THRESHOLD=80  # 80%
```

### 3. PR 验证 (pr-validation.yml)

**触发条件:**
- Pull Request 打开、同步、重新打开

**检查项目:**

| 检查项 | 描述 |
|--------|------|
| PR 基础检查 | 格式化、Clippy、测试 |
| PR 描述检查 | 验证标题和描述完整性 |
| 变更文件检查 | 识别变更的文件类型 |
| 安全审计 | 当 Cargo.toml 变更时运行 `cargo audit` |
| 文档构建 | 构建并上传 API 文档 |

### 4. 发布流程 (release.yml)

**触发条件:**
- Push 版本标签 (格式: `v*.*.*`)

**构建产物:**

| 平台 | 架构 | 产物格式 |
|------|------|----------|
| Linux | x86_64 | .tar.gz + .sha256 |
| Linux | x86_64 (musl) | .tar.gz + .sha256 |
| Linux | aarch64 | .tar.gz + .sha256 |
| Windows | x86_64 | .zip + .sha256 |
| macOS | x86_64 | .tar.gz + .sha256 |
| macOS | aarch64 | .tar.gz + .sha256 |

**发布步骤:**
1. 创建 GitHub Release
2. 构建所有平台二进制文件
3. 压缩并计算 SHA256 校验和
4. 上传到 GitHub Releases
5. (可选) 发布到 crates.io

## 本地开发

### 运行与 CI 相同的检查

```bash
# 代码格式化检查
cargo fmt --all -- --check

# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 运行所有测试
cargo test --all-features --verbose

# 生成覆盖率报告
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --out Html --output-dir ./coverage
```

### 发布新版本

```bash
# 1. 更新 Cargo.toml 版本号
# 2. 提交更改
git add Cargo.toml
git commit -m "chore: bump version to 0.1.1"

# 3. 创建并推送标签
git tag v0.1.1
git push origin v0.1.1

# 4. GitHub Actions 自动触发发布流程
```

## 分支策略

```
main (受保护)
  ↑
  │ Pull Request
  │
develop (开发分支)
  ↑
  │ Feature Branch
  │
feature/*
```

**分支保护规则:**
- `main` 分支需要 PR 和 CI 通过才能合并
- 需要至少 1 个审查者
- 需要所有 CI 检查通过
- 需要覆盖率 ≥80%

## 故障排除

### CI 失败常见原因

1. **格式化失败**: 运行 `cargo fmt`
2. **Clippy 警告**: 运行 `cargo clippy --fix`
3. **测试失败**: 本地运行 `cargo test` 调试
4. **覆盖率不足**: 添加更多测试用例

### 查看覆盖率报告

1. 在 GitHub Actions 中下载 `coverage-report-html` 工件
2. 解压后打开 `index.html`
3. 查看详细的覆盖率统计

## 配置自定义

### 修改覆盖率阈值

编辑 `.github/workflows/coverage.yml`:
```yaml
THRESHOLD=80  # 修改为你需要的百分比
```

### 添加新的测试平台

编辑 `.github/workflows/ci.yml`, 在 `strategy.matrix` 中添加新平台。

### 配置发布到 crates.io

1. 在 crates.io 获取 API token
2. 在 GitHub Secrets 中添加 `CARGO_REGISTRY_TOKEN`
3. 发布流程会自动发布

## 相关文件

- `.github/workflows/ci.yml` - 主 CI 流程
- `.github/workflows/coverage.yml` - 覆盖率报告
- `.github/workflows/pr-validation.yml` - PR 验证
- `.github/workflows/release.yml` - 发布流程
