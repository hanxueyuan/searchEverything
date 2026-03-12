#!/bin/bash
# searchEverything 安装脚本
# 用于 Linux 系统

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
BIN_DIR="$INSTALL_PREFIX/bin"

echo "======================================"
echo "  searchEverything 安装脚本"
echo "======================================"
echo

# 检查是否以 root 运行
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo "错误：请使用 sudo 运行此脚本"
        echo "  sudo $0"
        exit 1
    fi
}

# 检查 Rust 环境
check_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "错误：未找到 Rust/Cargo"
        echo "请先安装 Rust: https://rustup.rs/"
        exit 1
    fi
    echo "✓ Rust 环境检查通过"
}

# 编译项目
build() {
    echo "正在编译 searchEverything..."
    cd "$PROJECT_DIR"
    cargo build --release
    
    echo "✓ 编译完成"
}

# 安装二进制文件
install_binary() {
    echo "正在安装二进制文件到 $BIN_DIR..."
    
    mkdir -p "$BIN_DIR"
    cp "$PROJECT_DIR/target/release/searchEverything" "$BIN_DIR/"
    
    chmod +x "$BIN_DIR/searchEverything"
    
    echo "✓ 二进制文件安装完成"
}

# 安装 systemd 服务
install_systemd() {
    echo "正在安装 systemd 服务..."
    
    cp "$SCRIPT_DIR/searcheverything.service" /etc/systemd/system/
    
    echo "✓ systemd 服务安装完成"
    echo
    echo "启用服务命令:"
    echo "  sudo systemctl daemon-reload"
    echo "  sudo systemctl enable searcheverything"
    echo "  sudo systemctl start searcheverything"
}

# 配置 shell 自动补全
install_completion() {
    echo "正在安装 shell 自动补全..."
    
    # Bash 补全
    local bash_completion_dir="/etc/bash_completion.d"
    mkdir -p "$bash_completion_dir"
    
    cat > "$bash_completion_dir/searcheverything" << 'EOF'
_searchEverything_completion() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local commands="search info cat copy move delete index config help"
    
    if [ $COMP_CWORD -eq 1 ]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
    else
        COMPREPLY=($(compgen -f -- "$cur"))
    fi
}
complete -F _searchEverything_completion searchEverything
EOF
    
    echo "✓ Bash 补全安装完成"
}

# 显示使用说明
show_usage() {
    echo
    echo "======================================"
    echo "  安装完成!"
    echo "======================================"
    echo
    echo "使用方法:"
    echo "  searchEverything search \"*.rs\"          # 搜索文件"
    echo "  searchEverything index status             # 查看索引状态"
    echo "  searchEverything index rebuild            # 重建索引"
    echo "  searchEverything index watch /home        # 启动实时监控"
    echo
    echo "systemd 服务管理:"
    echo "  sudo systemctl enable searcheverything"
    echo "  sudo systemctl start searcheverything"
    echo "  sudo systemctl status searcheverything"
    echo
    echo "配置文件位置:"
    echo "  ~/.config/searchEverything/config.yaml"
    echo "  ~/.config/searchEverything/index-config.yaml"
    echo
}

# 主流程
main() {
    check_rust
    
    # 如果是系统安装，需要 root
    if [ "$1" = "--system" ]; then
        check_root
        build
        install_binary
        install_systemd
    else
        # 用户安装
        build
        install_binary
    fi
    
    install_completion
    show_usage
}

main "$@"
