# searchEverything

[中文](README.md) | [English](README_en.md)

---

跨平台文件搜索 CLI 工具，支持 OpenClaw 集成。

## 特性

- 🔍 **三种搜索模式**：通配符（默认）、正则表达式、模糊搜索
- ⚡ **高性能**：Rust 实现，Trie 树索引，O(m) 前缀搜索
- 🖥️ **跨平台**：Windows/Linux/macOS
- 🤖 **OpenClaw 集成**：JSON 输出，大模型友好
- 📁 **索引管理**：手动添加/移除索引目录
- 🔔 **实时监控**：Linux inotify / Windows USN Journal
- 📊 **流式输出**：边匹配边输出，支持分页
- 📝 **审计日志**：操作历史追踪，支持查询导出

## 安装

```bash
# 从源码编译
cargo build --release

# 复制二进制到 PATH
cp target/release/searchEverything /usr/local/bin/

# 验证安装
searchEverything --version
```

## 基本用法

### 搜索文件

```bash
# 通配符模式（默认）
searchEverything search "*.pdf" --path D:/work
searchEverything search "report*"

# 正则表达式模式
searchEverything search --regex ".*\.pdf$"
searchEverything search --regex "report-\d{4}\.docx"

# 模糊搜索
searchEverything search --fuzzy "repot"  # 匹配 report
searchEverything search --fuzzy "readme"  # 匹配 README.md

# JSON 输出（OpenClaw 集成，默认）
searchEverything search "*.md" --format json
```

### 文件操作

```bash
# 查看文件信息
searchEverything info README.md
searchEverything info D:/work/report.pdf --json

# 读取文件内容
searchEverything cat config.yaml
searchEverything cat app.log --lines 50  # 前 50 行
searchEverything cat app.log --tail --lines 10  # 末尾 10 行

# 复制/移动/删除
searchEverything copy "*.txt" /backup/
searchEverything move temp.zip /archive/
searchEverything delete "*.tmp" --force
```

### 索引管理

```bash
# 索引状态（首次启动自动索引全系统）
searchEverything index status

# 排除特定目录（隐私/系统目录）
searchEverything index exclude add --path C:/Windows
searchEverything index exclude add --path /proc

# 查看排除列表
searchEverything index exclude list

# 移除排除
searchEverything index exclude remove --path C:/Windows

# 添加额外索引路径
searchEverything index add --path /mnt/data

# 移除索引路径
searchEverything index remove --path /mnt/data

# 重建索引（全量）
searchEverything index rebuild

# 重建特定路径
searchEverything index rebuild --path D:/work

# 快速查看状态
searchEverything index-status

# 实时监控（Linux inotify / Windows USN）
searchEverything index watch /home    # Linux
searchEverything index watch C:/work  # Windows

### 流式输出

```bash
# 流式输出（边匹配边输出）
searchEverything search "*.log" --stream

# 分页显示（每页 20 条结果）
searchEverything search "*.txt" --stream --page-size 20

# 大量文件搜索时自动启用进度显示
searchEverything search "*.pdf" --stream
```

### 审计日志

```bash
# 查看最近的日志
searchEverything audit list
searchEverything audit list --limit 50
searchEverything audit list --command search

# 搜索日志
searchEverything audit search --command search --after 2024-01-01T00:00:00+08:00
searchEverything audit search --result error

# 导出日志
searchEverything audit export --format json --output audit.json
searchEverything audit export --format csv --output audit.csv

# 查看统计信息
searchEverything audit stats

# 清理旧日志
searchEverything audit cleanup --days 30
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
→ searchEverything search "*.pdf" --path D:/work --format json

用户：用正则搜索 2024 年的报告
→ searchEverything search --regex "report-2024.*\.docx"

用户：模糊查找 readme 文件
→ searchEverything search --fuzzy "readme"

用户：查看这个文件的信息
→ searchEverything info D:/work/report.pdf

用户：读取昨天的会议记录
→ searchEverything cat meeting-notes.md --lines 50

用户：把这些文件复制到备份目录
→ searchEverything copy "D:/work/*.txt" "D:/backup/"
```

## 输出格式

### JSON 输出（默认，大模型友好）

```json
{
  "results": [
    {
      "path": "/home/user/document.pdf",
      "size": 1048576,
      "size_human": "1.0 MB",
      "modified": "2026-03-10T14:30:00+00:00",
      "modified_human": "2 天前",
      "file_type": "pdf",
      "is_dir": false
    }
  ],
  "total": 1,
  "scanned": 15234,
  "search_time_ms": 45,
  "summary": {
    "total_size": 1048576,
    "total_size_human": "1.0 MB",
    "file_types": "pdf: 1",
    "oldest_file": "/home/user/document.pdf",
    "newest_file": "/home/user/document.pdf"
  },
  "error": null
}
```

### 错误响应

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "NOT_FOUND",
    "message": "文件未找到",
    "details": "路径 /not/exist 不存在",
    "suggestion": "请检查路径是否正确，或使用 searchEverything search 搜索文件"
  }
}
```

## 命令参考

| 命令 | 说明 | 示例 |
|------|------|------|
| `searchEverything search <pattern>` | 搜索文件 | `searchEverything search "*.pdf" --path D:/work` |
| `searchEverything search --regex <pattern>` | 正则搜索 | `searchEverything search --regex ".*\.pdf$"` |
| `searchEverything search --fuzzy <pattern>` | 模糊搜索 | `searchEverything search --fuzzy "repot"` |
| `searchEverything info <file>` | 查看文件信息 | `searchEverything info README.md` |
| `searchEverything cat <file>` | 读取文件内容 | `searchEverything cat config.yaml --lines 50` |
| `searchEverything copy <src> <dest>` | 复制文件 | `searchEverything copy "*.txt" /backup/` |
| `searchEverything move <src> <dest>` | 移动文件 | `searchEverything move temp.zip /archive/` |
| `searchEverything delete <path>` | 删除文件 | `searchEverything delete "*.tmp" --force` |
| `searchEverything index add` | 添加索引目录 | `searchEverything index add --path D:/work` |
| `searchEverything index remove` | 移除索引目录 | `searchEverything index remove --path D:/work` |
| `searchEverything index list` | 列出索引目录 | `searchEverything index list` |
| `searchEverything index rebuild` | 重建索引 | `searchEverything index rebuild --path D:/` |

## 技术栈

- **语言**: Rust
- **CLI 解析**: clap
- **序列化**: serde + serde_json
- **文件搜索**: walkdir + glob + regex
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
