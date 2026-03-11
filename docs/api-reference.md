# File Search API 参考文档

**版本：** 1.0.0  
**基础 URL：** `http://localhost:8080/api/v1`

---

## 认证

当前版本无需认证（本地服务）。生产环境建议添加 API Key 认证。

---

## 搜索接口

### POST /search

搜索文件。

**请求体：**

```json
{
  "query": "*.rs",
  "filters": {
    "path": "/home/user/projects",
    "ext": [".rs", ".toml"],
    "minSize": 1024,
    "maxSize": 10485760,
    "modifiedAfter": "2024-01-01T00:00:00Z",
    "modifiedBefore": "2024-12-31T23:59:59Z",
    "isDirectory": false
  },
  "limit": 100,
  "offset": 0,
  "sortBy": "name",
  "sortOrder": "asc"
}
```

**参数说明：**

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| query | string | ✅ | 搜索关键词（支持通配符 * 和 ?） |
| filters.path | string | ❌ | 限定搜索路径 |
| filters.ext | array | ❌ | 文件扩展名过滤 |
| filters.minSize | integer | ❌ | 最小文件大小（字节） |
| filters.maxSize | integer | ❌ | 最大文件大小（字节） |
| filters.modifiedAfter | string | ❌ | 修改时间之后（ISO 8601） |
| filters.modifiedBefore | string | ❌ | 修改时间之前（ISO 8601） |
| filters.isDirectory | boolean | ❌ | 是否只搜索目录 |
| limit | integer | ❌ | 返回结果数量（默认 100，最大 1000） |
| offset | integer | ❌ | 偏移量（用于分页） |
| sortBy | string | ❌ | 排序字段：name/path/size/modified/created |
| sortOrder | string | ❌ | 排序方向：asc/desc |

**搜索语法：**

| 语法 | 示例 | 描述 |
|------|------|------|
| 通配符 | `*.rs` | 匹配所有 .rs 文件 |
| 通配符 | `test_?.txt` | 匹配 test_1.txt, test_a.txt 等 |
| AND | `*.rs AND src` | 同时匹配 .rs 和 src |
| OR | `*.rs OR *.toml` | 匹配 .rs 或 .toml |
| NOT | `*.rs NOT test` | 匹配 .rs 但不包含 test |
| 括号 | `(*.rs OR *.toml) AND src` | 分组 |
| 正则 | `regex:^test.*\.rs$` | 正则表达式 |

**响应：**

```json
{
  "success": true,
  "total": 156,
  "limit": 100,
  "offset": 0,
  "results": [
    {
      "path": "/home/user/projects/myapp/src/main.rs",
      "name": "main.rs",
      "size": 4096,
      "isDirectory": false,
      "created": "2024-01-15T10:30:00Z",
      "modified": "2024-03-10T14:20:00Z",
      "accessed": "2024-03-11T09:00:00Z",
      "attributes": {
        "readOnly": false,
        "hidden": false,
        "system": false
      }
    }
  ]
}
```

**错误响应：**

```json
{
  "success": false,
  "error": {
    "code": "INVALID_QUERY",
    "message": "Invalid search query syntax",
    "details": "Unexpected token at position 5"
  }
}
```

---

## 文件信息接口

### POST /file/info

获取文件详细信息。

**请求体：**

```json
{
  "path": "/home/user/projects/myapp/src/main.rs"
}
```

**响应：**

```json
{
  "success": true,
  "file": {
    "path": "/home/user/projects/myapp/src/main.rs",
    "name": "main.rs",
    "size": 4096,
    "isDirectory": false,
    "created": "2024-01-15T10:30:00Z",
    "modified": "2024-03-10T14:20:00Z",
    "accessed": "2024-03-11T09:00:00Z",
    "attributes": {
      "readOnly": false,
      "hidden": false,
      "system": false
    },
    "hash": {
      "md5": "d41d8cd98f00b204e9800998ecf8427e",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    }
  }
}
```

---

## 文件读取接口

### POST /file/read

读取文件内容。

**请求体：**

```json
{
  "path": "/home/user/projects/myapp/src/main.rs",
  "encoding": "utf-8",
  "limit": 1048576
}
```

**参数说明：**

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| path | string | ✅ | 文件路径 |
| encoding | string | ❌ | 文件编码（默认 utf-8） |
| limit | integer | ❌ | 最大读取字节数（默认 1MB） |

**响应：**

```json
{
  "success": true,
  "content": "fn main() {\n    println!(\"Hello, world!\");\n}",
  "encoding": "utf-8",
  "size": 4096,
  "truncated": false
}
```

**支持的编码：**

- utf-8
- gbk
- gb2312
- big5
- iso-8859-1
- ascii

---

## 文件操作接口

### POST /file/copy

复制文件或目录。

**请求体：**

```json
{
  "source": "/home/user/projects/myapp/src/main.rs",
  "destination": "/home/user/backup/main.rs",
  "overwrite": false
}
```

**参数说明：**

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| source | string | ✅ | 源文件路径 |
| destination | string | ✅ | 目标路径 |
| overwrite | boolean | ❌ | 是否覆盖已存在文件（默认 false） |

**响应：**

```json
{
  "success": true,
  "message": "File copied successfully",
  "bytesCopied": 4096
}
```

---

### POST /file/move

移动文件或目录。

**请求体：**

```json
{
  "source": "/home/user/projects/myapp/src/main.rs",
  "destination": "/home/user/projects/myapp/src/main_backup.rs"
}
```

**响应：**

```json
{
  "success": true,
  "message": "File moved successfully"
}
```

---

### POST /file/delete

删除文件或目录。

**请求体：**

```json
{
  "path": "/home/user/projects/myapp/src/main.rs",
  "recursive": false,
  "skipTrash": false
}
```

**参数说明：**

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| path | string | ✅ | 文件/目录路径 |
| recursive | boolean | ❌ | 是否递归删除目录（默认 false） |
| skipTrash | boolean | ❌ | 是否跳过回收站直接删除（默认 false） |

**响应：**

```json
{
  "success": true,
  "message": "File deleted successfully",
  "deletedCount": 1
}
```

---

### POST /file/rename

重命名文件或目录。

**请求体：**

```json
{
  "path": "/home/user/projects/myapp/src/main.rs",
  "newName": "main_backup.rs"
}
```

**响应：**

```json
{
  "success": true,
  "message": "File renamed successfully",
  "newPath": "/home/user/projects/myapp/src/main_backup.rs"
}
```

---

## 索引管理接口

### POST /index/rebuild

强制重建索引。

**请求体：**

```json
{
  "paths": ["/home/user/projects", "/home/user/documents"],
  "excludePatterns": ["*.log", "*.tmp", "node_modules"]
}
```

**参数说明：**

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| paths | array | ❌ | 要索引的路径列表（默认配置的路径） |
| excludePatterns | array | ❌ | 排除的文件/目录模式 |

**响应：**

```json
{
  "success": true,
  "message": "Index rebuild started",
  "jobId": "rebuild_20240311_142000"
}
```

**异步任务查询：**

```bash
GET /index/jobs/rebuild_20240311_142000
```

```json
{
  "jobId": "rebuild_20240311_142000",
  "status": "running",
  "progress": 45,
  "filesIndexed": 45000,
  "totalFiles": 100000,
  "startedAt": "2024-03-11T14:20:00Z",
  "estimatedComplete": "2024-03-11T14:25:00Z"
}
```

---

### GET /index/stats

获取索引统计信息。

**响应：**

```json
{
  "success": true,
  "stats": {
    "totalFiles": 156789,
    "totalDirectories": 12345,
    "totalSize": 5368709120,
    "indexedPaths": [
      "/home/user/projects",
      "/home/user/documents"
    ],
    "excludePatterns": ["*.log", "*.tmp"],
    "lastUpdate": "2024-03-11T14:15:00Z",
    "indexVersion": "1.0.0",
    "memoryUsage": 52428800
  }
}
```

---

### POST /index/pause

暂停索引更新。

**响应：**

```json
{
  "success": true,
  "message": "Index update paused"
}
```

---

### POST /index/resume

恢复索引更新。

**响应：**

```json
{
  "success": true,
  "message": "Index update resumed"
}
```

---

### GET /index/status

获取索引服务状态。

**响应：**

```json
{
  "success": true,
  "status": {
    "running": true,
    "paused": false,
    "monitoring": true,
    "watchedPaths": 2,
    "pendingEvents": 0,
    "uptime": 86400
  }
}
```

---

## WebSocket 接口

### WS /ws

实时搜索和事件订阅。

**连接：**

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

**消息格式：**

所有消息均为 JSON 格式。

**客户端 → 服务端：**

```json
{
  "type": "search",
  "id": "req_001",
  "query": "*.rs",
  "filters": {
    "path": "/home/user/projects"
  },
  "limit": 100
}
```

```json
{
  "type": "subscribe",
  "id": "sub_001",
  "events": ["file.created", "file.modified", "file.deleted"]
}
```

```json
{
  "type": "unsubscribe",
  "id": "sub_001"
}
```

**服务端 → 客户端：**

```json
{
  "type": "result",
  "id": "req_001",
  "file": {
    "path": "/home/user/projects/main.rs",
    "name": "main.rs",
    "size": 4096
  }
}
```

```json
{
  "type": "complete",
  "id": "req_001",
  "total": 50,
  "duration": 15
}
```

```json
{
  "type": "event",
  "subscription": "sub_001",
  "event": "file.created",
  "file": {
    "path": "/home/user/projects/new_file.rs",
    "name": "new_file.rs"
  }
}
```

```json
{
  "type": "error",
  "id": "req_001",
  "error": {
    "code": "INVALID_QUERY",
    "message": "Invalid search query"
  }
}
```

---

## 错误码

| 错误码 | HTTP 状态码 | 描述 |
|--------|-----------|------|
| SUCCESS | 200 | 成功 |
| INVALID_REQUEST | 400 | 请求格式错误 |
| INVALID_QUERY | 400 | 搜索语法错误 |
| FILE_NOT_FOUND | 404 | 文件/目录不存在 |
| PERMISSION_DENIED | 403 | 权限不足 |
| PATH_TRAVERSAL | 403 | 路径遍历攻击检测 |
| FILE_IN_USE | 423 | 文件被占用 |
| INSUFFICIENT_SPACE | 507 | 磁盘空间不足 |
| INDEX_UNAVAILABLE | 503 | 索引服务不可用 |
| INTERNAL_ERROR | 500 | 服务器内部错误 |

---

## 速率限制

| 接口 | 限制 |
|------|------|
| /search | 100 请求/分钟 |
| /file/read | 50 请求/分钟 |
| /file/copy | 20 请求/分钟 |
| /file/move | 20 请求/分钟 |
| /file/delete | 20 请求/分钟 |
| /index/rebuild | 5 请求/小时 |

超出限制返回 HTTP 429 Too Many Requests。

---

## 配置

### 服务端配置文件 (config.yaml)

```yaml
server:
  host: 0.0.0.0
  port: 8080
  ws_port: 8080

index:
  paths:
    - /home/user/projects
    - /home/user/documents
  exclude:
    - "*.log"
    - "*.tmp"
    - "node_modules"
    - ".git"
  max_memory_mb: 512
  snapshot_interval_minutes: 60

security:
  allowed_paths:
    - /home/user
  blocked_paths:
    - /etc
    - /root
  max_read_size_mb: 10

logging:
  level: info
  file: /var/log/file-search.log
```

---

## SDK 示例

### Python SDK

```python
from file_search import FileSearchClient

# 初始化客户端
client = FileSearchClient(host='localhost', port=8080)

# 搜索文件
results = client.search(
    query='*.rs',
    path='/home/user/projects',
    limit=50
)

for file in results['results']:
    print(f"{file['path']} ({file['size']} bytes)")

# 获取文件信息
info = client.get_file_info('/path/to/file.rs')
print(f"Created: {info['file']['created']}")

# 读取文件内容
content = client.read_file('/path/to/file.rs')
print(content)

# 复制文件
client.copy_file('/src/file.rs', '/dst/file.rs')

# 移动文件
client.move_file('/src/file.rs', '/dst/file.rs')

# 删除文件
client.delete_file('/tmp/old_file.rs')

# 重建索引
job = client.rebuild_index()
print(f"Rebuild job started: {job['jobId']}")

# 获取索引统计
stats = client.get_index_stats()
print(f"Total files: {stats['stats']['totalFiles']}")
```

### Node.js SDK

```javascript
const { FileSearchClient } = require('file-search-sdk');

// 初始化客户端
const client = new FileSearchClient({
  host: 'localhost',
  port: 8080
});

// 搜索文件
const results = await client.search('*.rs', {
  path: '/home/user/projects',
  limit: 50
});

results.results.forEach(file => {
  console.log(`${file.path} (${file.size} bytes)`);
});

// 获取文件信息
const info = await client.getFileInfo('/path/to/file.rs');
console.log(`Created: ${info.file.created}`);

// 读取文件内容
const content = await client.readFile('/path/to/file.rs');
console.log(content);

// 复制文件
await client.copyFile('/src/file.rs', '/dst/file.rs');

// 移动文件
await client.moveFile('/src/file.rs', '/dst/file.rs');

// 删除文件
await client.deleteFile('/tmp/old_file.rs');

// 重建索引
const job = await client.rebuildIndex();
console.log(`Rebuild job started: ${job.jobId}`);

// 获取索引统计
const stats = await client.getIndexStats();
console.log(`Total files: ${stats.stats.totalFiles}`);
```

---

**文档结束**
