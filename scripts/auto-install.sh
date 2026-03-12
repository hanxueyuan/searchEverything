#!/bin/bash
#===============================================================================
# searchEverything 自动安装脚本
# 功能：自动下载并安装 searchEverything 可执行文件
# GitHub: https://github.com/hanxueyuan/searchEverything
#===============================================================================

set -e

# 配置常量
REPO_OWNER="hanxueyuan"
REPO_NAME="searchEverything"
GITHUB_API_BASE="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}"
GITHUB_RELEASES_BASE="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/searchEverything"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

#-------------------------------------------------------------------------------
# 打印函数
#-------------------------------------------------------------------------------
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

#-------------------------------------------------------------------------------
# 检测操作系统和架构
#-------------------------------------------------------------------------------
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
    # 格式：searchEverything-{arch}-{os}{.exe}
    BINARY_NAME="searchEverything-${ARCH}-${OS}${EXT}"
    
    success "检测到平台：${ARCH}-${OS}"
    info "目标文件：$BINARY_NAME"
}

#-------------------------------------------------------------------------------
# 获取最新版本号
#-------------------------------------------------------------------------------
get_latest_version() {
    info "获取最新版本信息..."
    
    # 尝试通过 GitHub API 获取最新版本
    if command -v curl &> /dev/null; then
        LATEST_VERSION=$(curl -s "${GITHUB_API_BASE}/releases/latest" | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
        
        if [ -n "$LATEST_VERSION" ]; then
            success "最新版本：$LATEST_VERSION"
            return 0
        fi
    fi
    
    warning "无法获取版本信息，将下载最新 release"
    LATEST_VERSION="latest"
    return 0
}

#-------------------------------------------------------------------------------
# 构建下载 URL
#-------------------------------------------------------------------------------
build_download_url() {
    if [ "$LATEST_VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/${BINARY_NAME}"
    else
        DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${LATEST_VERSION}/${BINARY_NAME}"
    fi
    
    info "下载 URL: $DOWNLOAD_URL"
}

#-------------------------------------------------------------------------------
# 检查依赖
#-------------------------------------------------------------------------------
check_dependencies() {
    info "检查依赖工具..."
    
    if ! command -v curl &> /dev/null; then
        error "需要 curl 工具，请先安装：sudo apt install curl (Linux) 或 brew install curl (macOS)"
        exit 1
    fi
    
    success "依赖检查通过"
}

#-------------------------------------------------------------------------------
# 创建安装目录
#-------------------------------------------------------------------------------
setup_install_dir() {
    info "准备安装目录..."
    
    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR"
        info "创建安装目录：$INSTALL_DIR"
    fi
    
    # 创建配置目录
    if [ ! -d "$CONFIG_DIR" ]; then
        mkdir -p "$CONFIG_DIR"
        info "创建配置目录：$CONFIG_DIR"
    fi
}

#-------------------------------------------------------------------------------
# 下载二进制文件
#-------------------------------------------------------------------------------
download_binary() {
    local temp_file=$(mktemp)
    
    info "下载二进制文件..."
    info "从 $DOWNLOAD_URL 下载"
    
    # 下载文件
    if curl -L -# --fail -o "$temp_file" "$DOWNLOAD_URL" 2>&1; then
        success "下载完成"
        mv "$temp_file" "${INSTALL_DIR}/searchEverything${EXT}"
    else
        local exit_code=$?
        rm -f "$temp_file"
        
        error "下载失败 (错误代码：$exit_code)"
        echo ""
        echo "=========================================="
        echo "手动安装说明"
        echo "=========================================="
        echo ""
        echo "1. 访问 GitHub Releases 页面:"
        echo "   ${GITHUB_RELEASES_BASE}/latest"
        echo ""
        echo "2. 下载适合您平台的文件:"
        echo "   - Linux x86_64: searchEverything-x86_64-linux"
        echo "   - Linux aarch64: searchEverything-aarch64-linux"
        echo "   - macOS x86_64: searchEverything-x86_64-macos"
        echo "   - macOS aarch64: searchEverything-aarch64-macos"
        echo "   - Windows x86_64: searchEverything-x86_64-windows.exe"
        echo ""
        echo "3. 将下载的文件移动到 ~/.local/bin/ 并添加执行权限:"
        if [ "$OS" != "windows" ]; then
            echo "   chmod +x searchEverything"
            echo "   mv searchEverything ~/.local/bin/"
        else
            echo "   将 searchEverything.exe 移动到 %USERPROFILE%\\.local\\bin\\"
        fi
        echo ""
        echo "4. 验证安装:"
        echo "   searchEverything --version"
        echo ""
        echo "=========================================="
        
        exit 1
    fi
}

#-------------------------------------------------------------------------------
# 设置执行权限
#-------------------------------------------------------------------------------
set_permissions() {
    if [ "$OS" != "windows" ]; then
        info "设置执行权限..."
        chmod +x "${INSTALL_DIR}/searchEverything"
        success "权限设置完成"
    fi
}

#-------------------------------------------------------------------------------
# 验证安装
#-------------------------------------------------------------------------------
verify_installation() {
    info "验证安装..."
    
    # 临时添加安装目录到 PATH
    export PATH="${INSTALL_DIR}:${PATH}"
    
    # 尝试运行版本命令
    if "${INSTALL_DIR}/searchEverything${EXT}" --version > /dev/null 2>&1; then
        local version=$("${INSTALL_DIR}/searchEverything${EXT}" --version 2>&1)
        success "安装验证成功！$version"
        return 0
    else
        warning "验证失败，但文件已安装"
        return 1
    fi
}

#-------------------------------------------------------------------------------
# 检查 PATH 配置
#-------------------------------------------------------------------------------
check_path() {
    info "检查 PATH 配置..."
    
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        success "安装目录已在 PATH 中"
        return 0
    fi
    
    warning "安装目录不在 PATH 中"
    echo ""
    echo "=========================================="
    echo "PATH 配置说明"
    echo "=========================================="
    echo ""
    echo "请将以下行添加到您的 shell 配置文件中:"
    echo ""
    
    case "$SHELL" in
        */bash*)
            echo "对于 bash (~/.bashrc 或 ~/.bash_profile):"
            echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
        */zsh*)
            echo "对于 zsh (~/.zshrc):"
            echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
        */fish*)
            echo "对于 fish (~/.config/fish/config.fish):"
            echo "  set -gx PATH ${INSTALL_DIR} \$PATH"
            ;;
        *)
            echo "export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
    esac
    
    echo ""
    echo "然后重新加载配置文件或重新打开终端"
    echo ""
    
    # 尝试自动添加到配置文件
    local profile_added=false
    
    if [ -f "$HOME/.bashrc" ] && [ "$SHELL" = */bash* ]; then
        if ! grep -q "$INSTALL_DIR" "$HOME/.bashrc" 2>/dev/null; then
            echo "" >> "$HOME/.bashrc"
            echo "# searchEverything" >> "$HOME/.bashrc"
            echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$HOME/.bashrc"
            info "已添加到 ~/.bashrc"
            profile_added=true
        fi
    elif [ -f "$HOME/.zshrc" ] && [ "$SHELL" = */zsh* ]; then
        if ! grep -q "$INSTALL_DIR" "$HOME/.zshrc" 2>/dev/null; then
            echo "" >> "$HOME/.zshrc"
            echo "# searchEverything" >> "$HOME/.zshrc"
            echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$HOME/.zshrc"
            info "已添加到 ~/.zshrc"
            profile_added=true
        fi
    fi
    
    if [ "$profile_added" = true ]; then
        echo ""
        info "请运行 'source ~/.bashrc' 或 'source ~/.zshrc' 使配置生效"
    fi
    
    echo "=========================================="
}

#-------------------------------------------------------------------------------
# 创建默认配置
#-------------------------------------------------------------------------------
create_default_config() {
    info "创建默认配置文件..."
    
    local config_file="${CONFIG_DIR}/config.yaml"
    
    if [ ! -f "$config_file" ]; then
        cat > "$config_file" << 'EOF'
# searchEverything 配置文件
# 自动生成于安装时

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

audit:
  enabled: true
  log_file: "~/.config/searchEverything/audit.log"
EOF
        success "默认配置已创建：$config_file"
    else
        info "配置文件已存在，跳过创建"
    fi
}

#-------------------------------------------------------------------------------
# 主函数
#-------------------------------------------------------------------------------
main() {
    echo ""
    echo "=========================================="
    echo "  searchEverything 自动安装脚本"
    echo "  GitHub: https://github.com/hanxueyuan/searchEverything"
    echo "=========================================="
    echo ""
    
    # 执行安装步骤
    detect_platform
    check_dependencies
    get_latest_version
    build_download_url
    setup_install_dir
    download_binary
    set_permissions
    verify_installation
    check_path
    create_default_config
    
    echo ""
    echo "=========================================="
    success "searchEverything 安装完成！"
    echo "=========================================="
    echo ""
    echo "快速开始:"
    echo "  searchEverything search \"*.pdf\"     # 搜索 PDF 文件"
    echo "  searchEverything index list           # 查看索引"
    echo "  searchEverything config show          # 查看配置"
    echo "  searchEverything --help               # 显示帮助"
    echo ""
    echo "文档：https://github.com/hanxueyuan/searchEverything"
    echo ""
}

# 运行主函数
main "$@"
