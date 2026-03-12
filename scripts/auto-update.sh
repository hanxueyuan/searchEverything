#!/bin/bash
#===============================================================================
# searchEverything 自动更新脚本
# 功能：检查并更新到最新版本
# GitHub: https://github.com/hanxueyuan/searchEverything
#===============================================================================

set -e

# 配置常量
REPO_OWNER="hanxueyuan"
REPO_NAME="searchEverything"
GITHUB_API_BASE="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"
INSTALL_DIR="${HOME}/.local/bin"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

#-------------------------------------------------------------------------------
# 获取当前版本
#-------------------------------------------------------------------------------
get_current_version() {
    if [ -x "${INSTALL_DIR}/searchEverything" ]; then
        CURRENT_VERSION=$("${INSTALL_DIR}/searchEverything" --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        if [ -n "$CURRENT_VERSION" ]; then
            echo "$CURRENT_VERSION"
            return 0
        fi
    fi
    echo "unknown"
}

#-------------------------------------------------------------------------------
# 获取最新版本
#-------------------------------------------------------------------------------
get_latest_version() {
    local version=$(curl -s "${GITHUB_API_BASE}/releases/latest" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -n "$version" ]; then
        echo "$version"
        return 0
    fi
    echo "unknown"
}

#-------------------------------------------------------------------------------
# 比较版本
#-------------------------------------------------------------------------------
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

#-------------------------------------------------------------------------------
# 下载并安装更新
#-------------------------------------------------------------------------------
download_update() {
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
    
    info "从 $DOWNLOAD_URL 下载..."
    
    local temp_file=$(mktemp)
    
    if curl -L -# --fail -o "$temp_file" "$DOWNLOAD_URL" 2>&1; then
        success "下载完成"
        
        # 备份旧版本
        if [ -f "${INSTALL_DIR}/searchEverything${EXT}" ]; then
            cp "${INSTALL_DIR}/searchEverything${EXT}" "${INSTALL_DIR}/searchEverything${EXT}.bak"
            info "已备份旧版本"
        fi
        
        # 替换新版本
        mv "$temp_file" "${INSTALL_DIR}/searchEverything${EXT}"
        chmod +x "${INSTALL_DIR}/searchEverything${EXT}"
        
        success "更新完成"
        return 0
    else
        rm -f "$temp_file"
        error "下载失败"
        return 1
    fi
}

#-------------------------------------------------------------------------------
# 主函数
#-------------------------------------------------------------------------------
main() {
    echo ""
    echo "=========================================="
    echo "  searchEverything 更新检查"
    echo "=========================================="
    echo ""
    
    CURRENT=$(get_current_version)
    LATEST=$(get_latest_version)
    
    info "当前版本：$CURRENT"
    info "最新版本：$LATEST"
    
    if [ "$CURRENT" = "unknown" ]; then
        error "未检测到已安装的 searchEverything"
        echo ""
        echo "请先运行安装脚本：scripts/auto-install.sh"
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
            exit 0
            ;;
        newer)
            warning "当前版本比最新 release 更新 (可能是开发版本)"
            exit 0
            ;;
        older)
            info "发现新版本：$LATEST (当前：$CURRENT)"
            echo ""
            read -p "是否更新到最新版本？[y/N] " -n 1 -r
            echo ""
            
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                download_update "latest"
                
                # 验证更新
                if "${INSTALL_DIR}/searchEverything" --version > /dev/null 2>&1; then
                    success "更新成功！"
                    rm -f "${INSTALL_DIR}/searchEverything.bak" "${INSTALL_DIR}/searchEverything.exe.bak"
                    "${INSTALL_DIR}/searchEverything" --version
                else
                    error "更新后验证失败，恢复旧版本"
                    if [ -f "${INSTALL_DIR}/searchEverything.bak" ]; then
                        mv "${INSTALL_DIR}/searchEverything.bak" "${INSTALL_DIR}/searchEverything"
                    elif [ -f "${INSTALL_DIR}/searchEverything.exe.bak" ]; then
                        mv "${INSTALL_DIR}/searchEverything.exe.bak" "${INSTALL_DIR}/searchEverything.exe"
                    fi
                    exit 1
                fi
            else
                info "取消更新"
            fi
            ;;
    esac
    
    echo ""
}

main "$@"
