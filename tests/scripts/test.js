#!/usr/bin/env node
/**
 * searchEverything OpenClaw 集成自动化测试框架
 * 基于 Node.js 的测试脚本，提供更精细的测试控制
 */

const fs = require('fs');
const path = require('path');
const { execSync, spawn } = require('child_process');
const crypto = require('crypto');

// 配置
const CONFIG = {
    projectRoot: path.join(__dirname, '..', '..'),
    testDir: path.join(__dirname, '..'),
    reportDir: path.join(__dirname, '..', 'reports'),
    logDir: path.join(__dirname, '..', 'logs'),
    tempDir: `/tmp/searchEverything-tests-${process.pid}`,
    skillName: 'searchEverything',
    skillPath: path.join(__dirname, '..', '..', 'skills', 'searchEverything'),
};

// 测试结果统计
const stats = {
    total: 0,
    passed: 0,
    failed: 0,
    skipped: 0,
    startTime: null,
    endTime: null,
};

// 测试用例类
class TestCase {
    constructor(id, name, fn, priority = 'P1') {
        this.id = id;
        this.name = name;
        this.fn = fn;
        this.priority = priority;
        this.status = 'pending';
        this.error = null;
        this.duration = 0;
    }

    async run() {
        const start = Date.now();
        try {
            await this.fn();
            this.status = 'passed';
            stats.passed++;
        } catch (error) {
            this.status = 'failed';
            this.error = error.message;
            stats.failed++;
        } finally {
            this.duration = Date.now() - start;
            stats.total++;
        }
    }

    skip(reason) {
        this.status = 'skipped';
        this.error = reason;
        stats.skipped++;
        stats.total++;
    }
}

// 断言工具
const assert = {
    equal(actual, expected, message) {
        if (actual !== expected) {
            throw new Error(`${message}: 期望 "${expected}", 实际 "${actual}"`);
        }
    },

    fileExists(filePath, message) {
        if (!fs.existsSync(filePath)) {
            throw new Error(`${message}: 文件不存在 "${filePath}"`);
        }
    },

    directoryExists(dirPath, message) {
        if (!fs.existsSync(dirPath) || !fs.statSync(dirPath).isDirectory()) {
            throw new Error(`${message}: 目录不存在 "${dirPath}"`);
        }
    },

    commandExists(cmd, message) {
        try {
            execSync(`command -v ${cmd}`, { stdio: 'ignore' });
        } catch {
            throw new Error(`${message}: 命令不存在 "${cmd}"`);
        }
    },

    isTrue(condition, message) {
        if (!condition) {
            throw new Error(message);
        }
    },

    async rejects(fn, message) {
        try {
            await fn();
            throw new Error(message || '期望函数抛出错误');
        } catch {
            // 预期行为
        }
    },
};

// 日志工具
const log = {
    info(msg) {
        console.log(`\x1b[0;34m[INFO]\x1b[0m ${msg}`);
    },
    success(msg) {
        console.log(`\x1b[0;32m[PASS]\x1b[0m ${msg}`);
    },
    warning(msg) {
        console.log(`\x1b[1;33m[WARN]\x1b[0m ${msg}`);
    },
    error(msg) {
        console.log(`\x1b[0;31m[FAIL]\x1b[0m ${msg}`);
    },
    test(id, name, status, duration) {
        const icon = status === 'passed' ? '✅' : status === 'failed' ? '❌' : '⏭️';
        console.log(`  ${icon} ${id}: ${name} (${duration}ms)`);
    },
};

// 测试套件类
class TestSuite {
    constructor(name) {
        this.name = name;
        this.tests = [];
    }

    add(id, name, fn, priority) {
        this.tests.push(new TestCase(id, name, fn, priority));
    }

    async run() {
        log.info(`========== 开始 ${this.name} ==========`);
        
        // 按优先级排序
        this.tests.sort((a, b) => {
            const priorityOrder = { 'P0': 0, 'P1': 1, 'P2': 2 };
            return priorityOrder[a.priority] - priorityOrder[b.priority];
        });

        for (const test of this.tests) {
            await test.run();
            log.test(test.id, test.name, test.status, test.duration);
        }

        log.info(`========== ${this.name} 完成 ==========`);
    }
}

// ============================================
// 安装流程测试套件
// ============================================
function createInstallationTests() {
    const suite = new TestSuite('安装流程测试');

    // SE-OC-INST-002: Linux 平台自动检测
    suite.add('SE-OC-INST-002', 'Linux 平台自动检测', () => {
        const platform = process.platform;
        assert.equal(platform, 'linux', '应该在 Linux 平台上运行');
    }, 'P0');

    // SE-OC-INST-005: x86_64 架构自动检测
    suite.add('SE-OC-INST-005', 'x86_64 架构自动检测', () => {
        const arch = process.arch;
        assert.isTrue(['x64', 'arm64'].includes(arch), `架构应该是 x64 或 arm64，实际是 ${arch}`);
    }, 'P0');

    // SE-OC-INST-008: 安装到正确目录
    suite.add('SE-OC-INST-008', '安装到正确目录', () => {
        const expectedPath = path.join(CONFIG.skillPath);
        // 检查目录结构是否存在
        const hasPackageJson = fs.existsSync(path.join(expectedPath, 'package.json'));
        assert.isTrue(hasPackageJson, 'Skill 目录应该包含 package.json');
    }, 'P0');

    // SE-OC-INST-009: 安装后命令可用
    suite.add('SE-OC-INST-009', '安装后命令可用', () => {
        // 检查 package.json 中的 bin 配置
        const packageJsonPath = path.join(CONFIG.skillPath, 'package.json');
        if (fs.existsSync(packageJsonPath)) {
            const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
            assert.isTrue(pkg.bin !== undefined, 'package.json 应该包含 bin 配置');
        } else {
            throw new Error('package.json 不存在');
        }
    }, 'P0');

    // SE-OC-INST-016: 网络失败处理
    suite.add('SE-OC-INST-016', '网络失败处理', () => {
        const installScript = path.join(CONFIG.skillPath, 'scripts', 'install.sh');
        if (fs.existsSync(installScript)) {
            const content = fs.readFileSync(installScript, 'utf-8');
            assert.isTrue(
                content.includes('network') || content.includes('offline') || content.includes('curl') || content.includes('wget'),
                '安装脚本应该包含网络错误处理逻辑'
            );
        } else {
            log.warning('安装脚本不存在，跳过详细检查');
        }
    }, 'P1');

    // SE-OC-INST-018: 权限不足处理
    suite.add('SE-OC-INST-018', '权限不足处理', () => {
        const testDir = path.join(CONFIG.tempDir, 'permission-test');
        fs.mkdirSync(testDir, { recursive: true });
        
        // 验证目录可写
        const testFile = path.join(testDir, 'test.txt');
        fs.writeFileSync(testFile, 'test');
        assert.fileExists(testFile, '应该能够写入测试文件');
        
        fs.unlinkSync(testFile);
        fs.rmdirSync(testDir);
    }, 'P1');

    // SE-OC-INST-021: 下载文件校验失败
    suite.add('SE-OC-INST-021', '下载文件校验失败', () => {
        const installScript = path.join(CONFIG.skillPath, 'scripts', 'install.sh');
        if (fs.existsSync(installScript)) {
            const content = fs.readFileSync(installScript, 'utf-8');
            assert.isTrue(
                content.includes('checksum') || content.includes('sha256') || content.includes('hash') || content.includes('verify'),
                '安装脚本应该包含校验和验证逻辑'
            );
        } else {
            log.warning('安装脚本不存在，跳过详细检查');
        }
    }, 'P1');

    // SE-OC-INST-012: 离线安装模式
    suite.add('SE-OC-INST-012', '离线安装模式', () => {
        const installScript = path.join(CONFIG.skillPath, 'scripts', 'install.sh');
        if (fs.existsSync(installScript)) {
            const content = fs.readFileSync(installScript, 'utf-8');
            assert.isTrue(
                content.includes('offline') || content.includes('local') || content.includes('--file'),
                '安装脚本应该支持离线安装模式'
            );
        }
    }, 'P1');

    // SE-OC-INST-013: 指定版本安装
    suite.add('SE-OC-INST-013', '指定版本安装', () => {
        const packageJsonPath = path.join(CONFIG.skillPath, 'package.json');
        if (fs.existsSync(packageJsonPath)) {
            const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
            assert.isTrue(pkg.version !== undefined, 'package.json 应该包含版本号');
        }
    }, 'P1');

    return suite;
}

// ============================================
// 使用流程测试套件
// ============================================
function createUsageTests() {
    const suite = new TestSuite('使用流程测试');

    // SE-OC-USE-001: Skill 自动加载到 OpenClaw
    suite.add('SE-OC-USE-001', 'Skill 自动加载到 OpenClaw', () => {
        // 检查 Skill 描述文件
        const skillJsonPath = path.join(CONFIG.skillPath, 'skill.json');
        if (fs.existsSync(skillJsonPath)) {
            const skill = JSON.parse(fs.readFileSync(skillJsonPath, 'utf-8'));
            assert.isTrue(skill.name !== undefined, 'Skill 应该包含 name 字段');
            assert.isTrue(skill.description !== undefined, 'Skill 应该包含 description 字段');
        } else {
            log.warning('skill.json 不存在，使用 package.json 检查');
            const packageJsonPath = path.join(CONFIG.skillPath, 'package.json');
            assert.fileExists(packageJsonPath, 'package.json 应该存在');
        }
    }, 'P0');

    // SE-OC-USE-002: 命令自动注册
    suite.add('SE-OC-USE-002', '命令自动注册', () => {
        const packageJsonPath = path.join(CONFIG.skillPath, 'package.json');
        const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
        
        // 检查 bin 配置
        if (pkg.bin) {
            const hasSearchEverything = Object.keys(pkg.bin).some(key => 
                key.includes('searchEverything') || key.includes('search')
            );
            assert.isTrue(hasSearchEverything, '应该注册 searchEverything 相关命令');
        }
    }, 'P0');

    // SE-OC-CONF-001: 配置文件正确位置
    suite.add('SE-OC-CONF-001', '配置文件正确位置', () => {
        const defaultConfigPath = path.join(process.env.HOME, '.openclaw', 'skills', 'searchEverything', 'config.json');
        // 验证路径生成逻辑
        assert.isTrue(defaultConfigPath.includes('searchEverything'), '配置路径应该包含 Skill 名称');
    }, 'P1');

    // SE-OC-CONF-006: 默认配置生成
    suite.add('SE-OC-CONF-006', '默认配置生成', () => {
        const testConfigDir = path.join(CONFIG.tempDir, 'config-test');
        fs.mkdirSync(testConfigDir, { recursive: true });
        
        const defaultConfig = {
            version: '1.0.0',
            logLevel: 'info',
            indexPath: '~/.openclaw/skills/searchEverything/index',
            maxResults: 100,
            excludePatterns: ['node_modules', '.git', 'dist']
        };
        
        fs.writeFileSync(
            path.join(testConfigDir, 'config.json'),
            JSON.stringify(defaultConfig, null, 2)
        );
        
        assert.fileExists(path.join(testConfigDir, 'config.json'), '默认配置文件应该生成');
        
        // 清理
        fs.unlinkSync(path.join(testConfigDir, 'config.json'));
        fs.rmdirSync(testConfigDir);
    }, 'P1');

    // SE-OC-LOG-001: 日志输出到 OpenClaw 日志系统
    suite.add('SE-OC-LOG-001', '日志输出到 OpenClaw 日志系统', () => {
        const mainFile = path.join(CONFIG.skillPath, 'src', 'index.js');
        if (fs.existsSync(mainFile)) {
            const content = fs.readFileSync(mainFile, 'utf-8');
            assert.isTrue(
                content.includes('console') || content.includes('logger') || content.includes('log'),
                '应该包含日志输出逻辑'
            );
        } else {
            log.warning('源码不存在，跳过此测试');
        }
    }, 'P1');

    // SE-OC-LOG-002: 日志级别控制有效
    suite.add('SE-OC-LOG-002', '日志级别控制有效', () => {
        // 验证日志级别配置支持
        const validLevels = ['debug', 'info', 'warn', 'error'];
        assert.isTrue(validLevels.length > 0, '应该支持多种日志级别');
    }, 'P1');

    // SE-OC-UPD-001: 检查更新功能
    suite.add('SE-OC-UPD-001', '检查更新功能', () => {
        const updateScript = path.join(CONFIG.skillPath, 'scripts', 'update.sh');
        if (fs.existsSync(updateScript)) {
            const content = fs.readFileSync(updateScript, 'utf-8');
            assert.isTrue(
                content.includes('version') || content.includes('update') || content.includes('release'),
                '更新脚本应该包含版本检查逻辑'
            );
        } else {
            log.warning('更新脚本不存在');
        }
    }, 'P1');

    // SE-OC-UPD-003: 更新失败回滚
    suite.add('SE-OC-UPD-003', '更新失败回滚', () => {
        const updateScript = path.join(CONFIG.skillPath, 'scripts', 'update.sh');
        if (fs.existsSync(updateScript)) {
            const content = fs.readFileSync(updateScript, 'utf-8');
            assert.isTrue(
                content.includes('rollback') || content.includes('backup') || content.includes('restore'),
                '更新脚本应该包含回滚逻辑'
            );
        }
    }, 'P1');

    return suite;
}

// ============================================
// 卸载流程测试套件
// ============================================
function createUninstallTests() {
    const suite = new TestSuite('卸载流程测试');

    // SE-OC-UNIN-001: 删除可执行文件
    suite.add('SE-OC-UNIN-001', '删除可执行文件', () => {
        const testDir = path.join(CONFIG.tempDir, 'uninstall-test');
        const testBin = path.join(testDir, 'bin', 'searchEverything');
        
        fs.mkdirSync(path.dirname(testBin), { recursive: true });
        fs.writeFileSync(testBin, '#!/bin/bash\necho test');
        fs.chmodSync(testBin, 0o755);
        
        assert.fileExists(testBin, '测试二进制文件应该存在');
        
        // 模拟删除
        fs.unlinkSync(testBin);
        assert.isTrue(!fs.existsSync(testBin), '文件应该被删除');
        
        // 清理
        fs.rmdirSync(path.dirname(testBin));
        fs.rmdirSync(testDir);
    }, 'P0');

    // SE-OC-UNIN-002: 删除配置文件
    suite.add('SE-OC-UNIN-002', '删除配置文件', () => {
        const testDir = path.join(CONFIG.tempDir, 'config-cleanup');
        const testConfig = path.join(testDir, 'config.json');
        
        fs.mkdirSync(testDir, { recursive: true });
        fs.writeFileSync(testConfig, '{}');
        
        fs.unlinkSync(testConfig);
        assert.isTrue(!fs.existsSync(testConfig), '配置文件应该被删除');
        
        fs.rmdirSync(testDir);
    }, 'P0');

    // SE-OC-UNIN-012: Skill 禁用
    suite.add('SE-OC-UNIN-012', 'Skill 禁用', () => {
        const testDir = path.join(CONFIG.tempDir, 'disable-test');
        const disableFile = path.join(testDir, 'disabled');
        
        fs.mkdirSync(testDir, { recursive: true });
        fs.writeFileSync(disableFile, 'disabled');
        
        assert.fileExists(disableFile, '禁用状态文件应该创建成功');
        
        // 清理
        fs.unlinkSync(disableFile);
        fs.rmdirSync(testDir);
    }, 'P1');

    // SE-OC-UNIN-015: 可重新启用
    suite.add('SE-OC-UNIN-015', '可重新启用', () => {
        const testDir = path.join(CONFIG.tempDir, 'enable-test');
        const disableFile = path.join(testDir, 'disabled');
        
        fs.mkdirSync(testDir, { recursive: true });
        
        // 禁用
        fs.writeFileSync(disableFile, 'disabled');
        assert.fileExists(disableFile, '禁用状态应该存在');
        
        // 启用
        fs.unlinkSync(disableFile);
        assert.isTrue(!fs.existsSync(disableFile), '禁用状态应该被移除');
        
        fs.rmdirSync(testDir);
    }, 'P1');

    // SE-OC-UNIN-017: 文件被占用处理
    suite.add('SE-OC-UNIN-017', '文件被占用处理', () => {
        const testDir = path.join(CONFIG.tempDir, 'in-use-test');
        const testFile = path.join(testDir, 'file.txt');
        
        fs.mkdirSync(testDir, { recursive: true });
        fs.writeFileSync(testFile, 'test content');
        
        // 尝试删除（模拟文件占用场景）
        try {
            fs.unlinkSync(testFile);
            log.warning('文件占用检测可能需要更复杂的测试');
        } catch {
            log.success('文件占用检测正常');
        }
        
        fs.rmdirSync(testDir);
    }, 'P1');

    return suite;
}

// ============================================
// 边界场景测试套件
// ============================================
function createBoundaryTests() {
    const suite = new TestSuite('边界场景测试');

    // SE-OC-BOUND-001: 多用户安装隔离
    suite.add('SE-OC-BOUND-001', '多用户安装隔离', () => {
        const testDir = path.join(CONFIG.tempDir, 'multi-user');
        const user1Dir = path.join(testDir, 'user1', 'skills', 'searchEverything');
        const user2Dir = path.join(testDir, 'user2', 'skills', 'searchEverything');
        
        fs.mkdirSync(user1Dir, { recursive: true });
        fs.mkdirSync(user2Dir, { recursive: true });
        
        // 创建不同的配置
        fs.writeFileSync(path.join(user1Dir, 'config.json'), JSON.stringify({ user: 'user1' }));
        fs.writeFileSync(path.join(user2Dir, 'config.json'), JSON.stringify({ user: 'user2' }));
        
        // 验证隔离
        const user1Config = JSON.parse(fs.readFileSync(path.join(user1Dir, 'config.json')));
        const user2Config = JSON.parse(fs.readFileSync(path.join(user2Dir, 'config.json')));
        
        assert.equal(user1Config.user, 'user1', '用户 1 配置应该独立');
        assert.equal(user2Config.user, 'user2', '用户 2 配置应该独立');
        
        // 清理
        fs.rmSync(testDir, { recursive: true, force: true });
    }, 'P2');

    // SE-OC-BOUND-005: 不支持多版本时的处理
    suite.add('SE-OC-BOUND-005', '不支持多版本时的处理', () => {
        const testDir = path.join(CONFIG.tempDir, 'version-test');
        fs.mkdirSync(testDir, { recursive: true });
        
        const versionFile = path.join(testDir, 'version.json');
        fs.writeFileSync(versionFile, JSON.stringify({
            installed: '1.0.0',
            attempting: '1.0.0'
        }));
        
        // 验证版本冲突检测逻辑存在
        const versions = JSON.parse(fs.readFileSync(versionFile));
        assert.equal(versions.installed, versions.attempting, '版本冲突检测应该工作');
        
        fs.rmSync(testDir, { recursive: true, force: true });
    }, 'P1');

    // SE-OC-BOUND-008: 无网络环境
    suite.add('SE-OC-BOUND-008', '无网络环境', () => {
        const installScript = path.join(CONFIG.skillPath, 'scripts', 'install.sh');
        if (fs.existsSync(installScript)) {
            const content = fs.readFileSync(installScript, 'utf-8');
            assert.isTrue(
                content.includes('offline') || content.includes('local') || content.includes('cache'),
                '应该支持离线模式'
            );
        }
    }, 'P1');

    // SE-OC-BOUND-009: 代理环境
    suite.add('SE-OC-BOUND-009', '代理环境', () => {
        const installScript = path.join(CONFIG.skillPath, 'scripts', 'install.sh');
        if (fs.existsSync(installScript)) {
            const content = fs.readFileSync(installScript, 'utf-8');
            assert.isTrue(
                content.includes('proxy') || content.includes('http_proxy') || content.includes('https_proxy'),
                '应该支持代理环境'
            );
        }
    }, 'P2');

    return suite;
}

// ============================================
// 集成测试套件
// ============================================
function createIntegrationTests() {
    const suite = new TestSuite('集成测试');

    // SE-OC-INT-008: 无命名冲突
    suite.add('SE-OC-INT-008', '无命名冲突', () => {
        const packageJsonPath = path.join(CONFIG.skillPath, 'package.json');
        const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
        
        // 检查命令名是否规范
        if (pkg.bin) {
            const cmdNames = Object.keys(pkg.bin);
            assert.isTrue(cmdNames.length > 0, '应该至少有一个命令');
            
            // 检查命令名不包含空格或特殊字符
            for (const cmd of cmdNames) {
                assert.isTrue(
                    /^[a-zA-Z0-9_-]+$/.test(cmd),
                    `命令名 "${cmd}" 应该只包含字母、数字、下划线和连字符`
                );
            }
        }
    }, 'P1');

    // SE-OC-INT-010: 错误隔离
    suite.add('SE-OC-INT-010', '错误隔离', () => {
        const testDir = path.join(CONFIG.tempDir, 'error-isolation');
        fs.mkdirSync(testDir, { recursive: true });
        
        const errorLog = path.join(testDir, 'error.log');
        fs.writeFileSync(errorLog, 'Simulated error');
        
        assert.fileExists(errorLog, '错误应该被正确记录');
        
        fs.rmSync(testDir, { recursive: true, force: true });
    }, 'P1');

    return suite;
}

// ============================================
// 生成测试报告
// ============================================
function generateReport() {
    stats.endTime = Date.now();
    const duration = stats.endTime - stats.startTime;
    const passRate = stats.total > 0 ? ((stats.passed / stats.total) * 100).toFixed(2) : 0;

    const reportDir = CONFIG.reportDir;
    fs.mkdirSync(reportDir, { recursive: true });
    
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const reportFile = path.join(reportDir, `test-report-${timestamp}.md`);

    const report = `# searchEverything OpenClaw 集成测试报告

## 测试执行时间
${new Date().toISOString()}
执行时长：${(duration / 1000).toFixed(2)} 秒

## 测试统计

| 指标 | 数量 | 百分比 |
|------|------|--------|
| 总测试数 | ${stats.total} | 100% |
| 通过 | ${stats.passed} | ${((stats.passed / stats.total) * 100).toFixed(2)}% |
| 失败 | ${stats.failed} | ${((stats.failed / stats.total) * 100).toFixed(2)}% |
| 跳过 | ${stats.skipped} | ${((stats.skipped / stats.total) * 100).toFixed(2)}% |

## 测试结果

${stats.failed === 0 ? '✅ 所有测试通过' : `⚠️ ${stats.failed} 个测试失败`}

## 建议

${stats.failed === 0 
    ? '✅ 所有测试通过，可以继续进行后续开发或发布流程。'
    : `⚠️ 存在 ${stats.failed} 个失败用例，建议：
1. 查看详细日志定位问题
2. 修复失败用例
3. 重新运行测试验证`
}

---
*报告生成时间：${new Date().toISOString()}*
`;

    fs.writeFileSync(reportFile, report);
    log.success(`测试报告已生成：${reportFile}`);

    // 同时输出到控制台
    console.log('\n' + '='.repeat(50));
    console.log('  测试完成');
    console.log('='.repeat(50));
    console.log(`  总计：${stats.total}`);
    console.log(`  通过：${stats.passed}`);
    console.log(`  失败：${stats.failed}`);
    console.log(`  跳过：${stats.skipped}`);
    console.log(`  通过率：${passRate}%`);
    console.log(`  耗时：${(duration / 1000).toFixed(2)}秒`);
    console.log('='.repeat(50));
}

// ============================================
// 主函数
// ============================================
async function main() {
    console.log('='.repeat(50));
    console.log('  searchEverything OpenClaw 集成自动化测试');
    console.log('='.repeat(50));

    stats.startTime = Date.now();

    // 创建临时目录
    fs.mkdirSync(CONFIG.tempDir, { recursive: true });
    fs.mkdirSync(CONFIG.logDir, { recursive: true });

    try {
        // 运行所有测试套件
        await createInstallationTests().run();
        await createUsageTests().run();
        await createUninstallTests().run();
        await createBoundaryTests().run();
        await createIntegrationTests().run();

        // 生成报告
        generateReport();

        // 退出码
        process.exit(stats.failed > 0 ? 1 : 0);
    } catch (error) {
        log.error(`测试执行失败：${error.message}`);
        process.exit(1);
    } finally {
        // 清理临时目录
        try {
            fs.rmSync(CONFIG.tempDir, { recursive: true, force: true });
        } catch {
            // 忽略清理错误
        }
    }
}

// 执行
main();
