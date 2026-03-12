#!/bin/bash
#===============================================================================
# searchEverything 卸载脚本
# 功能：完全卸载 searchEverything 及相关配置
# GitHub: https://github.com/hanxueyuan/searchEverything
#===============================================================================

set -e

# 配置常量
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/searchEverything"
CACHE_DIR="${HOME}/.cache/searchEverything"

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
# 确认卸载
#-------------------------------------------------------------------------------
confirm() {
    echo ""
    echo "=========================================="
    echo "  即将卸载以下内容:"
    echo "=========================================="
    echo ""
    echo "  可执行文件：${INSTALL_DIR}/searchEverything"
    echo "  配置文件：  ${CONFIG_DIR}/"
    echo "  缓存文件：  ${CACHE_DIR}/"
    echo ""
    echo "  注意：索引数据库将被删除，重新安装需要重建索引"
    echo ""
    
    read -p "确定要卸载 searchEverything 吗？[y/N] " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "取消卸载"
        exit 0
    fi
}

#-------------------------------------------------------------------------------
# 删除可执行文件
#-------------------------------------------------------------------------------
remove_binary() {
    info "删除可执行文件..."
    
    if [ -f "${INSTALL_DIR}/searchEverything" ]; then
        rm -f "${INSTALL_DIR}/searchEverything"
        success "已删除：${INSTALL_DIR}/searchEverything"
    elif [ -f "${INSTALL_DIR}/searchEverything.exe" ]; then
        rm -f "${INSTALL_DIR}/searchEverything.exe"
        success "已删除：${INSTALL_DIR}/searchEverything.exe"
    else
        warning "未找到可执行文件"
    fi
    
    # 删除备份文件
    rm -f "${INSTALL_DIR}/searchEverything.bak"
    rm -f "${INSTALL_DIR}/searchEverything.exe.bak"
}

#-------------------------------------------------------------------------------
# 删除配置文件
#-------------------------------------------------------------------------------
remove_config() {
    info "删除配置文件..."
    
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        success "已删除：$CONFIG_DIR"
    else
        warning "配置目录不存在"
    fi
}

#-------------------------------------------------------------------------------
# 删除缓存文件
#-------------------------------------------------------------------------------
remove_cache() {
    info "删除缓存文件..."
    
    if [ -d "$CACHE_DIR" ]; then
        rm -rf "$CACHE_DIR"
        success "已删除：$CACHE_DIR"
    else
        warning "缓存目录不存在"
    fi
}

#-------------------------------------------------------------------------------
# 清理 PATH (可选)
#-------------------------------------------------------------------------------
cleanup_path() {
    info "检查 shell 配置文件..."
    
    local files=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile")
    local cleaned=0
    
    for file in "${files[@]}"; do
        if [ -f "$file" ]; then
            if grep -q "searchEverything" "$file" 2>/dev/null; then
                # 创建备份
                cp "$file" "${file}.backup.$(date +%Y%m%d%H%M%S)"
                
                # 删除相关行
                sed -i.bak '/searchEverything/d' "$file"
                sed -i.bak "/${INSTALL_DIR//\//\\/}/d" "$file"
                
                # 清理空行
                sed -i '/^$/N;/^\n$/d' "$file"
                
                info "已清理：$file (备份：${file}.backup.*)"
                cleaned=$((cleaned + 1))
            fi
        fi
    done
    
    if [ $cleaned -gt 0 ]; then
        success "已清理 $cleaned 个配置文件中的 searchEverything 相关配置"
        echo ""
        warning "请手动检查并删除备份文件：*.backup.*"
    fi
}

#-------------------------------------------------------------------------------
# 主函数
#-------------------------------------------------------------------------------
main() {
    echo ""
    echo "=========================================="
    echo "  searchEverything 卸载程序"
    echo "=========================================="
    
    # 检查是否已安装
    if [ ! -f "${INSTALL_DIR}/searchEverything" ] && [ ! -f "${INSTALL_DIR}/searchEverything.exe" ]; then
        if [ ! -d "$CONFIG_DIR" ]; then
            warning "未检测到 searchEverything 安装"
            exit 0
        fi
    fi
    
    confirm
    echo ""
    
    remove_binary
    remove_config
    remove_cache
    cleanup_path
    
    echo ""
    echo "=========================================="
    success "searchEverything 已完全卸载"
    echo "=========================================="
    echo ""
    echo "如需重新安装，请运行:"
    echo "  scripts/auto-install.sh"
    echo ""
    echo "或访问 GitHub:"
    echo "  https://github.com/hanxueyuan/searchEverything"
    echo ""
}

main "$@"
