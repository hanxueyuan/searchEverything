#!/bin/bash
#===============================================================================
# searchEverything OpenClaw 更新脚本
# 功能：检查并更新到最新版本，支持静默更新和通知更新
# GitHub: https://github.com/hanxueyuan/searchEverything
#
# 更新模式:
#   - 交互模式：检查后询问用户是否更新
#   - 静默模式：自动更新，无提示
#   - 通知模式：检查并通知，等待用户确认
#
# 使用方法:
#   ./scripts/openclaw-update.sh              # 交互更新
#   ./scripts/openclaw-update.sh --silent     # 静默更新
#   ./scripts/openclaw-update.sh --check      # 仅检查，不更新
#   ./scripts/openclaw-update.sh --force      # 强制更新
#   ./scripts/openclaw-update.sh --verbose    # 详细输出
#   ./scripts/openclaw-update.sh --help       # 显示帮助
#===============================================================================

set -euo pipefail

#===============================================================================
# 配置常量
#===============================================================================
readonly REPO_OWNER="hanxueyuan"
readonly REPO_NAME="searchEverything"
readonly GITHUB_API_BASE="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"
readonly GITHUB_RELEASES_BASE="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"

# 安装目录
if [ -n "${OPENCLAW_HOME:-}" ] && [ -d "${OPENCLAW_HOME}" ]; then
    INSTALL_DIR="${OPENCLAW_HOME}/bin"
    OPENCLAW_MODE="true"
else
    INSTALL_DIR="${HOME}/.local/bin"
    OPENCLAW_MODE="false"
fi

# OpenClaw Skill 目录
OPENCLAW_SKILL_DIR="${HOME}/.openclaw/skills/searchEverything"
LOG_DIR="${OPENCLAW_SKILL_DIR}/logs"

# 最小版本要求
readonly MIN_VERSION="0.2.0"

# 更新模式
UPDATE_MODE="interactive"  # interactive, silent, notify, check_only
FORCE="false"
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
readonly NC='\033[0m'
readonly BOLD='\033[1m'

#===============================================================================
# 日志函数
#===============================================================================
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        INFO)  echo -e "${BLUE}[${timestamp}] [INFO]${NC} $message" ;;
        SUCCESS) echo -e "${GREEN}[${timestamp}] [SUCCESS]${NC} $message" ;;
        WARNING) echo -e "${YELLOW}[${timestamp}] [WARNING]${NC} $message" ;;
        ERROR) echo -e "${RED}[${timestamp}] [ERROR]${NC} $message" ;;
        DEBUG) [ "$VERBOSE" = "true" ] && echo -e "${CYAN}[${timestamp}] [DEBUG]${NC} $message" ;;
    esac
    
    if [ -d "$LOG_DIR" ]; then
        echo "[${timestamp}] [${level}] $message" >> "${LOG_DIR}/update.log" 2>/dev/null || true
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
            --silent)
                UPDATE_MODE="silent"
                shift
                ;;
            --check)
                UPDATE_MODE="check_only"
                shift
                ;;
            --notify)
                UPDATE_MODE="notify"
                shift
                ;;
            --force)
                FORCE="true"
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
${BOLD}searchEverything OpenClaw 更新脚本${NC}

用法：$0 [选项]

选项:
  --silent          静默更新，自动下载并安装，无提示
  --check           仅检查更新，不执行更新
  --notify          通知模式，检查并通知，等待用户确认
  --force           强制更新，即使已是最新版本
  --verbose, -v     显示详细输出
  --dry-run         模拟运行，不执行实际更新
  --help, -h        显示此帮助信息

更新模式:
  交互模式 (默认):  检查后询问用户是否更新
  静默模式：        自动更新，适合脚本调用
  通知模式：        检查并显示通知，等待确认
  仅检查：          只检查版本，不执行更新

示例:
  $0                    # 交互更新
  $0 --silent           # 静默更新
  $0 --check            # 仅检查更新
  $0 --force --silent   # 强制静默更新

EOF
}

#===============================================================================
# 获取当前版本
#===============================================================================
get_current_version() {
    if [ -x "${INSTALL_DIR}/searchEverything" ]; then
        local version=$("${INSTALL_DIR}/searchEverything" --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        if [ -n "$version" ]; then
            echo "$version"
            return 0
        fi
    fi
    echo "unknown"
}

#===============================================================================
# 获取最新版本
#===============================================================================
get_latest_version() {
    # 尝试通过 GitHub API 获取最新版本
    if command -v curl &> /dev/null; then
        local version=$(curl -s "${GITHUB_API_BASE}/releases/latest" 2>/dev/null | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
        if [ -n "$version" ]; then
            echo "$version"
            return 0
        fi
    fi
    
    # 如果 API 失败，尝试从 HTML 页面获取
    if command -v curl &> /dev/null; then
        local version=$(curl -sL "${GITHUB_RELEASES_BASE}/latest" 2>/dev/null | grep -oE '/releases/tag/v[0-9]+\.[0-9]+\.[0-9]+' | head -1 | sed 's/\/releases\/tag\///')
        if [ -n "$version" ]; then
            echo "$version"
            return 0
        fi
    fi
    
    echo "unknown"
}

#===============================================================================
# 比较版本
#===============================================================================
version_compare() {
    local v1="$1"
    local v2="$2"
    
    if [ "$v1" = "$v2" ]; then
        echo "equal"
        return
    fi
    
    # 使用 sort -V 比较版本
    if [ "$(printf '%s\n' "$v1" "$v2" | sort -V | head -n1)" = "$v1" ]; then
        echo "older"
    else
        echo "newer"
    fi
}

#===============================================================================
# 检测平台和构建下载信息
#===============================================================================
detect_platform_and_build_url() {
    local version="$1"
    
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    # 规范化
    case "$OS" in
        linux) OS="linux" ;;
        darwin) OS="macos" ;;
        mingw*|msys*|cygwin*) OS="windows" ;;
    esac
    
    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        armv7l|armhf) ARCH="armv7" ;;
    esac
    
    if [ "$OS" = "windows" ]; then
        EXT=".exe"
    else
        EXT=""
    fi
    
    BINARY_NAME="searchEverything-${ARCH}-${OS}${EXT}"
    
    if [ "$version" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/${BINARY_NAME}"
    else
        DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${BINARY_NAME}"
    fi
    
    debug "平台：${ARCH}-${OS}"
    debug "下载 URL: $DOWNLOAD_URL"
}

#===============================================================================
# 下载并安装更新
#===============================================================================
download_update() {
    local version="$1"
    
    info "从 $DOWNLOAD_URL 下载..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将下载：$DOWNLOAD_URL"
        return 0
    fi
    
    local temp_file=$(mktemp)
    
    if curl -L -# --fail -o "$temp_file" "$DOWNLOAD_URL" 2>&1; then
        success "下载完成"
        
        # 备份旧版本
        if [ -f "${INSTALL_DIR}/searchEverything" ]; then
            cp "${INSTALL_DIR}/searchEverything" "${INSTALL_DIR}/searchEverything.bak"
            info "已备份旧版本"
        elif [ -f "${INSTALL_DIR}/searchEverything.exe" ]; then
            cp "${INSTALL_DIR}/searchEverything.exe" "${INSTALL_DIR}/searchEverything.exe.bak"
            info "已备份旧版本"
        fi
        
        # 替换新版本
        mv "$temp_file" "${INSTALL_DIR}/searchEverything${EXT}"
        chmod +x "${INSTALL_DIR}/searchEverything${EXT}" 2>/dev/null || true
        
        success "更新完成"
        return 0
    else
        rm -f "$temp_file"
        error "下载失败"
        return 1
    fi
}

#===============================================================================
# 验证更新
#===============================================================================
verify_update() {
    info "验证更新..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将验证更新"
        return 0
    fi
    
    if "${INSTALL_DIR}/searchEverything" --version > /dev/null 2>&1; then
        local version=$("${INSTALL_DIR}/searchEverything" --version 2>&1)
        success "更新验证成功！$version"
        
        # 删除备份
        rm -f "${INSTALL_DIR}/searchEverything.bak" 2>/dev/null || true
        rm -f "${INSTALL_DIR}/searchEverything.exe.bak" 2>/dev/null || true
        
        return 0
    else
        error "更新后验证失败"
        return 1
    fi
}

#===============================================================================
# 回滚到旧版本
#===============================================================================
rollback() {
    warning "回滚到旧版本..."
    
    if [ -f "${INSTALL_DIR}/searchEverything.bak" ]; then
        mv "${INSTALL_DIR}/searchEverything.bak" "${INSTALL_DIR}/searchEverything"
        success "已回滚到旧版本"
    elif [ -f "${INSTALL_DIR}/searchEverything.exe.bak" ]; then
        mv "${INSTALL_DIR}/searchEverything.exe.bak" "${INSTALL_DIR}/searchEverything.exe"
        success "已回滚到旧版本"
    else
        error "未找到备份文件，无法回滚"
    fi
}

#===============================================================================
# 通知用户（OpenClaw 集成）
#===============================================================================
notify_user() {
    local current="$1"
    local latest="$2"
    
    echo ""
    echo "=========================================="
    echo "${BOLD}更新通知${NC}"
    echo "=========================================="
    echo ""
    echo "发现新版本：${GREEN}$latest${NC}"
    echo "当前版本：${YELLOW}$current${NC}"
    echo ""
    
    # 如果 OpenClaw 可用，尝试发送通知
    if command -v openclaw &> /dev/null; then
        if openclaw notify --title "searchEverything 更新可用" --body "新版本 $latest 已发布，当前版本 $current" 2>/dev/null; then
            debug "已发送 OpenClaw 通知"
        fi
    fi
}

#===============================================================================
# 检查更新
#===============================================================================
check_for_updates() {
    info "检查更新..."
    
    CURRENT=$(get_current_version)
    LATEST=$(get_latest_version)
    
    info "当前版本：$CURRENT"
    info "最新版本：$LATEST"
    
    if [ "$CURRENT" = "unknown" ]; then
        error "未检测到已安装的 searchEverything"
        echo ""
        echo "请先运行安装脚本：scripts/openclaw-install.sh"
        exit 1
    fi
    
    if [ "$LATEST" = "unknown" ]; then
        warning "无法获取最新版本信息"
        echo ""
        echo "请检查网络连接或访问：https://github.com/hanxueyuan/searchEverything/releases"
        exit 1
    fi
    
    COMPARISON=$(version_compare "$CURRENT" "$LATEST")
    
    case "$COMPARISON" in
        equal)
            success "已是最新版本，无需更新"
            if [ "$FORCE" = "true" ]; then
                info "强制模式，将重新安装当前版本"
            else
                exit 0
            fi
            ;;
        newer)
            warning "当前版本比最新 release 更新 (可能是开发版本)"
            if [ "$FORCE" != "true" ]; then
                exit 0
            fi
            ;;
        older)
            info "发现新版本：$LATEST (当前：$CURRENT)"
            ;;
    esac
    
    # 保存版本信息供后续使用
    export CURRENT_VERSION="$CURRENT"
    export LATEST_VERSION="$LATEST"
}

#===============================================================================
# 执行更新流程
#===============================================================================
perform_update() {
    case "$UPDATE_MODE" in
        check_only)
            # 仅检查，已在 check_for_updates 中处理
            ;;
        silent)
            info "静默更新模式..."
            detect_platform_and_build_url "latest"
            download_update "latest"
            if ! verify_update; then
                rollback
                exit 1
            fi
            ;;
        notify)
            notify_user "$CURRENT_VERSION" "$LATEST_VERSION"
            echo ""
            read -p "是否更新到最新版本？[y/N] " -n 1 -r
            echo ""
            
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                detect_platform_and_build_url "latest"
                download_update "latest"
                if ! verify_update; then
                    rollback
                    exit 1
                fi
            else
                info "取消更新"
                exit 0
            fi
            ;;
        interactive|*)
            echo ""
            read -p "是否更新到最新版本？[y/N] " -n 1 -r
            echo ""
            
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                detect_platform_and_build_url "latest"
                download_update "latest"
                if ! verify_update; then
                    rollback
                    exit 1
                fi
            else
                info "取消更新"
                exit 0
            fi
            ;;
    esac
}

#===============================================================================
# 生成更新报告
#===============================================================================
generate_update_report() {
    local report_file="${LOG_DIR}/update-report-$(date +%Y%m%d-%H%M%S).json"
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将生成更新报告：$report_file"
        return 0
    fi
    
    mkdir -p "$LOG_DIR"
    
    cat > "$report_file" << EOF
{
  "update": {
    "timestamp": "$(date -Iseconds)",
    "mode": "$UPDATE_MODE",
    "versions": {
      "from": "$CURRENT_VERSION",
      "to": "$LATEST_VERSION"
    },
    "platform": {
      "os": "$OS",
      "arch": "$ARCH"
    },
    "status": "success"
  }
}
EOF
    
    debug "更新报告已生成：$report_file"
}

#===============================================================================
# 显示完成信息
#===============================================================================
show_completion_message() {
    if [ "$UPDATE_MODE" = "check_only" ]; then
        return 0
    fi
    
    echo ""
    echo "=========================================="
    success "${BOLD}searchEverything 更新完成！${NC}"
    echo "=========================================="
    echo ""
    
    "${INSTALL_DIR}/searchEverything" --version 2>/dev/null || true
    
    echo ""
    echo "更新日志：https://github.com/hanxueyuan/searchEverything/releases"
    echo ""
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    echo ""
    echo "=========================================="
    echo "${BOLD}  searchEverything OpenClaw 更新检查${NC}"
    echo "  GitHub: https://github.com/hanxueyuan/searchEverything"
    echo "=========================================="
    
    # 解析参数
    parse_args "$@"
    
    # 检查更新
    check_for_updates
    
    # 执行更新
    perform_update
    
    # 生成报告
    generate_update_report
    
    # 显示完成信息
    show_completion_message
}

# 运行主函数
main "$@"
