# searchEverything OpenClaw 集成设计文档

**版本**: 1.0  
**作者**: 韩雪原  
**日期**: 2026-03-12  
**状态**: 已完成

---

## 📋 目录

1. [设计目标](#1-设计目标)
2. [架构设计](#2-架构设计)
3. [安装流程设计](#3-安装流程设计)
4. [使用流程设计](#4-使用流程设计)
5. [卸载流程设计](#5-卸载流程设计)
6. [目录结构设计](#6-目录结构设计)
7. [OpenClaw API 集成](#7-openclaw-api-集成)
8. [错误处理](#8-错误处理)
9. [安全考虑](#9-安全考虑)
10. [测试策略](#10-测试策略)

---

## 1. 设计目标

### 1.1 核心目标

深度集成 OpenClaw，实现"**Skill 安装即完成部署，Skill 卸载即清理干净**"。

### 1.2 设计原则

| 原则 | 说明 |
|------|------|
| **零配置** | 用户无需手动配置，安装即可用 |
| **自动化** | 安装、更新、卸载全流程自动化 |
| **可回滚** | 任何操作失败都能安全回滚 |
| **可审计** | 所有操作都有日志记录 |
| **跨平台** | Linux、macOS、Windows 一致体验 |

### 1.3 用户体验目标

- **安装**: 一条命令完成，无需关心细节
- **使用**: 自然语言触发，无需记忆命令
- **更新**: 自动检查，一键更新
- **卸载**: 完全清理，无残留

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    OpenClaw Framework                    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │           searchEverything Skill                │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │    │
│  │  │   安装钩子   │  │   更新钩子   │  │ 卸载钩子 │ │    │
│  │  │ on_install  │  │ on_update   │  │on_uninstall│ │    │
│  │  └──────┬──────┘  └──────┬──────┘  └────┬────┘ │    │
│  │         │                │               │      │    │
│  │  ┌──────▼────────────────▼───────────────▼──────┐ │    │
│  │  │          OpenClaw Skill 配置                  │ │    │
│  │  │       (openclaw-skill.yaml)                   │ │    │
│  │  └───────────────────────────────────────────────┘ │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   脚本层 (Scripts)                       │
│  ┌──────────────────┐  ┌──────────────────┐            │
│  │ openclaw-install │  │ openclaw-update  │            │
│  │      .sh         │  │      .sh         │            │
│  └──────────────────┘  └──────────────────┘            │
│  ┌──────────────────┐                                   │
│  │openclaw-uninstall│                                   │
│  │      .sh         │                                   │
│  └──────────────────┘                                   │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  二进制文件层                             │
│              searchEverything (Rust CLI)                │
└─────────────────────────────────────────────────────────┘
```

### 2.2 组件职责

| 组件 | 职责 |
|------|------|
| `openclaw-skill.yaml` | 定义 Skill 元数据、命令、钩子、权限 |
| `openclaw-install.sh` | 下载、安装、配置、注册 |
| `openclaw-update.sh` | 版本检查、下载、更新、验证 |
| `openclaw-uninstall.sh` | 停止进程、删除文件、清理配置 |
| `searchEverything` | 核心 CLI 功能 |

---

## 3. 安装流程设计

### 3.1 安装流程图

```
                    用户触发安装
                         │
                         ▼
            ┌────────────────────────┐
            │  OpenClaw 调用钩子      │
            │  on_install.script     │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │  openclaw-install.sh   │
            └───────────┬────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
        ▼               ▼               ▼
   检测环境        下载二进制       创建目录
        │               │               │
        └───────────────┼───────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    安装到目标目录       │
            │   (~/.local/bin)       │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    设置执行权限         │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    验证安装成功         │
            │   (--version)          │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    配置 PATH           │
            │   (自动添加到 shell)    │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    创建默认配置         │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    注册到 OpenClaw     │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    生成安装报告         │
            └───────────┬────────────┘
                        │
                        ▼
                   ✅ 安装完成
```

### 3.2 安装脚本功能清单

| 步骤 | 功能 | 脚本函数 |
|------|------|----------|
| 1 | 检测 OpenClaw 环境 | `check_openclaw()` |
| 2 | 检测平台（OS + 架构） | `detect_platform()` |
| 3 | 检查依赖（curl 等） | `check_dependencies()` |
| 4 | 获取最新版本 | `get_latest_version()` |
| 5 | 构建下载 URL | `build_download_url()` |
| 6 | 创建安装目录 | `setup_install_dirs()` |
| 7 | 下载二进制文件 | `download_binary()` |
| 8 | 设置执行权限 | `set_permissions()` |
| 9 | 验证安装 | `verify_installation()` |
| 10 | 配置 PATH | `check_path()`, `auto_add_to_path()` |
| 11 | 创建默认配置 | `create_default_config()` |
| 12 | 注册到 OpenClaw | `register_with_openclaw()` |
| 13 | 生成安装报告 | `generate_install_report()` |

### 3.3 安装模式

#### 3.3.1 自动模式

```bash
# OpenClaw 自动调用
openclaw skills install searchEverything
```

- 由 OpenClaw 框架自动触发
- 无需用户干预
- 适合首次安装

#### 3.3.2 手动模式

```bash
# 用户手动执行
./scripts/openclaw-install.sh
./scripts/openclaw-install.sh --verbose
```

- 用户主动执行
- 可添加参数控制行为
- 适合调试和重新安装

#### 3.3.3 离线模式

```bash
# 使用预下载文件
./scripts/openclaw-install.sh --offline ./searchEverything
```

- 无需网络连接
- 使用本地二进制文件
- 适合内网环境

### 3.4 安装报告

安装完成后生成 JSON 格式报告：

```json
{
  "installation": {
    "timestamp": "2026-03-12T10:30:00+08:00",
    "mode": "auto",
    "version": "0.2.0",
    "platform": {
      "os": "linux",
      "arch": "x86_64"
    },
    "paths": {
      "install_dir": "/home/user/.local/bin",
      "config_dir": "/home/user/.openclaw/skills/searchEverything",
      "log_dir": "/home/user/.openclaw/skills/searchEverything/logs",
      "binary": "/home/user/.local/bin/searchEverything"
    },
    "openclaw": {
      "enabled": true,
      "skill_dir": "/home/user/.openclaw/skills/searchEverything"
    },
    "status": "success"
  }
}
```

---

## 4. 使用流程设计

### 4.1 OpenClaw 集成

#### 4.1.1 Skill 自动加载

- Skill 安装后自动加载到 OpenClaw
- 命令自动注册到 OpenClaw 命令系统
- 无需手动配置

#### 4.1.2 自然语言触发

通过 `trigger_patterns` 在 `openclaw-skill.yaml` 中定义：

```yaml
commands:
  - name: search
    trigger_patterns:
      - "搜索.*文件"
      - "查找.*"
      - "找.*文件"
      - "search.*files"
      - "find.*"
```

#### 4.1.3 命令别名

```yaml
commands:
  - name: search
    aliases: ["find", "lookup", "查询"]
```

### 4.2 配置管理

#### 4.2.1 配置文件位置

```
~/.openclaw/skills/searchEverything/
├── config.yaml          # 主配置
├── index-config.yaml    # 索引配置
└── ...
```

#### 4.2.2 配置同步

- 支持 OpenClaw 统一配置管理
- 支持 Skill 间共享配置
- 配置变更自动备份

### 4.3 日志集成

#### 4.3.1 日志级别

```yaml
openclaw:
  logging:
    enabled: true
    level: info  # debug, info, warn, error
```

#### 4.3.2 日志输出

- 输出到 OpenClaw 日志系统
- 支持日志轮转
- 最大文件大小限制

#### 4.3.3 审计日志

```yaml
openclaw:
  audit:
    enabled: true
    log_file: "~/.openclaw/skills/searchEverything/logs/audit.log"
    log_commands: true
    log_errors: true
    retention_days: 30
```

### 4.4 更新机制

#### 4.4.1 更新命令

```bash
# 用户触发
searchEverything update

# OpenClaw 触发
openclaw skills update searchEverything
```

#### 4.4.2 更新流程

```
检查版本 → 比较版本 → 下载更新 → 备份旧版 → 安装新版 → 验证 → 清理备份
```

#### 4.4.3 更新模式

| 模式 | 说明 | 命令 |
|------|------|------|
| 交互模式 | 检查后询问用户 | `./scripts/openclaw-update.sh` |
| 静默模式 | 自动更新 | `--silent` |
| 通知模式 | 检查并通知 | `--notify` |
| 仅检查 | 只检查不更新 | `--check` |

---

## 5. 卸载流程设计

### 5.1 卸载流程图

```
                    用户触发卸载
                         │
                         ▼
            ┌────────────────────────┐
            │  OpenClaw 调用钩子      │
            │  on_uninstall.script   │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │ openclaw-uninstall.sh  │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    选择卸载模式         │
            │  (full/keep/disable)   │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    停止相关进程         │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    备份配置（可选）      │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    删除可执行文件       │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │  删除/保留配置和索引    │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    清理 PATH 引用       │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │  从 OpenClaw 移除注册   │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    生成卸载报告         │
            └───────────┬────────────┘
                        │
                        ▼
                   ✅ 卸载完成
```

### 5.2 卸载脚本功能清单

| 步骤 | 功能 | 脚本函数 |
|------|------|----------|
| 1 | 检查安装状态 | `check_installation()` |
| 2 | 显示卸载摘要 | `show_uninstall_summary()` |
| 3 | 确认卸载 | `confirm_uninstall()` |
| 4 | 停止相关进程 | `stop_processes()` |
| 5 | 备份配置 | `backup_config()` |
| 6 | 删除可执行文件 | `remove_binary()` |
| 7 | 处理配置 | `handle_config()` |
| 8 | 处理索引 | `handle_index()` |
| 9 | 处理缓存 | `handle_cache()` |
| 10 | 处理日志 | `handle_logs()` |
| 11 | 清理 PATH | `cleanup_path()` |
| 12 | 从 OpenClaw 注销 | `unregister_from_openclaw()` |
| 13 | 生成卸载报告 | `generate_uninstall_report()` |

### 5.3 卸载模式

#### 5.3.1 完全卸载

```bash
./scripts/openclaw-uninstall.sh --full
```

删除内容：
- ✅ 可执行文件
- ✅ 配置文件
- ✅ 索引数据
- ✅ 缓存文件
- ✅ 日志文件

#### 5.3.2 保留配置卸载

```bash
./scripts/openclaw-uninstall.sh --keep-config
```

删除内容：
- ✅ 可执行文件
- ✅ 缓存文件
- ❌ 保留配置文件
- ❌ 保留索引数据

#### 5.3.3 仅禁用

```bash
./scripts/openclaw-uninstall.sh --disable
```

删除内容：
- ❌ 保留所有数据
- ✅ 仅从 OpenClaw 禁用 Skill

---

## 6. 目录结构设计

### 6.1 OpenClaw 集成目录

```
~/.openclaw/
├── skills/
│   └── searchEverything/
│       ├── config.yaml          # 配置文件
│       ├── index-config.yaml    # 索引配置
│       ├── indexes/             # 索引数据
│       │   └── *.db             # 索引数据库
│       ├── logs/                # 日志文件
│       │   ├── install.log      # 安装日志
│       │   ├── update.log       # 更新日志
│       │   ├── uninstall.log    # 卸载日志
│       │   └── audit.log        # 审计日志
│       └── cache/               # 缓存文件
│           └── *.cache
└── bin/
    └── searchEverything         # 可执行文件
```

### 6.2 备选安装位置

如果 OpenClaw 未指定目录，使用标准位置：

```
~/.local/
├── bin/
│   └── searchEverything
└── share/
    └── searchEverything/
        └── ...
```

### 6.3 配置文件位置

| 配置类型 | 位置 |
|---------|------|
| 主配置 | `~/.openclaw/skills/searchEverything/config.yaml` |
| 索引配置 | `~/.openclaw/skills/searchEverything/index-config.yaml` |
| 审计日志 | `~/.openclaw/skills/searchEverything/logs/audit.log` |
| 缓存 | `~/.openclaw/skills/searchEverything/cache/` |

---

## 7. OpenClaw API 集成

### 7.1 使用 OpenClaw 能力

| OpenClaw 能力 | 用途 |
|--------------|------|
| 下载管理器 | 下载二进制文件 |
| 配置系统 | 统一管理配置 |
| 日志系统 | 输出日志 |
| 更新检查 | 版本检查 |
| 通知系统 | 发送更新通知 |

### 7.2 技能元数据注册

```yaml
openclaw:
  metadata:
    category: "file_management"
    tags: ["search", "file", "index", "utility"]
    icon: "🔍"
    color: "#4A90D9"
```

### 7.3 命令别名注册

```yaml
commands:
  - name: search
    aliases: ["find", "lookup", "查询"]
```

### 7.4 Trigger Patterns 注册

```yaml
commands:
  - name: search
    trigger_patterns:
      - "搜索.*文件"
      - "查找.*"
      - "找.*文件"
```

---

## 8. 错误处理

### 8.1 安装失败处理

#### 8.1.1 错误场景

| 场景 | 处理 |
|------|------|
| 网络错误 | 显示手动安装指南 |
| 下载失败 | 重试 3 次后失败 |
| 权限不足 | 提示使用 sudo |
| 磁盘空间不足 | 检查并提示 |
| 版本不兼容 | 显示最小版本要求 |

#### 8.1.2 回滚机制

```bash
# 安装失败时自动回滚
trap cleanup_on_error ERR

cleanup_on_error() {
    # 删除已安装的文件
    rm -f "${INSTALL_DIR}/searchEverything"
    # 保留配置文件以便调试
}
```

#### 8.1.3 手动安装指南

安装失败时显示详细的手动安装步骤。

### 8.2 运行失败处理

#### 8.2.1 自动诊断

```bash
# 检查常见问题
if ! command -v searchEverything &> /dev/null; then
    echo "错误：searchEverything 未找到"
    echo "建议：检查 PATH 配置"
fi
```

#### 8.2.2 修复建议

每个错误都提供修复建议。

#### 8.2.3 降级选项

```bash
# 支持安装指定版本
searchEverything install --version 0.1.0
```

### 8.3 卸载失败处理

#### 8.3.1 强制卸载选项

```bash
./scripts/openclaw-uninstall.sh --force --full
```

#### 8.3.2 残留清理指南

卸载失败时提供手动清理步骤。

---

## 9. 安全考虑

### 9.1 下载安全

- 仅从 GitHub Releases 下载
- 验证文件完整性（未来支持 SHA256）
- 不执行任何未经验证的代码

### 9.2 权限控制

- 最小权限原则
- 文件系统操作需要用户确认
- 敏感操作（删除）需要二次确认

### 9.3 审计日志

- 所有操作记录到审计日志
- 日志保留 30 天
- 支持日志导出

### 9.4 配置安全

- 配置文件不包含敏感信息
- 配置变更自动备份
- 支持配置验证

---

## 10. 测试策略

### 10.1 安装测试

```bash
# 测试自动安装
./scripts/openclaw-install.sh --dry-run

# 测试离线安装
./scripts/openclaw-install.sh --offline ./searchEverything

# 测试验证
searchEverything --version
```

### 10.2 更新测试

```bash
# 测试更新检查
./scripts/openclaw-update.sh --check

# 测试静默更新
./scripts/openclaw-update.sh --silent --dry-run
```

### 10.3 卸载测试

```bash
# 测试卸载（保留配置）
./scripts/openclaw-uninstall.sh --keep-config --dry-run

# 测试完全卸载
./scripts/openclaw-uninstall.sh --full --dry-run
```

### 10.4 Skill 测试

```bash
# 运行 Skill 测试
searchEverything skill-test
```

测试用例包括：
- 基本搜索
- 索引列表
- 配置显示
- 文件操作

---

## 附录

### A. 脚本参数参考

#### openclaw-install.sh

```
--offline PATH    离线安装模式
--manual          手动安装模式
--verbose, -v     详细输出
--dry-run         模拟运行
--help, -h        显示帮助
```

#### openclaw-update.sh

```
--silent          静默更新
--check           仅检查
--notify          通知模式
--force           强制更新
--verbose, -v     详细输出
--dry-run         模拟运行
--help, -h        显示帮助
```

#### openclaw-uninstall.sh

```
--full            完全卸载
--keep-config     保留配置
--disable         仅禁用
--force, -f       强制卸载
--backup          备份配置
--verbose, -v     详细输出
--dry-run         模拟运行
--help, -h        显示帮助
```

### B. 故障排查

#### 安装失败

1. 检查网络连接
2. 检查磁盘空间
3. 检查权限
4. 查看安装日志

#### 更新失败

1. 检查当前版本
2. 检查最新版本
3. 查看更新日志
4. 尝试手动下载

#### 卸载失败

1. 停止相关进程
2. 手动删除文件
3. 清理 PATH 配置
4. 查看卸载日志

### C. 联系方式

- **作者**: 韩雪原
- **邮箱**: xueyuan_han@163.com
- **GitHub**: https://github.com/hanxueyuan/searchEverything

---

**文档版本**: 1.0  
**最后更新**: 2026-03-12
