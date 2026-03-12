/// Linux 特定功能测试
use crate::file_index::linux::{get_linux_system_excludes, should_exclude};

#[test]
fn test_exclude_system_paths() {
    let exclude_paths: Vec<String> = vec![];

    // 测试系统目录排除
    assert!(should_exclude("/proc/1", &exclude_paths));
    assert!(should_exclude("/proc/cpuinfo", &exclude_paths));
    assert!(should_exclude("/sys/class", &exclude_paths));
    assert!(should_exclude("/sys/fs", &exclude_paths));
    assert!(should_exclude("/dev/null", &exclude_paths));
    assert!(should_exclude("/dev/sda", &exclude_paths));
    assert!(should_exclude("/run/lock", &exclude_paths));
    assert!(should_exclude("/snap/core", &exclude_paths));
    assert!(should_exclude("/lost+found", &exclude_paths));
}

#[test]
fn test_exclude_custom_paths() {
    let exclude_paths = vec![
        "/home/user/.cache".to_string(),
        "**/node_modules".to_string(),
        "/tmp/*".to_string(),
        "/var/log".to_string(),
    ];

    // 测试精确匹配
    assert!(should_exclude("/home/user/.cache", &exclude_paths));
    assert!(should_exclude("/home/user/.cache/npm", &exclude_paths));

    // 测试 ** 通配符
    assert!(should_exclude("/home/project/node_modules", &exclude_paths));
    assert!(should_exclude(
        "/usr/local/node_modules/lodash",
        &exclude_paths
    ));

    // 测试 * 通配符
    assert!(should_exclude("/tmp/test.txt", &exclude_paths));
    assert!(should_exclude("/tmp/anything", &exclude_paths));

    // 测试不应该排除的路径
    assert!(!should_exclude("/home/user/documents", &exclude_paths));
    assert!(!should_exclude("/home/user/code", &exclude_paths));
}

#[test]
fn test_exclude_case_insensitive() {
    let exclude_paths = vec!["/Home/User".to_string()];

    // 应该不区分大小写
    assert!(should_exclude("/home/user", &exclude_paths));
    assert!(should_exclude("/HOME/USER", &exclude_paths));
}

#[test]
fn test_get_linux_system_excludes() {
    let excludes = get_linux_system_excludes();

    // 验证包含所有必要的系统目录
    assert!(excludes.contains(&"/proc".to_string()));
    assert!(excludes.contains(&"/sys".to_string()));
    assert!(excludes.contains(&"/dev".to_string()));
    assert!(excludes.contains(&"/run".to_string()));
    assert!(excludes.contains(&"/snap".to_string()));
    assert!(excludes.contains(&"/lost+found".to_string()));
    assert!(excludes.contains(&"/tmp".to_string()));
    assert!(excludes.contains(&"/var/tmp".to_string()));

    // 验证不包含用户目录
    assert!(!excludes.contains(&"/home".to_string()));
    assert!(!excludes.contains(&"/opt".to_string()));
}

#[test]
fn test_exclude_edge_cases() {
    let exclude_paths: Vec<String> = vec![];

    // 测试边界情况
    assert!(!should_exclude("/", &exclude_paths));
    assert!(!should_exclude("/home", &exclude_paths));
    assert!(!should_exclude("/usr", &exclude_paths));

    // 测试路径前缀匹配
    assert!(should_exclude("/proc", &exclude_paths));
    assert!(should_exclude("/proc/", &exclude_paths));
    assert!(should_exclude("/sys", &exclude_paths));
    assert!(should_exclude("/dev", &exclude_paths));
}

#[test]
fn test_exclude_complex_patterns() {
    let exclude_paths = vec![
        "**/.git".to_string(),
        "**/__pycache__".to_string(),
        "*.log".to_string(),
    ];

    // 测试深层目录匹配
    assert!(should_exclude("/home/project/.git", &exclude_paths));
    assert!(should_exclude("/home/project/src/.git", &exclude_paths));
    assert!(should_exclude("/usr/local/__pycache__", &exclude_paths));

    // 测试日志文件匹配
    assert!(should_exclude("/var/log/app.log", &exclude_paths));
    assert!(should_exclude("/home/user/test.log", &exclude_paths));
}
