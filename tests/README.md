# searchEverything OpenClaw 集成测试套件

本目录包含 searchEverything Skill 与 OpenClaw 集成的完整测试方案。

## 📁 目录结构

```
tests/searchEverything/
├── README.md                    # 本文件
├── TEST_PLAN.md                 # 测试计划文档
├── TEST_CASES.md                # 完整测试用例清单
├── MANUAL_CHECKLIST.md          # 手动测试检查清单
├── package.json                 # 测试项目配置
├── scripts/
│   ├── test.js                  # Node.js 自动化测试脚本
│   └── run_tests.sh             # Bash 自动化测试脚本
├── reports/                     # 测试报告输出目录
├── logs/                        # 测试日志目录
└── .github/
    └── workflows/
        └── ci.yml               # GitHub Actions CI/CD 配置
```

## 🚀 快速开始

### 前置条件

- Node.js >= 18.0.0
- OpenClaw (可选，用于完整集成测试)
- searchEverything Skill 源码

### 安装测试依赖

```bash
cd tests/searchEverything
npm install
```

### 运行测试

#### 运行所有测试

```bash
# 使用 Node.js 脚本
npm test

# 或使用 Bash 脚本
npm run test:manual
```

#### 运行特定类别测试

```bash
# 安装流程测试
npm run test:install

# 使用流程测试
npm run test:usage

# 卸载流程测试
npm run test:uninstall

# 边界场景测试
npm run test:boundary

# 集成测试
npm run test:integration
```

#### 运行单个测试用例

```bash
node scripts/test.js --test SE-OC-INST-001
```

## 📋 测试用例统计

| 类别 | P0 | P1 | P2 | 总计 | 自动化率 |
|------|----|----|----|------|---------|
| 安装流程 | 7 | 8 | 5 | 20 | 90% |
| 使用流程 | 4 | 12 | 6 | 22 | 91% |
| 卸载流程 | 5 | 8 | 4 | 17 | 88% |
| 边界场景 | 0 | 3 | 9 | 12 | 83% |
| 集成测试 | 5 | 2 | 3 | 10 | 50% |
| **总计** | **21** | **33** | **27** | **81** | **84%** |

## 📊 测试报告

测试执行后，报告将生成在 `reports/` 目录：

```
reports/
└── test-report-20260312-123456.md
```

报告包含：
- 测试执行时间
- 通过/失败/跳过统计
- 通过率
- 详细结果
- 改进建议

## 🔧 CI/CD 集成

### GitHub Actions

本项目包含完整的 GitHub Actions 工作流配置 (`.github/workflows/ci.yml`)，支持：

- ✅ 多平台测试（Linux/macOS/Windows）
- ✅ 多架构测试（x86_64/aarch64）
- ✅ 单元测试
- ✅ 集成测试
- ✅ E2E 测试
- ✅ 性能测试
- ✅ 安全扫描
- ✅ 代码质量检查
- ✅ 自动发布检查

### 触发条件

- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 分支
- 每日定时运行（UTC 2:00）
- 手动触发

### 查看测试结果

1. 访问 GitHub Actions 标签页
2. 选择对应的工作流运行
3. 查看各 Job 的详细输出
4. 下载测试报告 artifact

## 📝 手动测试

对于需要真实环境验证的场景，使用 `MANUAL_CHECKLIST.md`：

1. 打印检查清单
2. 逐项执行测试
3. 记录结果
4. 汇总报告

## 🎯 测试覆盖范围

### 安装流程
- [x] 自动安装
- [x] 手动安装
- [x] 离线安装
- [x] 指定版本安装
- [x] 安装失败处理

### 使用流程
- [x] OpenClaw 集成
- [x] 配置管理
- [x] 日志集成
- [x] 更新机制

### 卸载流程
- [x] 完全卸载
- [x] 保留配置卸载
- [x] 仅禁用
- [x] 卸载失败处理

### 边界场景
- [x] 多用户环境
- [x] 多版本共存
- [x] 系统环境（无网络/代理/受限权限）

### 集成测试
- [x] 真实 OpenClaw 环境
- [x] 与其他 Skill 协作

## 🐛 问题报告

发现测试问题时，请提供：

1. 测试用例 ID
2. 执行环境信息
3. 错误日志
4. 复现步骤

## 📈 持续改进

测试套件应持续改进：

1. 新增功能时添加对应测试
2. 发现 Bug 时添加回归测试
3. 定期审查测试覆盖率
4. 优化慢速测试

## 🔗 相关文档

- [测试计划](./TEST_PLAN.md)
- [测试用例](./TEST_CASES.md)
- [手动检查清单](./MANUAL_CHECKLIST.md)
- [searchEverything Skill 文档](../../skills/searchEverything/README.md)
- [OpenClaw 文档](https://openclaw.dev)

## 📄 许可证

MIT License
