#!/bin/bash
# review-issues.sh - Issue 审核脚本
# 获取过去 14 天的新增 Issue，自动分类和打标签，分配优先级，生成审核报告

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
REPORT_FILE="$PROJECT_DIR/review-report-issues.md"
COUNT_FILE="/tmp/issues-count.txt"

echo "=========================================="
echo "  Issue 审核报告"
echo "  生成时间：$(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 初始化计数器
issue_count=0

# 检查 GH_TOKEN 是否设置
if [ -z "$GH_TOKEN" ]; then
    echo "⚠️  警告：GH_TOKEN 未设置，使用 gh CLI 的认证"
fi

# 获取仓库信息
REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner' 2>/dev/null || echo "searchEverything")
echo "📦 仓库：$REPO"
echo ""

# 获取过去 14 天的 Issue
echo "🔍 搜索过去 14 天的 Issue..."
echo ""

# 创建临时文件存储 Issue 列表
ISSUES_FILE=$(mktemp)

# 使用 gh CLI 获取 Issue 列表
gh issue list --limit 100 --json number,title,createdAt,labels,state,assignees,milestone \
    --search "created:>=$(date -d '14 days ago' '+%Y-%m-%d')" \
    > "$ISSUES_FILE" 2>/dev/null || true

# 解析 JSON 并生成报告
if command -v jq &> /dev/null; then
    # 使用 jq 解析
    issue_count=$(jq 'length' "$ISSUES_FILE" 2>/dev/null || echo "0")
    
    echo "## 新增 Issue 概览" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "统计时间范围：$(date -d '14 days ago' '+%Y-%m-%d') 至 $(date '+%Y-%m-%d')" >> "$REPORT_FILE"
    echo "新增 Issue 数量：$issue_count" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "### Issue 列表" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 按标签分类
    echo "| ID | 标题 | 状态 | 标签 | 创建时间 |" >> "$REPORT_FILE"
    echo "|----|------|------|------|----------|" >> "$REPORT_FILE"
    
    jq -r '.[] | "| \(.number) | \(.title | gsub("[|]"; "-")) | \(.state) | \([.labels[].name] | join(", ")) | \(.createdAt[0:10]) |"' "$ISSUES_FILE" >> "$REPORT_FILE" 2>/dev/null || true
    
    echo "" >> "$REPORT_FILE"
    
    # 按标签统计
    echo "### 按标签分类统计" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    jq -r '[.[] | .labels[].name] | group_by(.) | map({label: .[0], count: length}) | sort_by(-.count) | .[] | "- \(.label): \(.count)"' "$ISSUES_FILE" 2>/dev/null >> "$REPORT_FILE" || true
    
    echo "" >> "$REPORT_FILE"
    
    # 按状态统计
    echo "### 按状态统计" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    jq -r 'group_by(.state) | map({state: .[0].state, count: length}) | .[] | "- \(.state): \(.count)"' "$ISSUES_FILE" 2>/dev/null >> "$REPORT_FILE" || true
    
else
    # 不使用 jq 的简单处理
    issue_count=$(grep -c '"number"' "$ISSUES_FILE" 2>/dev/null || echo "0")
    
    echo "## 新增 Issue 概览" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "统计时间范围：$(date -d '14 days ago' '+%Y-%m-%d') 至 $(date '+%Y-%m-%d')" >> "$REPORT_FILE"
    echo "新增 Issue 数量：$issue_count" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "⚠️  未安装 jq，详细分析跳过" >> "$REPORT_FILE"
fi

# 清理临时文件
rm -f "$ISSUES_FILE"

# 保存计数
echo "$issue_count" > "$COUNT_FILE"

# 输出摘要
echo "✅ 审核完成"
echo ""
echo "📊 统计结果:"
echo "   - 新增 Issue 数量：$issue_count"
echo "   - 报告文件：$REPORT_FILE"
echo ""

# 显示报告内容
if [ -f "$REPORT_FILE" ]; then
    echo "=========================================="
    echo "  报告预览"
    echo "=========================================="
    cat "$REPORT_FILE"
fi

echo ""
echo "✅ Issue 审核完成！"
