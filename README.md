# searchEverything

A cross-platform file search CLI tool with OpenClaw integration.

## 快速开始

### 安装

```bash
# 从源码编译
cargo build --release

# 复制二进制到 PATH
cp target/release/se /usr/local/bin/
```

### 基本用法

```bash
# 搜索文件
se search "*.pdf" --path D:/work

# 快速搜索（别名）
se "*.rs"

# JSON 输出（OpenClaw 集成）
se search "*.md" --format json

# 查看文件信息
se info README.md

# 读取文件内容
se cat config.yaml --lines 50

# 文件操作
se copy "*.txt" /backup/
se move temp.zip /archive/
se delete "*.tmp" --force

# 索引管理
se index status
se index rebuild --path D:/
```

## OpenClaw Skill 集成

### 安装 Skill

```bash
# 复制 Skill 定义
cp skills/openclaw-skill.yaml ~/.openclaw/skills/

# 注册技能
openclaw skills register searchEverything
```

### 使用示例

```
用户：搜索工作目录的 PDF 文件
→ se search "*.pdf" --path D:/work --format json

用户：查看这个文件的信息
→ se info D:/work/report.pdf

用户：读取昨天的会议记录
→ se cat meeting-notes.md --lines 50

用户：把这些文件复制到备份目录
→ se copy "D:/work/*.txt" "D:/backup/"
```

## 命令参考

| 命令 | 说明 | 示例 |
|------|------|------|
| `se search <pattern>` | 搜索文件 | `se "*.pdf" --path D:/work` |
| `se info <file>` | 查看文件信息 | `se info README.md` |
| `se cat <file>` | 读取文件内容 | `se cat config.yaml --lines 50` |
| `se copy <src> <dest>` | 复制文件 | `se copy "*.txt" /backup/` |
| `se move <src> <dest>` | 移动文件 | `se move temp.zip /archive/` |
| `se delete <path>` | 删除文件 | `se delete "*.tmp" --force` |
| `se index status` | 索引状态 | `se index status` |
| `se index rebuild` | 重建索引 | `se index rebuild --path D:/` |

## 技术栈

- **语言**: Rust
- **CLI 解析**: clap
- **序列化**: serde + serde_json
- **文件搜索**: walkdir + glob
- **跨平台**: Windows USN / Linux inotify

## 开发

```bash
# 开发模式
cargo run -- search "*.rs"

# 发布构建
cargo build --release

# 运行测试
cargo test
```

## 项目结构

```
searchEverything/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI 入口
│   ├── commands/        # 命令实现
│   ├── error.rs         # 错误定义
│   └── output.rs        # 输出格式化
├── skills/
│   └── openclaw-skill.yaml
└── docs/
```

## License

MIT
