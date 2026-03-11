#!/bin/bash
# searchEverything 自动启动配置脚本

set -e

echo "======================================"
echo "  searchEverything 自动启动配置"
echo "======================================"
echo

# 检测 init 系统
detect_init_system() {
    if [ -f /run/systemd/system ]; then
        echo "systemd"
    elif [ -f /etc/init.d/cron ]; then
        echo "sysvinit"
    else
        echo "unknown"
    fi
}

# systemd 配置
setup_systemd() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    echo "检测到 systemd 系统"
    echo
    
    # 检查服务文件
    if [ ! -f "$SCRIPT_DIR/searcheverything.service" ]; then
        echo "错误：未找到服务文件"
        exit 1
    fi
    
    # 复制到系统目录
    echo "正在安装服务文件..."
    sudo cp "$SCRIPT_DIR/searcheverything.service" /etc/systemd/system/
    
    # 重新加载 systemd
    echo "正在重新加载 systemd 配置..."
    sudo systemctl daemon-reload
    
    echo
    echo "✓ systemd 服务已安装"
    echo
    echo "可用命令:"
    echo "  sudo systemctl enable searcheverything   # 开机自启"
    echo "  sudo systemctl start searcheverything    # 启动服务"
    echo "  sudo systemctl stop searcheverything     # 停止服务"
    echo "  sudo systemctl status searcheverything   # 查看状态"
    echo "  journalctl -u searcheverything -f        # 查看日志"
}

# cron 配置（备用方案）
setup_cron() {
    echo "配置 cron 定时索引更新..."
    
    CRON_JOB="0 * * * * /usr/local/bin/se index rebuild > /dev/null 2>&1"
    
    # 检查是否已存在
    if crontab -l 2>/dev/null | grep -q "searchEverything"; then
        echo "✓ cron 任务已存在"
    else
        (crontab -l 2>/dev/null; echo "$CRON_JOB") | crontab -
        echo "✓ cron 任务已添加（每小时重建索引）"
    fi
}

# 主流程
main() {
    INIT_SYSTEM=$(detect_init_system)
    
    case $INIT_SYSTEM in
        systemd)
            setup_systemd
            ;;
        sysvinit)
            setup_cron
            ;;
        *)
            echo "警告：无法检测到 init 系统"
            echo "请手动配置自动启动"
            ;;
    esac
    
    echo
    echo "配置完成!"
}

main "$@"
