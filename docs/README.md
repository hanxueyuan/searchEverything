# 类 Everything 文件搜索工具

> 跨平台文件搜索工具，支持 Linux/Windows，提供大模型接口和 OpenClaw Skill 集成

---

## 📁 项目文件

| 文件 | 描述 |
|------|------|
| `README.md` | 项目说明（本文件） |
| `executive-summary.md` | **执行摘要** - 快速了解项目核心内容 |
| `technical-proposal.md` | **技术方案** - 详细的技术调研报告 |
| `api-reference.md` | **API 文档** - 完整的接口参考 |
| `openclaw-skill.yaml` | **Skill 定义** - OpenClaw 技能配置 |

---

## 🎯 项目目标

开发一套类似 **Everything** 的文件搜索工具，具有以下特点：

1. **跨平台** - 支持 Windows 和 Linux
2. **快速搜索** - 基于 USN Journal (Windows) / inotify (Linux) 实时索引
3. **大模型接口** - REST API + WebSocket + SDK
4. **OpenClaw 集成** - 原生 Skill 支持

---

## 📊 核心结论

| 评估项 | 结论 |
|--------|------|
| **技术可行性** | ✅ **高** |
| **开发周期** | ⏱️ **6 周** (MVP) |
| **开发难度** | 📈 **中等** |
| **建议** | ✅ **推荐启动** |

---

## 🏗️ 技术架构

```
┌─────────────────────────────────────────────────────────┐
│         前端 (Tauri + React) / CLI / REST API          │
├─────────────────────────────────────────────────────────┤
│                  核心服务 (Rust)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │  索引引擎   │  │  搜索引擎   │  │   文件操作      │ │
│  │  USN/inotify│  │  Trie/正则  │  │  复制/移动/删除 │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 技术选型

| 组件 | 选型 |
|------|------|
| 后端语言 | **Rust** |
| 前端框架 | **Tauri + React** |
| 索引存储 | **内存 Trie + 二进制快照** |
| API 协议 | **REST + WebSocket** |

---

## 📅 开发计划

```
Week 1-3: 核心功能
├─ Week 1: 索引引擎 (USN/inotify)
├─ Week 2: 搜索引擎 (Trie/正则/过滤)
└─ Week 3: CLI + 测试

Week 4-5: GUI 和跨平台
├─ Week 4: Tauri 前端
└─ Week 5: 打包 + 适配

Week 6: 大模型接口
├─ REST API + WebSocket
├─ Python/Node.js SDK
└─ OpenClaw Skill
```

---

## 🔌 API 接口

### 核心接口

| 接口 | 方法 | 描述 |
|------|------|------|
| `/api/v1/search` | POST | 搜索文件 |
| `/api/v1/file/info` | POST | 获取文件信息 |
| `/api/v1/file/read` | POST | 读取文件内容 |
| `/api/v1/file/copy` | POST | 复制文件 |
| `/api/v1/file/move` | POST | 移动文件 |
| `/api/v1/file/delete` | POST | 删除文件 |
| `/api/v1/index/rebuild` | POST | 重建索引 |
| `/api/v1/index/stats` | GET | 索引统计 |
| `/ws` | WebSocket | 实时搜索 |

### Python SDK 示例

```python
from file_search import FileSearchClient

client = FileSearchClient(host='localhost', port=8080)

# 搜索文件
results = client.search('*.rs', path='/home/user/projects', limit=50)

# 获取文件信息
info = client.get_file_info('/path/to/file.rs')

# 读取文件内容
content = client.read_file('/path/to/file.rs')

# 文件操作
client.copy_file('/src/file.rs', '/dst/file.rs')
client.delete_file('/tmp/old_file.rs')

# 索引管理
stats = client.get_index_stats()
```

### OpenClaw Skill 示例

```javascript
const fileSearch = await loadSkill('file-search');

// 搜索文件
const results = await fileSearch.search('*.rs', {
  path: '/home/user/projects',
  limit: 50
});

// 读取文件
const content = await fileSearch.readFile('/path/to/file.rs');

// 索引状态
const stats = await fileSearch.index.stats();
```

---

## 🔍 搜索语法

| 语法 | 示例 | 描述 |
|------|------|------|
| 通配符 | `*.rs` | 匹配所有 .rs 文件 |
| 通配符 | `test_?.txt` | 匹配 test_1.txt, test_a.txt |
| AND | `*.rs AND src` | 同时匹配 .rs 和 src |
| OR | `*.rs OR *.toml` | 匹配 .rs 或 .toml |
| NOT | `*.rs NOT test` | 匹配 .rs 但不包含 test |
| 括号 | `(*.rs OR *.toml) AND src` | 分组 |
| 正则 | `regex:^test.*\.rs$` | 正则表达式 |

---

## ⚠️ 风险与缓解

| 风险 | 概率 | 缓解措施 |
|------|------|----------|
| USN 权限问题 | 中 | 需要管理员权限，提供降级方案 |
| inotify watch 限制 | 低 | 合理设置监控目录 |
| 大文件内存占用 | 中 | 支持选择性索引目录 |

---

## 💡 关键建议

1. **优先开发 CLI 版本** - 快速验证核心功能
2. **渐进式开发** - 先 Windows 后 Linux
3. **开源策略** - 考虑开源获取社区贡献
4. **性能基准** - 与 Everything/fsearch 对比测试
5. **安全考虑** - 文件操作需要权限验证

---

## 📋 交付物清单

- ✅ **调研报告** - Everything 功能和技术分析
- ✅ **技术方案** - 架构设计、技术选型
- ✅ **开发计划** - 时间表、里程碑
- ✅ **接口设计** - API 文档草稿
- ✅ **Skill 设计** - OpenClaw 技能接口定义

---

## 🚀 下一步行动

如需启动项目：

1. **确认技术栈** - Rust + Tauri 是否接受
2. **确定优先级** - 先 CLI 还是直接 GUI
3. **资源配置** - 1-2 名 Rust 开发人员
4. **启动开发** - 按 6 周计划推进

---

## 📞 联系

如有疑问，请联系 Lisa 团队。

---

**结论：项目技术可行，建议启动。**
