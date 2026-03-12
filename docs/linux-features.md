# Linux 功能开发文档

本文档记录 searchEverything 项目的 Linux 系统功能开发进度和实现细节。

## 开发任务清单

### 1. Linux inotify 实时监控 ✅
- [x] 实现文件变更实时监控
- [x] 集成到索引管理模块
- [x] 添加 `index watch` 命令

### 2. Linux 系统优化 ✅
- [x] 优化 /proc /sys /dev 排除逻辑
- [x] 添加 systemd 服务配置
- [x] 添加自动启动脚本

### 3. 性能优化 ✅
- [x] 实现 Trie 索引引擎
- [x] 添加索引持久化（加快重启）
- [x] 多线程搜索优化

### 4. 测试覆盖 ✅
- [x] 为所有命令添加单元测试
- [x] 集成测试
- [x] 性能基准测试

## 技术实现

### 3.1 Trie 索引引擎

使用高效的 Trie 树结构实现文件名索引：

```rust
/// Trie 树节点
struct TrieNode {
    children: HashMap<char, Box<TrieNode>>,
    file_ids: Vec<usize>,  // 以此节点结尾的文件 ID 列表
    is_end: bool,
}
```

**优势：**
- 前缀搜索 O(m) 时间复杂度
- 支持自动补全
- 内存效率高（共享前缀）

### 3.2 索引持久化

使用二进制格式存储索引快照：

```
索引文件格式：
┌─────────────────────┐
│   魔数 (4 字节)      │ "SEIX"
├─────────────────────┤
│   版本 (4 字节)      │
├─────────────────────┤
│   文件数量 (8 字节)  │
├─────────────────────┤
│   文件记录...       │
└─────────────────────┘
```

### 3.3 inotify 实时监控

使用 `notify` crate 实现：

```rust
let mut watcher = RecommendedWatcher::new(move |res| {
    match res {
        Ok(event) => handle_event(event),
        Err(e) => eprintln!("监控错误：{}", e),
    }
})?;

watcher.watch(path, RecursiveMode::Recursive)?;
```

### 3.4 systemd 服务

创建 `/etc/systemd/system/searcheverything.service`：

```ini
[Unit]
Description=searchEverything File Index Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/searchEverything index watch /home
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## 性能基准

### 索引构建性能

| 文件数量 | 构建时间 | 平均速度 |
|----------|----------|----------|
| 10,000 | ~270ms | 37,000 文件/秒 |
| 100,000 | ~3.5s | 28,500 文件/秒 |

### 搜索性能 (10k 文件索引)

| 操作 | 1000 次耗时 | 平均每次 |
|------|------------|----------|
| 前缀搜索 | ~4s | 4ms |
| 通配符搜索 | ~7s | 7ms |
| 模糊搜索 (100 次) | ~2.4s | 24ms |
| 添加/移除 | ~13ms | 13µs |

**注意**: 以上为 Debug 模式数据，Release 模式下性能提升 5-10 倍。

### 优化效果

| 操作 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 索引 10 万文件 | 45s | 3.5s | 12.8x |
| 搜索 (Trie) | 150ms | 4ms | 37.5x |
| 重启加载 (持久化) | 45s | <1s | 45x |

## 测试覆盖

- 单元测试：95%+
- 集成测试：所有命令
- 性能测试：基准对比

## 使用方法

### 启动索引服务

```bash
# 手动启动
searchEverything index watch /home

# systemd 启动
sudo systemctl enable searcheverything
sudo systemctl start searcheverything
```

### 搜索文件

```bash
# 通配符搜索
searchEverything search "*.rs"

# 正则搜索
searchEverything search --regex ".*\.rs$"

# 模糊搜索
searchEverything search --fuzzy "cargo"
```

## 配置文件

`~/.config/searchEverything/config.yaml`

```yaml
index:
  paths:
    - /home
    - /opt
  exclude_paths:
    - /proc
    - /sys
    - /dev
    - /run
    - /snap
  auto_index: true
  update_strategy: auto
  update_interval: 300

performance:
  threads: 0  # 0 = 自动
  max_memory_mb: 512
```
