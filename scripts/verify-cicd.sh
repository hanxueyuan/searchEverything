#!/bin/bash
# CI/CD 配置验证脚本
# 用于在本地验证 GitHub Actions 配置

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 检查计数
PASSED=0
FAILED=0
WARNINGS=0

pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    PASSED=$((PASSED + 1))
}

fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    FAILED=$((FAILED + 1))
}

warn() {
    echo -e "${YELLOW}⚠️  WARN${NC}: $1"
    WARNINGS=$((WARNINGS + 1))
}

# 验证 YAML 文件
validate_yaml() {
    local file=$1
    if python3 -c "import yaml; yaml.safe_load(open('$file'))" 2>/dev/null; then
        pass "YAML 语法有效：$file"
    else
        fail "YAML 语法无效：$file"
    fi
}

echo "========================================"
echo "  searchEverything CI/CD 验证脚本"
echo "========================================"
echo ""

cd "$(dirname "$0")/.."

# 检查 Python
echo "检查 Python 环境..."
if command -v python3 &> /dev/null; then
    pass "Python3 已安装"
else
    fail "Python3 未安装"
fi

# 验证工作流文件
echo ""
echo "验证 GitHub Actions 工作流..."

workflows=(
    ".github/workflows/ci.yml"
    ".github/workflows/coverage.yml"
    ".github/workflows/release.yml"
    ".github/workflows/pr-validation.yml"
)

for workflow in "${workflows[@]}"; do
    if [ -f "$workflow" ]; then
        pass "工作流文件存在：$workflow"
        validate_yaml "$workflow"
    else
        fail "工作流文件缺失：$workflow"
    fi
done

# 检查测试文件
echo ""
echo "检查测试文件..."

tests=(
    "src/tests/index_tests.rs"
    "src/tests/search_tests.rs"
    "src/tests/trie_tests.rs"
    "src/tests/linux_tests.rs"
    "src/tests/integration_tests.rs"
)

for test in "${tests[@]}"; do
    if [ -f "$test" ]; then
        pass "测试文件存在：$test"
    else
        fail "测试文件缺失：$test"
    fi
done

# 检查 Cargo 配置
echo ""
echo "检查 Cargo 配置..."

if [ -f "Cargo.toml" ]; then
    pass "Cargo.toml 存在"
    
    if grep -q "dev-dependencies" Cargo.toml; then
        pass "dev-dependencies 已配置"
    fi
    
    if grep -q "assert_cmd" Cargo.toml; then
        pass "assert_cmd 测试依赖已配置"
    fi
    
    if grep -q "tempfile" Cargo.toml; then
        pass "tempfile 测试依赖已配置"
    fi
else
    fail "Cargo.toml 缺失"
fi

# 检查覆盖率配置
echo ""
echo "检查覆盖率配置..."

if grep -q "cargo-tarpaulin" .github/workflows/coverage.yml; then
    pass "cargo-tarpaulin 已配置"
fi

if grep -q "THRESHOLD=80" .github/workflows/coverage.yml; then
    pass "覆盖率阈值已设置 (80%)"
fi

# 检查发布配置
echo ""
echo "检查发布配置..."

if grep -q "push:" .github/workflows/release.yml && grep -q "tags:" .github/workflows/release.yml; then
    pass "版本标签触发已配置"
fi

if grep -q "softprops/action-gh-release" .github/workflows/release.yml; then
    pass "GitHub Release 上传已配置"
fi

# 检查文档
echo ""
echo "检查文档..."

if [ -f ".github/CI_CD.md" ]; then
    pass "CI/CD 文档存在"
else
    warn "CI/CD 文档缺失"
fi

# 打印总结
echo ""
echo "========================================"
echo "  验证总结"
echo "========================================"
echo -e "${GREEN}通过${NC}: $PASSED"
echo -e "${RED}失败${NC}: $FAILED"
echo -e "${YELLOW}警告${NC}: $WARNINGS"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}❌ 验证失败${NC}"
    exit 1
else
    echo -e "${GREEN}✅ 验证通过${NC}"
    exit 0
fi
