#!/bin/bash
#===============================================================================
# searchEverything OpenClaw 安装脚本
# 功能：深度集成 OpenClaw，自动下载并安装 searchEverything 可执行文件
# GitHub: https://github.com/hanxueyuan/searchEverything
# 
# 安装模式:
#   - 自动模式：OpenClaw Skill 安装时自动触发
#   - 手动模式：用户手动运行此脚本
#   - 离线模式：使用预下载的二进制文件
#
# 使用方法:
#   ./scripts/openclaw-install.sh                    # 自动安装
#   ./scripts/openclaw-install.sh --offline PATH     # 离线安装
#   ./scripts/openclaw-install.sh --verbose          # 详细输出
#   ./scripts/openclaw-install.sh --help             # 显示帮助
#===============================================================================

set -euo pipefail

#===============================================================================
# 配置常量
#===============================================================================
readonly REPO_OWNER="hanxueyuan"
readonly REPO_NAME="searchEverything"
readonly GITHUB_API_BASE="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"
readonly GITHUB_RELEASES_BASE="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"

# 安装目录优先级：OpenClaw 目录 > ~/.local/bin
if [ -n "${OPENCLAW_HOME:-}" ] && [ -d "${OPENCLAW_HOME}" ]; then
    INSTALL_DIR="${OPENCLAW_HOME}/bin"
    OPENCLAW_MODE="true"
else
    INSTALL_DIR="${HOME}/.local/bin"
    OPENCLAW_MODE="false"
fi

# OpenClaw Skill 目录
OPENCLAW_SKILL_DIR="${HOME}/.openclaw/skills/searchEverything"
CONFIG_DIR="${OPENCLAW_SKILL_DIR}"
LOG_DIR="${OPENCLAW_SKILL_DIR}/logs"
INDEX_DIR="${OPENCLAW_SKILL_DIR}/indexes"
CACHE_DIR="${OPENCLAW_SKILL_DIR}/cache"

# 最小版本要求
readonly MIN_VERSION="0.2.0"

# 脚本模式
INSTALL_MODE="auto"  # auto, manual, offline
OFFLINE_PATH=""
VERBOSE="false"
DRY_RUN="false"

#===============================================================================
# 颜色输出
#===============================================================================
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly MAGENTA='\033[0;35m'
readonly NC='\033[0m'  # No Color
readonly BOLD='\033[1m'

#===============================================================================
# 日志函数
#===============================================================================
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # 输出到控制台
    case "$level" in
        INFO)  echo -e "${BLUE}[${timestamp}] [INFO]${NC} $message" ;;
        SUCCESS) echo -e "${GREEN}[${timestamp}] [SUCCESS]${NC} $message" ;;
        WARNING) echo -e "${YELLOW}[${timestamp}] [WARNING]${NC} $message" ;;
        ERROR) echo -e "${RED}[${timestamp}] [ERROR]${NC} $message" ;;
        DEBUG) [ "$VERBOSE" = "true" ] && echo -e "${CYAN}[${timestamp}] [DEBUG]${NC} $message" ;;
    esac
    
    # 输出到日志文件（如果存在）
    if [ -d "$LOG_DIR" ]; then
        echo "[${timestamp}] [${level}] $message" >> "${LOG_DIR}/install.log" 2>/dev/null || true
    fi
}

info() { log "INFO" "$@"; }
success() { log "SUCCESS" "$@"; }
warning() { log "WARNING" "$@"; }
error() { log "ERROR" "$@"; }
debug() { log "DEBUG" "$@"; }

#===============================================================================
# 解析命令行参数
#===============================================================================
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --offline)
                INSTALL_MODE="offline"
                OFFLINE_PATH="${2:-}"
                shift 2 || { error "--offline 需要指定文件路径"; exit 1; }
                ;;
            --manual)
                INSTALL_MODE="manual"
                shift
                ;;
            --verbose|-v)
                VERBOSE="true"
                shift
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                error "未知参数：$1"
                show_help
                exit 1
                ;;
        esac
    done
}

#===============================================================================
# 显示帮助信息
#===============================================================================
show_help() {
    cat << EOF
${BOLD}searchEverything OpenClaw 安装脚本${NC}

用法：$0 [选项]

选项:
  --offline PATH    离线安装模式，使用预下载的二进制文件
  --manual          手动安装模式（非自动）
  --verbose, -v     显示详细输出
  --dry-run         模拟运行，不执行实际安装
  --help, -h        显示此帮助信息

安装模式:
  自动模式 (默认):  OpenClaw Skill 安装时自动触发
  手动模式：        用户手动运行此脚本
  离线模式：        使用预下载的二进制文件，无需网络

示例:
  $0                              # 自动安装
  $0 --offline ./searchEverything # 离线安装
  $0 --verbose                    # 详细输出
  $0 --dry-run                    # 模拟运行

EOF
}

#===============================================================================
# 检测操作系统和架构
#===============================================================================
detect_platform() {
    info "检测当前平台..."
    
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    # 规范化操作系统名称
    case "$OS" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        mingw*|msys*|cygwin*)
            OS="windows"
            ;;
        *)
            error "不支持的操作系统：$OS"
            exit 1
            ;;
    esac
    
    # 规范化架构名称
    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        armv7l|armhf)
            ARCH="armv7"
            ;;
        i386|i686)
            ARCH="i686"
            ;;
        *)
            error "不支持的架构：$ARCH"
            exit 1
            ;;
    esac
    
    # 确定文件扩展名
    if [ "$OS" = "windows" ]; then
        EXT=".exe"
    else
        EXT=""
    fi
    
    # 构建下载文件名
    BINARY_NAME="searchEverything-${ARCH}-${OS}${EXT}"
    
    success "检测到平台：${ARCH}-${OS}"
    debug "目标文件：$BINARY_NAME"
}

#===============================================================================
# 检查 OpenClaw 环境
#===============================================================================
check_openclaw() {
    info "检查 OpenClaw 环境..."
    
    if [ "$OPENCLAW_MODE" = "true" ]; then
        success "OpenClaw 环境已检测到"
        debug "OPENCLAW_HOME: $OPENCLAW_HOME"
    else
        debug "OpenClaw 未检测到，使用默认安装目录"
    fi
    
    # 检查 OpenClaw 命令
    if command -v openclaw &> /dev/null; then
        success "OpenClaw 命令可用"
    else
        warning "OpenClaw 命令未找到，部分功能可能不可用"
    fi
}

#===============================================================================
# 获取最新版本号
#===============================================================================
get_latest_version() {
    info "获取最新版本信息..."
    
    # 尝试通过 GitHub API 获取最新版本
    if command -v curl &> /dev/null; then
        LATEST_VERSION=$(curl -s "${GITHUB_API_BASE}/releases/latest" 2>/dev/null | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
        
        if [ -n "$LATEST_VERSION" ]; then
            success "最新版本：$LATEST_VERSION"
            return 0
        fi
    fi
    
    # 如果 API 失败，尝试从 HTML 页面获取
    if command -v curl &> /dev/null; then
        LATEST_VERSION=$(curl -sL "${GITHUB_RELEASES_BASE}/latest" 2>/dev/null | grep -oE '/releases/tag/v[0-9]+\.[0-9]+\.[0-9]+' | head -1 | sed 's/\/releases\/tag\///' || echo "")
        
        if [ -n "$LATEST_VERSION" ]; then
            success "最新版本：$LATEST_VERSION"
            return 0
        fi
    fi
    
    warning "无法获取版本信息，将下载最新 release"
    LATEST_VERSION="latest"
    return 0
}

#===============================================================================
# 构建下载 URL
#===============================================================================
build_download_url() {
    if [ "$LATEST_VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/${BINARY_NAME}"
    else
        DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${LATEST_VERSION}/${BINARY_NAME}"
    fi
    
    debug "下载 URL: $DOWNLOAD_URL"
}

#===============================================================================
# 检查依赖
#===============================================================================
check_dependencies() {
    info "检查依赖工具..."
    
    local missing=()
    
    if ! command -v curl &> /dev/null; then
        missing+=("curl")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        error "缺少必要工具：${missing[*]}"
        echo ""
        echo "请安装缺失的工具:"
        echo "  Linux:   sudo apt install curl"
        echo "  macOS:   brew install curl"
        echo "  Windows: choco install curl"
        exit 1
    fi
    
    success "依赖检查通过"
}

#===============================================================================
# 创建安装目录
#===============================================================================
setup_install_dirs() {
    info "准备安装目录..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将创建目录：$INSTALL_DIR"
        debug "[DRY-RUN] 将创建目录：$CONFIG_DIR"
        debug "[DRY-RUN] 将创建目录：$LOG_DIR"
        debug "[DRY-RUN] 将创建目录：$INDEX_DIR"
        debug "[DRY-RUN] 将创建目录：$CACHE_DIR"
        return 0
    fi
    
    # 创建安装目录
    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR"
        info "创建安装目录：$INSTALL_DIR"
    fi
    
    # 创建 OpenClaw Skill 目录结构
    for dir in "$CONFIG_DIR" "$LOG_DIR" "$INDEX_DIR" "$CACHE_DIR"; do
        if [ ! -d "$dir" ]; then
            mkdir -p "$dir"
            debug "创建目录：$dir"
        fi
    done
    
    success "目录准备完成"
}

#===============================================================================
# 下载二进制文件
#===============================================================================
download_binary() {
    if [ "$INSTALL_MODE" = "offline" ]; then
        info "使用离线文件：$OFFLINE_PATH"
        
        if [ ! -f "$OFFLINE_PATH" ]; then
            error "离线文件不存在：$OFFLINE_PATH"
            exit 1
        fi
        
        if [ "$DRY_RUN" = "true" ]; then
            debug "[DRY-RUN] 将复制文件：$OFFLINE_PATH -> ${INSTALL_DIR}/searchEverything${EXT}"
            return 0
        fi
        
        cp "$OFFLINE_PATH" "${INSTALL_DIR}/searchEverything${EXT}"
        success "文件复制完成"
        return 0
    fi
    
    local temp_file=$(mktemp)
    
    info "下载二进制文件..."
    info "从 $DOWNLOAD_URL 下载"
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将下载：$DOWNLOAD_URL"
        rm -f "$temp_file"
        return 0
    fi
    
    # 下载文件
    if curl -L -# --fail -o "$temp_file" "$DOWNLOAD_URL" 2>&1; then
        success "下载完成"
        mv "$temp_file" "${INSTALL_DIR}/searchEverything${EXT}"
    else
        local exit_code=$?
        rm -f "$temp_file"
        
        error "下载失败 (错误代码：$exit_code)"
        show_manual_instructions
        exit 1
    fi
}

#===============================================================================
# 显示手动安装说明
#===============================================================================
show_manual_instructions() {
    echo ""
    echo "=========================================="
    echo "${BOLD}手动安装说明${NC}"
    echo "=========================================="
    echo ""
    echo "1. 访问 GitHub Releases 页面:"
    echo "   ${GITHUB_RELEASES_BASE}/latest"
    echo ""
    echo "2. 下载适合您平台的文件:"
    echo "   - Linux x86_64:    searchEverything-x86_64-linux"
    echo "   - Linux aarch64:   searchEverything-aarch64-linux"
    echo "   - macOS x86_64:    searchEverything-x86_64-macos"
    echo "   - macOS aarch64:   searchEverything-aarch64-macos"
    echo "   - Windows x86_64:  searchEverything-x86_64-windows.exe"
    echo "   - Windows aarch64: searchEverything-aarch64-windows.exe"
    echo ""
    echo "3. 将下载的文件移动到安装目录并添加执行权限:"
    if [ "$OS" != "windows" ]; then
        echo "   chmod +x searchEverything"
        echo "   mv searchEverything ${INSTALL_DIR}/"
    else
        echo "   将 searchEverything.exe 移动到 ${INSTALL_DIR}\\"
    fi
    echo ""
    echo "4. 验证安装:"
    echo "   searchEverything --version"
    echo ""
    echo "=========================================="
}

#===============================================================================
# 设置执行权限
#===============================================================================
set_permissions() {
    if [ "$OS" != "windows" ]; then
        info "设置执行权限..."
        
        if [ "$DRY_RUN" = "true" ]; then
            debug "[DRY-RUN] 将设置权限：chmod +x ${INSTALL_DIR}/searchEverything"
            return 0
        fi
        
        chmod +x "${INSTALL_DIR}/searchEverything"
        success "权限设置完成"
    fi
}

#===============================================================================
# 验证安装
#===============================================================================
verify_installation() {
    info "验证安装..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将验证安装"
        return 0
    fi
    
    # 尝试运行版本命令
    if "${INSTALL_DIR}/searchEverything${EXT}" --version > /dev/null 2>&1; then
        local version=$("${INSTALL_DIR}/searchEverything${EXT}" --version 2>&1)
        success "安装验证成功！$version"
        
        # 检查版本是否满足最小要求
        local installed_version=$(echo "$version" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        if [ -n "$installed_version" ]; then
            if version_lt "$installed_version" "$MIN_VERSION"; then
                warning "安装版本 ($installed_version) 低于最小要求 ($MIN_VERSION)"
            else
                debug "版本检查通过：$installed_version >= $MIN_VERSION"
            fi
        fi
        
        return 0
    else
        warning "验证失败，但文件已安装"
        return 1
    fi
}

#===============================================================================
# 版本比较 (返回 0 如果 v1 < v2)
#===============================================================================
version_lt() {
    local v1="$1"
    local v2="$2"
    
    if [ "$v1" = "$v2" ]; then
        return 1
    fi
    
    if [ "$(printf '%s\n' "$v1" "$v2" | sort -V | head -n1)" = "$v1" ]; then
        return 0
    else
        return 1
    fi
}

#===============================================================================
# 检查 PATH 配置
#===============================================================================
check_path() {
    info "检查 PATH 配置..."
    
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        success "安装目录已在 PATH 中"
        return 0
    fi
    
    warning "安装目录不在 PATH 中"
    echo ""
    echo "=========================================="
    echo "${BOLD}PATH 配置说明${NC}"
    echo "=========================================="
    echo ""
    echo "请将以下行添加到您的 shell 配置文件中:"
    echo ""
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
    
    # 尝试自动添加到配置文件
    auto_add_to_path
    
    echo "=========================================="
}

#===============================================================================
# 自动添加到 PATH
#===============================================================================
auto_add_to_path() {
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将添加到 shell 配置文件"
        return 0
    fi
    
    local profile_added=false
    local shell_config=""
    
    # 检测 shell
    case "$SHELL" in
        */bash*)
            shell_config="$HOME/.bashrc"
            ;;
        */zsh*)
            shell_config="$HOME/.zshrc"
            ;;
        */fish*)
            shell_config="$HOME/.config/fish/config.fish"
            ;;
    esac
    
    if [ -n "$shell_config" ] && [ -f "$shell_config" ]; then
        if ! grep -q "$INSTALL_DIR" "$shell_config" 2>/dev/null; then
            echo "" >> "$shell_config"
            echo "# searchEverything (added by openclaw-install.sh on $(date))" >> "$shell_config"
            echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$shell_config"
            info "已添加到 $shell_config"
            profile_added=true
        fi
    fi
    
    if [ "$profile_added" = "true" ]; then
        echo ""
        info "请运行 'source $shell_config' 或重新打开终端使配置生效"
    fi
}

#===============================================================================
# 创建默认配置
#===============================================================================
create_default_config() {
    info "创建默认配置文件..."
    
    local config_file="${CONFIG_DIR}/config.yaml"
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将创建配置文件：$config_file"
        return 0
    fi
    
    if [ ! -f "$config_file" ]; then
        cat > "$config_file" << 'EOF'
# searchEverything 配置文件
# 由 OpenClaw 安装脚本自动生成
# 最后更新：TIMESTAMP_PLACEHOLDER

search:
  default_mode: glob        # glob, regex, fuzzy
  default_limit: 100
  case_sensitive: false

output:
  default_format: json      # json, text
  pretty_print: false
  stream: false

index:
  auto_index: true
  exclude_paths:
    - "/proc"
    - "/sys"
    - "/dev"
    - "/run"

performance:
  threads: 4
  memory_limit_mb: 512

openclaw:
  enabled: true
  skill_name: searchEverything
  logging:
    enabled: true
    level: info
  audit:
    enabled: true

audit:
  enabled: true
  log_file: "~/.openclaw/skills/searchEverything/logs/audit.log"
EOF
        # 替换时间戳
        sed -i "s/TIMESTAMP_PLACEHOLDER/$(date '+%Y-%m-%d %H:%M:%S')/" "$config_file" 2>/dev/null || true
        
        success "默认配置已创建：$config_file"
    else
        info "配置文件已存在，跳过创建"
    fi
}

#===============================================================================
# 注册到 OpenClaw
#===============================================================================
register_with_openclaw() {
    info "注册到 OpenClaw..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将注册到 OpenClaw"
        return 0
    fi
    
    # 如果 OpenClaw 可用，尝试注册
    if command -v openclaw &> /dev/null; then
        # 刷新技能列表
        if openclaw skills refresh 2>/dev/null; then
            success "已刷新 OpenClaw 技能列表"
        else
            debug "OpenClaw 技能刷新跳过"
        fi
    else
        debug "OpenClaw 命令不可用，跳过注册"
    fi
}

#===============================================================================
# 生成安装报告
#===============================================================================
generate_install_report() {
    local report_file="${LOG_DIR}/install-report-$(date +%Y%m%d-%H%M%S).json"
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将生成安装报告：$report_file"
        return 0
    fi
    
    cat > "$report_file" << EOF
{
  "installation": {
    "timestamp": "$(date -Iseconds)",
    "mode": "$INSTALL_MODE",
    "version": "$LATEST_VERSION",
    "platform": {
      "os": "$OS",
      "arch": "$ARCH"
    },
    "paths": {
      "install_dir": "$INSTALL_DIR",
      "config_dir": "$CONFIG_DIR",
      "log_dir": "$LOG_DIR",
      "binary": "${INSTALL_DIR}/searchEverything${EXT}"
    },
    "openclaw": {
      "enabled": $OPENCLAW_MODE,
      "skill_dir": "$OPENCLAW_SKILL_DIR"
    },
    "status": "success"
  }
}
EOF
    
    debug "安装报告已生成：$report_file"
}

#===============================================================================
# 显示完成信息
#===============================================================================
show_completion_message() {
    echo ""
    echo "=========================================="
    success "${BOLD}searchEverything 安装完成！${NC}"
    echo "=========================================="
    echo ""
    echo "${BOLD}快速开始:${NC}"
    echo "  searchEverything search \"*.pdf\"     # 搜索 PDF 文件"
    echo "  searchEverything index list           # 查看索引"
    echo "  searchEverything config show          # 查看配置"
    echo "  searchEverything --help               # 显示帮助"
    echo ""
    echo "${BOLD}OpenClaw 自然语言触发:${NC}"
    echo "  \"搜索 PDF 文件\"                  → 自动调用 search 命令"
    echo "  \"查看这个文件的信息\"           → 自动调用 info 命令"
    echo "  \"读取配置文件\"                 → 自动调用 cat 命令"
    echo "  \"添加工作目录到索引\"           → 自动调用 index_add 命令"
    echo ""
    echo "${BOLD}配置和日志:${NC}"
    echo "  配置文件：$CONFIG_DIR/config.yaml"
    echo "  日志目录：$LOG_DIR/"
    echo "  索引数据：$INDEX_DIR/"
    echo ""
    echo "${BOLD}文档：${NC}https://github.com/hanxueyuan/searchEverything"
    echo ""
}

#===============================================================================
# 错误处理和回滚
#===============================================================================
cleanup_on_error() {
    warning "安装失败，执行回滚..."
    
    # 删除可能创建的文件
    if [ -f "${INSTALL_DIR}/searchEverything${EXT}" ]; then
        rm -f "${INSTALL_DIR}/searchEverything${EXT}"
        info "已删除可执行文件"
    fi
    
    # 保留配置文件以便调试
    warning "配置文件保留在：$CONFIG_DIR"
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    echo ""
    echo "=========================================="
    echo "${BOLD}  searchEverything OpenClaw 安装脚本${NC}"
    echo "  GitHub: https://github.com/hanxueyuan/searchEverything"
    echo "  版本：$MIN_VERSION+"
    echo "=========================================="
    echo ""
    
    # 解析参数
    parse_args "$@"
    
    # 设置错误处理
    trap cleanup_on_error ERR
    
    # 执行安装步骤
    detect_platform
    check_openclaw
    check_dependencies
    
    if [ "$INSTALL_MODE" != "offline" ]; then
        get_latest_version
        build_download_url
    fi
    
    setup_install_dirs
    download_binary
    set_permissions
    
    if ! verify_installation; then
        warning "安装验证失败"
    fi
    
    check_path
    create_default_config
    register_with_openclaw
    generate_install_report
    
    show_completion_message
    
    # 清除错误陷阱
    trap - ERR
}

# 运行主函数
main "$@"
