# 贡献指南 (Contributing Guide)

欢迎为 **searchEverything** 项目做出贡献！本指南将帮助你了解如何参与项目开发。

---

## 📋 目录

1. [行为准则](#行为准则)
2. [开发环境搭建](#开发环境搭建)
3. [如何提交 Issue](#如何提交-issue)
4. [如何提交 PR](#如何提交-pr)
5. [代码风格要求](#代码风格要求)
6. [提交信息规范](#提交信息规范)
7. [测试要求](#测试要求)

---

## 行为准则

本项目采用 [Contributor Covenant](https://www.contributor-covenant.org/) 行为准则。请阅读 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) 了解详细信息。

---

## 开发环境搭建

### 系统要求

- **操作系统**: Linux (推荐), macOS, Windows (WSL2)
- **Rust**: 1.75.0 或更高版本
- **Cargo**: 与 Rust 版本配套

### 安装步骤

1. **安装 Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   rustup update stable
   ```

2. **克隆仓库**
   ```bash
   git clone https://github.com/your-org/searchEverything.git
   cd searchEverything
   ```

3. **安装依赖**
   ```bash
   cargo build
   ```

4. **运行测试**
   ```bash
   cargo test
   ```

5. **配置示例**
   ```bash
   cp config.example.yaml config.yaml
   # 根据实际情况修改配置
   ```

### 开发工具推荐

- **IDE**: VS Code + rust-analyzer, 或 RustRover
- **代码格式化**: rustfmt
- **代码检查**: clippy
- **Git 客户端**: GitKraken, SourceTree, 或命令行

---

## 如何提交 Issue

### Issue 类型

我们欢迎以下类型的 Issue：

| 类型 | 描述 | 标签 |
|------|------|------|
| 🐛 Bug 报告 | 功能异常、崩溃、性能问题 | `bug` |
| 💡 功能请求 | 新功能建议、改进提议 | `enhancement` |
| 📚 文档 | 文档错误、缺失、改进 | `documentation` |
| ❓ 问题咨询 | 使用问题、配置疑问 | `question` |
| 🧪 测试 | 测试用例补充、测试失败 | `testing` |

### Bug 报告模板

提交 Bug 时，请包含以下信息：

```markdown
### Bug 描述
[清晰简洁地描述 bug 是什么]

### 复现步骤
1. 执行步骤 '...'
2. 点击 '....'
3. 观察到 '....'

### 期望行为
[描述你期望发生什么]

### 实际行为
[描述实际发生了什么]

### 环境信息
- OS: [例如 Ubuntu 22.04]
- Rust 版本: [例如 1.75.0]
- searchEverything 版本: [例如 v0.2.0]

### 日志/错误输出
[粘贴相关日志或错误信息]

### 截图
[如有必要，添加截图]

### 附加信息
[任何其他有助于理解问题的信息]
```

### 功能请求模板

```markdown
### 功能描述
[清晰简洁地描述你希望实现的功能]

### 使用场景
[描述这个功能能解决什么问题，在什么场景下使用]

### 实现建议
[如果有实现思路，可以在这里描述]

### 替代方案
[是否考虑过其他解决方案]

### 附加信息
[任何其他有助于理解需求的信息]
```

### Issue 提交前检查

- [ ] 已搜索现有 Issue，确认没有重复
- [ ] 已阅读文档和 FAQ
- [ ] 已提供足够的复现信息（针对 Bug）
- [ ] 已说明功能的使用场景（针对功能请求）

---

## 如何提交 PR

### PR 流程

```
1. Fork 仓库
   ↓
2. 创建功能分支 (git checkout -b feature/your-feature)
   ↓
3. 开发并提交代码 (遵循提交信息规范)
   ↓
4. 运行测试 (cargo test)
   ↓
5. 代码格式化 (cargo fmt)
   ↓
6. 代码检查 (cargo clippy)
   ↓
7. 推送到远程 (git push origin feature/your-feature)
   ↓
8. 创建 Pull Request
```

### PR 模板

创建 PR 时，请填写以下信息：

```markdown
## 📝 变更描述
[清晰简洁地描述这个 PR 做了什么]

## 🔗 关联 Issue
[例如 Fixes #123 或 Closes #456]

## ✅ 测试
- [ ] 已添加/更新单元测试
- [ ] 已添加/更新集成测试
- [ ] 所有测试通过 (cargo test)
- [ ] 已手动测试关键功能

## 📚 文档
- [ ] 已更新相关文档
- [ ] 已添加/更新注释
- [ ] 已更新 CHANGELOG.md

## 🔍 代码质量
- [ ] 代码已通过 clippy 检查 (cargo clippy -- -D warnings)
- [ ] 代码已通过格式化检查 (cargo fmt --check)
- [ ] 无新的编译警告

## 🖼️ 截图/录屏
[如有 UI 变更，请添加截图或录屏]

## 📋 检查清单
- [ ] 我的代码遵循项目的代码风格
- [ ] 我已进行自审
- [ ] 变更是向后兼容的（或已说明破坏性变更）
- [ ] 我已阅读贡献指南
```

### PR 大小建议

- 单个 PR 建议不超过 500 行代码变更
- 大型功能请拆分为多个小 PR
- 每个 PR 聚焦单一功能或修复

---

## 代码风格要求

### Rust 代码规范

1. **格式化**: 使用 `cargo fmt` 格式化所有代码
   ```bash
   cargo fmt
   ```

2. **代码检查**: 使用 `cargo clippy` 检查代码质量
   ```bash
   cargo clippy -- -D warnings
   ```

3. **命名规范**:
   - 类型、结构体、枚举：`PascalCase`
   - 函数、变量、模块：`snake_case`
   - 常量：`UPPER_SNAKE_CASE`
   - 生命周期：`'a`, `'b` 等简短命名

4. **错误处理**:
   - 使用 `Result<T, E>` 处理可恢复错误
   - 使用 `panic!` 仅处理不可恢复错误
   - 使用 `?` 操作符传播错误
   - 自定义错误类型实现 `std::error::Error`

5. **注释规范**:
   - 公共 API 必须使用 `///` 文档注释
   - 复杂逻辑需要解释性注释
   - 避免无意义的注释（如 `// 增加计数器`）

6. **测试规范**:
   - 每个公共函数应有单元测试
   - 边界条件必须测试
   - 错误路径必须测试
   - 使用 `#[cfg(test)]` 隔离测试代码

### 文件组织

```
src/
├── main.rs          # 程序入口
├── lib.rs           # 库入口（如有）
├── config.rs        # 配置处理
├── error.rs         # 错误定义
├── cli.rs           # 命令行解析
└── search/          # 功能模块
    ├── mod.rs
    ├── engine.rs
    └── indexer.rs
```

---

## 提交信息规范

### Commit Message 格式

我们采用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

| Type | 描述 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `style` | 代码格式（不影响代码运行） |
| `refactor` | 重构（非新功能，非 bug 修复） |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建过程或辅助工具变动 |
| `ci` | CI/CD 配置变更 |

### Scope 范围（可选）

描述变更影响的模块，例如：
- `cli` - 命令行接口
- `search` - 搜索核心
- `config` - 配置处理
- `docs` - 文档

### Subject 主题

- 使用祈使句、现在时态（"add" 而非 "added" 或 "adds"）
- 首字母不大写
- 末尾不加句号
- 长度不超过 50 字符

### Body 正文（可选）

- 描述变更的动机和与之前行为的对比
- 每行不超过 72 字符

### Footer 页脚（可选）

- 关联 Issue：`Fixes #123`, `Closes #456`
- 破坏性变更：`BREAKING CHANGE: <描述>`

### 提交示例

```bash
# 新功能
feat(search): 添加全文搜索支持

实现基于倒排索引的全文搜索功能
- 添加 Indexer 结构体
- 实现 tokenize 函数
- 添加搜索 API

Fixes #42

# Bug 修复
fix(cli): 修复配置文件路径解析错误

当路径包含空格时，解析会失败。
使用 shellexpand 库处理路径。

Closes #57

# 文档更新
docs(readme): 更新安装说明

添加 Windows 安装步骤和注意事项

# 重构
refactor(error): 统一错误处理机制

- 创建 SearchError 枚举
- 实现 Display 和 Error trait
- 更新所有错误返回点
```

---

## 测试要求

### 测试类型

1. **单元测试**: 测试单个函数/方法的功能
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_parse_config() {
           // ...
       }
   }
   ```

2. **集成测试**: 测试模块间协作 (`tests/` 目录)
   ```rust
   // tests/integration_test.rs
   #[test]
   fn test_search_flow() {
       // ...
   }
   ```

3. **文档测试**: 确保文档示例可运行
   ```rust
   /// ```
   /// let result = search("query");
   /// assert!(result.is_ok());
   /// ```
   pub fn search(query: &str) -> Result<Vec<Hit>> {
       // ...
   }
   ```

### 测试覆盖率

- 新功能必须包含测试
- 核心模块覆盖率建议 > 80%
- 使用 `cargo tarpaulin` 生成覆盖率报告

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 生成覆盖率报告
cargo tarpaulin --out Html
```

---

## 审核流程

提交 PR 后，将进入审核流程：

1. **自动化检查**: CI 自动运行测试、clippy、fmt 检查
2. **代码审核**: 至少需要 1 名维护者审核通过
3. **QA 验证**: QA 工程师验证功能和测试
4. **合并**: 审核通过后由维护者合并

详细审核标准请参考 [GOVERNANCE.md](GOVERNANCE.md)。

---

## 获取帮助

- 📖 阅读 [README.md](README.md) 和 [docs/](docs/) 文档
- 💬 在 Issue 中提问
- 📧 联系维护者团队

感谢你的贡献！🎉
