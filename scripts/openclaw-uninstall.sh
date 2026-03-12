#!/bin/bash
#===============================================================================
# searchEverything OpenClaw 卸载脚本
# 功能：深度集成 OpenClaw，完全卸载 searchEverything 及相关配置
# GitHub: https://github.com/hanxueyuan/searchEverything
#
# 卸载模式:
#   - 完全卸载 (full): 删除所有数据，包括配置、索引、缓存
#   - 保留配置 (keep_config): 保留配置文件和索引数据
#   - 仅禁用 (disable_only): 保留所有数据，仅禁用 Skill
#
# 使用方法:
#   ./scripts/openclaw-uninstall.sh              # 交互式卸载
#   ./scripts/openclaw-uninstall.sh --full       # 完全卸载
#   ./scripts/openclaw-uninstall.sh --keep-config # 保留配置
#   ./scripts/openclaw-uninstall.sh --disable    # 仅禁用
#   ./scripts/openclaw-uninstall.sh --force      # 强制卸载，不确认
#   ./scripts/openclaw-uninstall.sh --verbose    # 详细输出
#   ./scripts/openclaw-uninstall.sh --help       # 显示帮助
#===============================================================================

set -euo pipefail

#===============================================================================
# 配置常量
#===============================================================================
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

# 备份目录
BACKUP_DIR="${HOME}/.openclaw/skills/searchEverything.backup.$(date +%Y%m%d-%H%M%S)"

# 卸载模式
UNINSTALL_MODE="interactive"  # interactive, full, keep_config, disable_only
FORCE="false"
VERBOSE="false"
DRY_RUN="false"
BACKUP_CONFIG="false"

#===============================================================================
# 颜色输出
#===============================================================================
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly MAGENTA='\033[0;35m'
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
        echo "[${timestamp}] [${level}] $message" >> "${LOG_DIR}/uninstall.log" 2>/dev/null || true
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
            --full)
                UNINSTALL_MODE="full"
                shift
                ;;
            --keep-config)
                UNINSTALL_MODE="keep_config"
                shift
                ;;
            --disable)
                UNINSTALL_MODE="disable_only"
                shift
                ;;
            --force|-f)
                FORCE="true"
                shift
                ;;
            --backup)
                BACKUP_CONFIG="true"
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
${BOLD}searchEverything OpenClaw 卸载脚本${NC}

用法：$0 [选项]

选项:
  --full            完全卸载，删除所有数据（包括配置、索引、缓存）
  --keep-config     保留配置文件和索引数据
  --disable         仅禁用 Skill，保留所有数据
  --force, -f       强制卸载，不显示确认提示
  --backup          卸载前备份配置文件
  --verbose, -v     显示详细输出
  --dry-run         模拟运行，不执行实际卸载
  --help, -h        显示此帮助信息

卸载模式:
  完全卸载 (full):      删除可执行文件、配置、索引、缓存、日志
  保留配置 (keep_config): 删除可执行文件，保留配置和索引
  仅禁用 (disable_only): 保留所有数据，仅从 OpenClaw 禁用

示例:
  $0                    # 交互式卸载
  $0 --full             # 完全卸载
  $0 --keep-config      # 保留配置
  $0 --force --full     # 强制完全卸载
  $0 --dry-run --full   # 模拟完全卸载

EOF
}

#===============================================================================
# 检查是否已安装
#===============================================================================
check_installation() {
    info "检查安装状态..."
    
    local installed=false
    
    if [ -f "${INSTALL_DIR}/searchEverything" ] || [ -f "${INSTALL_DIR}/searchEverything.exe" ]; then
        debug "找到可执行文件"
        installed=true
    fi
    
    if [ -d "$CONFIG_DIR" ]; then
        debug "找到配置目录"
        installed=true
    fi
    
    if [ "$installed" = "false" ]; then
        warning "未检测到 searchEverything 安装"
        echo ""
        echo "可能已经卸载，或者安装在非标准位置。"
        exit 0
    fi
    
    success "检测到 searchEverything 安装"
}

#===============================================================================
# 显示卸载内容
#===============================================================================
show_uninstall_summary() {
    echo ""
    echo "=========================================="
    echo "${BOLD}即将卸载以下内容:${NC}"
    echo "=========================================="
    echo ""
    
    case "$UNINSTALL_MODE" in
        full)
            echo "  ${RED}可执行文件:${NC} ${INSTALL_DIR}/searchEverything"
            echo "  ${RED}配置文件:${NC}  ${CONFIG_DIR}/"
            echo "  ${RED}索引数据:${NC}  ${INDEX_DIR}/"
            echo "  ${RED}缓存文件:${NC}  ${CACHE_DIR}/"
            echo "  ${RED}日志文件:${NC}  ${LOG_DIR}/"
            echo ""
            echo "  ${YELLOW}注意：索引数据库将被删除，重新安装需要重建索引${NC}"
            ;;
        keep_config)
            echo "  ${RED}可执行文件:${NC} ${INSTALL_DIR}/searchEverything"
            echo "  ${GREEN}保留配置:${NC}  ${CONFIG_DIR}/ (保留)"
            echo "  ${GREEN}保留索引:${NC}  ${INDEX_DIR}/ (保留)"
            echo "  ${YELLOW}删除缓存:${NC}  ${CACHE_DIR}/"
            echo "  ${YELLOW}删除日志:${NC}  ${LOG_DIR}/ (安装日志保留)"
            ;;
        disable_only)
            echo "  ${GREEN}保留所有数据${NC}"
            echo "  ${YELLOW}仅从 OpenClaw 禁用 Skill${NC}"
            ;;
    esac
    
    if [ "$BACKUP_CONFIG" = "true" ]; then
        echo ""
        echo "  ${CYAN}备份位置:${NC} $BACKUP_DIR"
    fi
    
    echo ""
}

#===============================================================================
# 确认卸载
#===============================================================================
confirm_uninstall() {
    if [ "$FORCE" = "true" ]; then
        info "强制模式，跳过确认"
        return 0
    fi
    
    if [ "$UNINSTALL_MODE" = "interactive" ]; then
        echo ""
        echo "请选择卸载模式:"
        echo "  1) 完全卸载 (删除所有数据)"
        echo "  2) 保留配置卸载 (保留配置和索引)"
        echo "  3) 仅禁用 (保留所有数据)"
        echo "  4) 取消卸载"
        echo ""
        read -p "请选择 [1-4]: " -n 1 -r
        echo ""
        
        case $REPLY in
            1) UNINSTALL_MODE="full" ;;
            2) UNINSTALL_MODE="keep_config" ;;
            3) UNINSTALL_MODE="disable_only" ;;
            4) info "取消卸载"; exit 0 ;;
            *) error "无效选择"; exit 1 ;;
        esac
        
        # 重新显示摘要
        show_uninstall_summary
    fi
    
    echo ""
    read -p "确定要继续卸载吗？[y/N] " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "取消卸载"
        exit 0
    fi
}

#===============================================================================
# 停止相关进程
#===============================================================================
stop_processes() {
    info "停止 searchEverything 相关进程..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将停止 searchEverything 进程"
        return 0
    fi
    
    # 查找并停止 searchEverything 进程
    local pids=$(pgrep -f "searchEverything" 2>/dev/null || true)
    
    if [ -n "$pids" ]; then
        for pid in $pids; do
            if kill "$pid" 2>/dev/null; then
                debug "已停止进程：$pid"
            fi
        done
        success "已停止所有 searchEverything 进程"
    else
        debug "未找到运行的 searchEverything 进程"
    fi
}

#===============================================================================
# 备份配置（可选）
#===============================================================================
backup_config() {
    if [ "$BACKUP_CONFIG" = "false" ]; then
        return 0
    fi
    
    info "备份配置文件..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将备份到：$BACKUP_DIR"
        return 0
    fi
    
    if [ -d "$CONFIG_DIR" ]; then
        mkdir -p "$BACKUP_DIR"
        cp -r "$CONFIG_DIR"/* "$BACKUP_DIR"/ 2>/dev/null || true
        success "配置已备份到：$BACKUP_DIR"
    fi
}

#===============================================================================
# 删除可执行文件
#===============================================================================
remove_binary() {
    if [ "$UNINSTALL_MODE" = "disable_only" ]; then
        info "禁用模式，保留可执行文件"
        return 0
    fi
    
    info "删除可执行文件..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将删除：${INSTALL_DIR}/searchEverything"
        debug "[DRY-RUN] 将删除：${INSTALL_DIR}/searchEverything.exe"
        return 0
    fi
    
    local removed=false
    
    if [ -f "${INSTALL_DIR}/searchEverything" ]; then
        rm -f "${INSTALL_DIR}/searchEverything"
        success "已删除：${INSTALL_DIR}/searchEverything"
        removed=true
    fi
    
    if [ -f "${INSTALL_DIR}/searchEverything.exe" ]; then
        rm -f "${INSTALL_DIR}/searchEverything.exe"
        success "已删除：${INSTALL_DIR}/searchEverything.exe"
        removed=true
    fi
    
    # 删除备份文件
    rm -f "${INSTALL_DIR}/searchEverything.bak" 2>/dev/null || true
    rm -f "${INSTALL_DIR}/searchEverything.exe.bak" 2>/dev/null || true
    
    if [ "$removed" = "false" ]; then
        warning "未找到可执行文件"
    fi
}

#===============================================================================
# 删除/保留配置文件
#===============================================================================
handle_config() {
    case "$UNINSTALL_MODE" in
        full)
            info "删除配置文件..."
            
            if [ "$DRY_RUN" = "true" ]; then
                debug "[DRY-RUN] 将删除：$CONFIG_DIR"
                return 0
            fi
            
            if [ -d "$CONFIG_DIR" ]; then
                rm -rf "$CONFIG_DIR"
                success "已删除：$CONFIG_DIR"
            else
                warning "配置目录不存在"
            fi
            ;;
        keep_config)
            info "保留配置文件..."
            debug "配置目录保留：$CONFIG_DIR"
            ;;
        disable_only)
            info "禁用模式，保留所有配置"
            ;;
    esac
}

#===============================================================================
# 删除/保留索引数据
#===============================================================================
handle_index() {
    case "$UNINSTALL_MODE" in
        full)
            info "删除索引数据..."
            
            if [ "$DRY_RUN" = "true" ]; then
                debug "[DRY-RUN] 将删除：$INDEX_DIR"
                return 0
            fi
            
            if [ -d "$INDEX_DIR" ]; then
                rm -rf "$INDEX_DIR"
                success "已删除：$INDEX_DIR"
            else
                debug "索引目录不存在"
            fi
            ;;
        keep_config|disable_only)
            info "保留索引数据..."
            debug "索引目录保留：$INDEX_DIR"
            ;;
    esac
}

#===============================================================================
# 删除/保留缓存文件
#===============================================================================
handle_cache() {
    case "$UNINSTALL_MODE" in
        full)
            info "删除缓存文件..."
            
            if [ "$DRY_RUN" = "true" ]; then
                debug "[DRY-RUN] 将删除：$CACHE_DIR"
                return 0
            fi
            
            if [ -d "$CACHE_DIR" ]; then
                rm -rf "$CACHE_DIR"
                success "已删除：$CACHE_DIR"
            else
                debug "缓存目录不存在"
            fi
            ;;
        keep_config|disable_only)
            info "保留缓存文件..."
            debug "缓存目录保留：$CACHE_DIR"
            ;;
    esac
}

#===============================================================================
# 清理日志文件
#===============================================================================
handle_logs() {
    case "$UNINSTALL_MODE" in
        full)
            info "删除日志文件..."
            
            if [ "$DRY_RUN" = "true" ]; then
                debug "[DRY-RUN] 将删除：$LOG_DIR"
                return 0
            fi
            
            # 保留当前卸载日志
            if [ -d "$LOG_DIR" ]; then
                # 先复制当前日志
                if [ -f "${LOG_DIR}/uninstall.log" ]; then
                    cp "${LOG_DIR}/uninstall.log" "${LOG_DIR}/uninstall.log.final" 2>/dev/null || true
                fi
                rm -rf "$LOG_DIR"
                success "已删除：$LOG_DIR"
            else
                debug "日志目录不存在"
            fi
            ;;
        keep_config|disable_only)
            info "保留日志文件..."
            debug "日志目录保留：$LOG_DIR"
            ;;
    esac
}

#===============================================================================
# 清理 PATH 配置
#===============================================================================
cleanup_path() {
    info "检查 shell 配置文件..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将清理 shell 配置文件"
        return 0
    fi
    
    local files=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile")
    local cleaned=0
    
    for file in "${files[@]}"; do
        if [ -f "$file" ]; then
            if grep -q "searchEverything" "$file" 2>/dev/null || grep -q "${INSTALL_DIR//\//\\/}" "$file" 2>/dev/null; then
                # 创建备份
                local backup_file="${file}.backup.$(date +%Y%m%d%H%M%S)"
                cp "$file" "$backup_file"
                
                # 删除相关行
                sed -i.bak '/searchEverything/d' "$file" 2>/dev/null || true
                sed -i.bak "/${INSTALL_DIR//\//\\/}/d" "$file" 2>/dev/null || true
                
                # 清理空行
                sed -i '/^$/N;/^\n$/d' "$file" 2>/dev/null || true
                
                info "已清理：$file (备份：$backup_file)"
                cleaned=$((cleaned + 1))
            fi
        fi
    done
    
    if [ $cleaned -gt 0 ]; then
        success "已清理 $cleaned 个配置文件"
        echo ""
        warning "请检查并删除备份文件：*.backup.*"
    fi
}

#===============================================================================
# 从 OpenClaw 移除注册
#===============================================================================
unregister_from_openclaw() {
    info "从 OpenClaw 移除注册..."
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将从 OpenClaw 移除注册"
        return 0
    fi
    
    # 如果 OpenClaw 可用，尝试注销
    if command -v openclaw &> /dev/null; then
        # 禁用技能
        if openclaw skills disable searchEverything 2>/dev/null; then
            success "已从 OpenClaw 禁用 searchEverything"
        else
            debug "OpenClaw 技能禁用跳过"
        fi
        
        # 刷新技能列表
        if openclaw skills refresh 2>/dev/null; then
            success "已刷新 OpenClaw 技能列表"
        fi
    else
        debug "OpenClaw 命令不可用，跳过注销"
    fi
    
    # 删除 OpenClaw 技能目录（如果不是保留模式）
    if [ "$UNINSTALL_MODE" = "full" ]; then
        if [ -d "$OPENCLAW_SKILL_DIR" ]; then
            rm -rf "$OPENCLAW_SKILL_DIR"
            success "已删除 OpenClaw 技能目录：$OPENCLAW_SKILL_DIR"
        fi
    fi
}

#===============================================================================
# 生成卸载报告
#===============================================================================
generate_uninstall_report() {
    local report_file="${HOME}/searchEverything-uninstall-report-$(date +%Y%m%d-%H%M%S).json"
    
    if [ "$DRY_RUN" = "true" ]; then
        debug "[DRY-RUN] 将生成卸载报告：$report_file"
        return 0
    fi
    
    cat > "$report_file" << EOF
{
  "uninstallation": {
    "timestamp": "$(date -Iseconds)",
    "mode": "$UNINSTALL_MODE",
    "force": $FORCE,
    "backup_created": $BACKUP_CONFIG,
    "paths": {
      "install_dir": "$INSTALL_DIR",
      "config_dir": "$CONFIG_DIR",
      "backup_dir": "$BACKUP_DIR"
    },
    "openclaw": {
      "enabled": $OPENCLAW_MODE,
      "unregistered": true
    },
    "status": "success"
  }
}
EOF
    
    success "卸载报告已生成：$report_file"
}

#===============================================================================
# 显示完成信息
#===============================================================================
show_completion_message() {
    echo ""
    echo "=========================================="
    success "${BOLD}searchEverything 卸载完成！${NC}"
    echo "=========================================="
    echo ""
    
    case "$UNINSTALL_MODE" in
        full)
            echo "已完全卸载 searchEverything 及所有相关数据。"
            echo ""
            echo "如需重新安装，请运行:"
            echo "  ./scripts/openclaw-install.sh"
            echo ""
            echo "或访问 GitHub:"
            echo "  https://github.com/hanxueyuan/searchEverything"
            ;;
        keep_config)
            echo "已卸载可执行文件，配置和索引数据已保留。"
            echo ""
            echo "配置文件位置：$CONFIG_DIR"
            echo "索引数据位置：$INDEX_DIR"
            echo ""
            echo "如需重新安装（保留配置），请运行:"
            echo "  ./scripts/openclaw-install.sh"
            ;;
        disable_only)
            echo "已禁用 searchEverything Skill，所有数据已保留。"
            echo ""
            echo "如需重新启用，请在 OpenClaw 中运行:"
            echo "  openclaw skills enable searchEverything"
            ;;
    esac
    
    echo ""
    
    if [ "$BACKUP_CONFIG" = "true" ]; then
        echo "配置备份位置：$BACKUP_DIR"
        echo ""
    fi
    
    echo "=========================================="
}

#===============================================================================
# 残留清理指南
#===============================================================================
show_cleanup_guide() {
    if [ "$UNINSTALL_MODE" != "full" ]; then
        return 0
    fi
    
    echo ""
    echo "=========================================="
    echo "${BOLD}残留清理指南${NC}"
    echo "=========================================="
    echo ""
    echo "如果仍有残留文件，请手动检查以下位置:"
    echo ""
    echo "  ~/.local/bin/searchEverything*"
    echo "  ~/.config/searchEverything/"
    echo "  ~/.cache/searchEverything/"
    echo "  ~/.openclaw/skills/searchEverything/"
    echo ""
    echo "Shell 配置文件中的残留:"
    echo "  ~/.bashrc"
    echo "  ~/.zshrc"
    echo "  ~/.bash_profile"
    echo ""
    echo "=========================================="
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    echo ""
    echo "=========================================="
    echo "${BOLD}  searchEverything OpenClaw 卸载程序${NC}"
    echo "  GitHub: https://github.com/hanxueyuan/searchEverything"
    echo "=========================================="
    
    # 解析参数
    parse_args "$@"
    
    # 检查安装
    check_installation
    
    # 显示摘要
    show_uninstall_summary
    
    # 确认卸载
    confirm_uninstall
    
    echo ""
    
    # 执行卸载步骤
    stop_processes
    backup_config
    remove_binary
    handle_config
    handle_index
    handle_cache
    handle_logs
    cleanup_path
    unregister_from_openclaw
    generate_uninstall_report
    
    show_completion_message
    show_cleanup_guide
}

# 运行主函数
main "$@"
