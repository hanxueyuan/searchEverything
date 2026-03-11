#!/bin/bash
# OpenClaw Skill 测试脚本
# 用于验证 searchEverything 的 OpenClaw Skill 集成

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_DIR/target/release/searchEverything"

echo "=============================================="
echo "OpenClaw Skill 测试"
echo "=============================================="
echo ""

# 检查 binary 是否存在
if [ ! -f "$BINARY" ]; then
    echo "正在构建 release 版本..."
    cd "$PROJECT_DIR"
    ~/.cargo/bin/cargo build --release
fi

echo "测试 binary: $BINARY"
echo ""

# 运行 Skill 测试
echo "运行 Skill 测试..."
echo "----------------------------------------------"
"$BINARY" skill-test

# 检查测试结果
if [ $? -eq 0 ]; then
    echo ""
    echo "=============================================="
    echo "✅ 所有 Skill 测试通过！"
    echo "=============================================="
    
    # 显示生成的报告
    if [ -f "$PROJECT_DIR/skill_test_report.json" ]; then
        echo ""
        echo "测试报告已生成：$PROJECT_DIR/skill_test_report.json"
        echo ""
        echo "报告摘要:"
        cat "$PROJECT_DIR/skill_test_report.json" | head -20
    fi
    
    exit 0
else
    echo ""
    echo "=============================================="
    echo "❌ 有测试失败！"
    echo "=============================================="
    exit 1
fi
