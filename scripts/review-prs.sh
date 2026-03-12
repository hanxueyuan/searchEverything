#!/bin/bash
# review-prs.sh - PR 审核脚本
# 获取新增 PR，运行自动化测试检查，代码质量检查，生成审核意见

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
REPORT_FILE="$PROJECT_DIR/review-report-prs.md"
COUNT_FILE="/tmp/prs-count.txt"

echo "=========================================="
echo "  PR 审核报告"
echo "  生成时间：$(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="
echo ""

# 初始化计数器
pr_count=0

# 检查 GH_TOKEN 是否设置
if [ -z "$GH_TOKEN" ]; then
    echo "⚠️  警告：GH_TOKEN 未设置，使用 gh CLI 的认证"
fi

# 获取仓库信息
REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner' 2>/dev/null || echo "searchEverything")
echo "📦 仓库：$REPO"
echo ""

# 获取过去 14 天的 PR
echo "🔍 搜索过去 14 天的 PR..."
echo ""

# 创建临时文件存储 PR 列表
PRS_FILE=$(mktemp)

# 使用 gh CLI 获取 PR 列表
gh pr list --limit 100 --json number,title,createdAt,labels,state,assignees,milestone,mergeable,isDraft,author \
    --search "created:>=$(date -d '14 days ago' '+%Y-%m-%d')" \
    > "$PRS_FILE" 2>/dev/null || true

# 解析 JSON 并生成报告
if command -v jq &> /dev/null; then
    # 使用 jq 解析
    pr_count=$(jq 'length' "$PRS_FILE" 2>/dev/null || echo "0")
    
    echo "## 新增 PR 概览" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "统计时间范围：$(date -d '14 days ago' '+%Y-%m-%d') 至 $(date '+%Y-%m-%d')" >> "$REPORT_FILE"
    echo "新增 PR 数量：$pr_count" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "### PR 列表" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    echo "| ID | 标题 | 状态 | 作者 | 可合并 | 创建时间 |" >> "$REPORT_FILE"
    echo "|----|------|------|------|--------|----------|" >> "$REPORT_FILE"
    
    jq -r '.[] | "| \(.number) | \(.title | gsub("[|]"; "-")) | \(.state) | \(.author.login) | \(.mergeable) | \(.createdAt[0:10]) |"' "$PRS_FILE" >> "$REPORT_FILE" 2>/dev/null || true
    
    echo "" >> "$REPORT_FILE"
    
    # 按状态统计
    echo "### 按状态统计" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    jq -r 'group_by(.state) | map({state: .[0].state, count: length}) | .[] | "- \(.state): \(.count)"' "$PRS_FILE" 2>/dev/null >> "$REPORT_FILE" || true
    
    echo "" >> "$REPORT_FILE"
    
    # Draft PR 统计
    echo "### Draft PR 统计" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    draft_count=$(jq '[.[] | select(.isDraft == true)] | length' "$PRS_FILE" 2>/dev/null || echo "0")
    echo "- Draft PR 数量：$draft_count" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 代码质量检查建议
    echo "### 代码质量检查建议" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 检查是否有 Rust 项目
    if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
        echo "✅ Rust 项目检测:" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "\`\`\`bash" >> "$REPORT_FILE"
        echo "# 建议运行以下检查:" >> "$REPORT_FILE"
        echo "cargo clippy --all-targets --all-features" >> "$REPORT_FILE"
        echo "cargo fmt --check" >> "$REPORT_FILE"
        echo "cargo test" >> "$REPORT_FILE"
        echo "\`\`\`" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
    
    # 检查是否有 Node.js 项目
    if [ -f "$PROJECT_DIR/package.json" ]; then
        echo "✅ Node.js 项目检测:" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "\`\`\`bash" >> "$REPORT_FILE"
        echo "# 建议运行以下检查:" >> "$REPORT_FILE"
        echo "npm run lint" >> "$REPORT_FILE"
        echo "npm test" >> "$REPORT_FILE"
        echo "\`\`\`" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
    
    # 自动化测试检查
    echo "### 自动化测试检查" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    # 检查 CI 工作流文件
    if [ -d "$PROJECT_DIR/.github/workflows" ]; then
        workflow_count=$(ls -1 "$PROJECT_DIR/.github/workflows"/*.yml 2>/dev/null | wc -l || echo "0")
        echo "- CI/CD 工作流文件数量：$workflow_count" >> "$REPORT_FILE"
        
        # 列出工作流文件
        echo "- 工作流列表:" >> "$REPORT_FILE"
        for wf in "$PROJECT_DIR/.github/workflows"/*.yml; do
            if [ -f "$wf" ]; then
                echo "  - $(basename "$wf")" >> "$REPORT_FILE"
            fi
        done
        echo "" >> "$REPORT_FILE"
    fi
    
else
    # 不使用 jq 的简单处理
    pr_count=$(grep -c '"number"' "$PRS_FILE" 2>/dev/null || echo "0")
    
    echo "## 新增 PR 概览" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "统计时间范围：$(date -d '14 days ago' '+%Y-%m-%d') 至 $(date '+%Y-%m-%d')" >> "$REPORT_FILE"
    echo "新增 PR 数量：$pr_count" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "⚠️  未安装 jq，详细分析跳过" >> "$REPORT_FILE"
fi

# 清理临时文件
rm -f "$PRS_FILE"

# 保存计数
echo "$pr_count" > "$COUNT_FILE"

# 输出摘要
echo "✅ 审核完成"
echo ""
echo "📊 统计结果:"
echo "   - 新增 PR 数量：$pr_count"
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
echo "✅ PR 审核完成！"
