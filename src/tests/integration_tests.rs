use crate::commands::index::IndexConfig;
/// 综合功能测试 - 端到端场景
use crate::file_index::trie::{FileRecord, TrieIndex};
use std::path::PathBuf;
use std::time::SystemTime;

fn create_test_record(name: &str, path: &str, size: u64, is_dir: bool) -> FileRecord {
    FileRecord {
        path: PathBuf::from(path),
        name: name.to_string(),
        size,
        is_dir,
        modified: SystemTime::now(),
        created: None,
        id: 0,
    }
}

// ============================================
// 索引管理器测试
// ============================================

#[test]
fn test_index_manager_basic() {
    use crate::file_index::IndexManager;

    let manager = IndexManager::new().expect("Failed to create IndexManager");

    // 初始状态应该没有索引
    assert!(manager.get_index().is_none());

    // 获取统计信息应该返回 None
    assert!(manager.get_stats().is_none());
}

#[test]
fn test_index_manager_with_persistence() {
    use crate::file_index::IndexManager;
    use tempfile::tempdir;

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let index_path = temp_dir.path().join("test-index");

    let manager = IndexManager::with_persistence(index_path, 300)
        .expect("Failed to create IndexManager with persistence");

    assert!(manager.get_index().is_none());
    // supports_realtime can be either true or false depending on platform
    let _ = manager.supports_realtime();
}

// ============================================
// 文件记录测试
// ============================================

#[test]
fn test_file_record_creation() {
    let record = create_test_record("test.txt", "/home/test.txt", 1024, false);

    assert_eq!(record.name, "test.txt");
    assert_eq!(record.path, PathBuf::from("/home/test.txt"));
    assert_eq!(record.size, 1024);
    assert!(!record.is_dir);
}

#[test]
fn test_file_record_directory() {
    let record = create_test_record("documents", "/home/documents", 0, true);

    assert_eq!(record.name, "documents");
    assert!(record.is_dir);
}

// ============================================
// 索引配置测试
// ============================================

#[test]
fn test_index_config_default() {
    let config = IndexConfig::auto_init();

    assert!(config.initialized);
    assert!(!config.indexed_paths.is_empty() || cfg!(not(target_os = "linux")));
}

#[test]
fn test_index_config_serialization() {
    let mut config = IndexConfig::auto_init();
    config.indexed_paths = vec!["/test/path".to_string()];
    config.excluded_paths = vec!["/test/exclude".to_string()];

    // 序列化
    let serialized = serde_yaml::to_string(&config).expect("Failed to serialize");
    assert!(!serialized.is_empty());

    // 反序列化
    let deserialized: IndexConfig =
        serde_yaml::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(deserialized.indexed_paths, config.indexed_paths);
    assert_eq!(deserialized.excluded_paths, config.excluded_paths);
}

// ============================================
// 搜索场景测试
// ============================================

#[test]
fn test_search_scenario_code_files() {
    let mut index = TrieIndex::new();

    // 添加代码文件
    index.add(create_test_record(
        "main.rs",
        "/project/src/main.rs",
        2048,
        false,
    ));
    index.add(create_test_record(
        "lib.rs",
        "/project/src/lib.rs",
        1536,
        false,
    ));
    index.add(create_test_record(
        "utils.rs",
        "/project/src/utils.rs",
        1024,
        false,
    ));
    index.add(create_test_record(
        "config.yaml",
        "/project/config.yaml",
        512,
        false,
    ));
    index.add(create_test_record(
        "Cargo.toml",
        "/project/Cargo.toml",
        256,
        false,
    ));

    // 搜索 Rust 文件
    let results = index.search_glob("*.rs");
    assert_eq!(results.len(), 3);

    // 搜索配置文件
    let results = index.search_glob("*.yaml");
    assert_eq!(results.len(), 1);

    // 前缀搜索
    let results = index.search_prefix("main");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "main.rs");
}

#[test]
fn test_search_scenario_documents() {
    let mut index = TrieIndex::new();

    // 添加文档文件
    index.add(create_test_record(
        "report.pdf",
        "/docs/report.pdf",
        102400,
        false,
    ));
    index.add(create_test_record(
        "notes.txt",
        "/docs/notes.txt",
        2048,
        false,
    ));
    index.add(create_test_record(
        "readme.md",
        "/docs/readme.md",
        4096,
        false,
    ));
    index.add(create_test_record(
        "spec.docx",
        "/docs/spec.docx",
        51200,
        false,
    ));

    // 搜索 markdown 文件
    let results = index.search_glob("*.md");
    assert_eq!(results.len(), 1);

    // 模糊搜索 "doc"
    let results = index.search_fuzzy("doc");
    assert!(results.iter().any(|r| r.name.contains("doc")));
}

#[test]
fn test_search_scenario_large_index() {
    let mut index = TrieIndex::new();

    // 模拟大型项目
    for i in 0..1000 {
        let dir = format!("/project/src/module_{}", i / 10);
        let name = format!("file_{}.rs", i);
        let path = format!("{}/{}", dir, name);
        index.add(create_test_record(&name, &path, 1024, false));
    }

    // 搜索应该仍然快速
    let start = std::time::Instant::now();
    let results = index.search_prefix("file_5");
    let elapsed = start.elapsed();

    assert!(!results.is_empty());
    assert!(
        elapsed.as_millis() < 100,
        "Search took too long: {:?}",
        elapsed
    );
}

// ============================================
// 边界情况测试
// ============================================

#[test]
fn test_empty_index() {
    let index = TrieIndex::new();

    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
    assert_eq!(index.search_prefix("anything").len(), 0);
    assert_eq!(index.search_glob("*.txt").len(), 0);
    assert_eq!(index.search_fuzzy("test").len(), 0);
}

#[test]
fn test_special_characters_in_names() {
    let mut index = TrieIndex::new();

    index.add(create_test_record(
        "file-with-dash.txt",
        "/home/file-with-dash.txt",
        1024,
        false,
    ));
    index.add(create_test_record(
        "file_with_underscore.txt",
        "/home/file_with_underscore.txt",
        1024,
        false,
    ));
    index.add(create_test_record(
        "file with space.txt",
        "/home/file with space.txt",
        1024,
        false,
    ));
    index.add(create_test_record(
        "文件.txt",
        "/home/文件.txt",
        1024,
        false,
    ));

    // 搜索应该能处理特殊字符
    let results = index.search_glob("*.txt");
    assert_eq!(results.len(), 4);

    let results = index.search_prefix("file");
    assert!(results.len() >= 3);
}

#[test]
fn test_deep_directory_structure() {
    let mut index = TrieIndex::new();

    // 创建深层目录结构
    index.add(create_test_record(
        "deep.txt",
        "/a/b/c/d/e/f/g/deep.txt",
        1024,
        false,
    ));
    index.add(create_test_record(
        "shallow.txt",
        "/shallow.txt",
        1024,
        false,
    ));

    let results = index.search_glob("*.txt");
    assert_eq!(results.len(), 2);

    let results = index.search_prefix("deep");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_duplicate_handling() {
    let mut index = TrieIndex::new();

    // 添加相同路径的文件
    index.add(create_test_record(
        "file.txt",
        "/home/file.txt",
        1024,
        false,
    ));
    index.add(create_test_record(
        "file.txt",
        "/home/file.txt",
        2048,
        false,
    )); // 不同大小
    index.add(create_test_record(
        "file.txt",
        "/home/file.txt",
        4096,
        false,
    )); // 再次更新

    // 应该只有 1 个条目（更新而不是新增）
    assert_eq!(index.len(), 1);

    let record = index.get_by_path(&PathBuf::from("/home/file.txt")).unwrap();
    assert_eq!(record.size, 4096); // 应该是最后一次更新的值
}

// ============================================
// 性能基准测试
// ============================================

#[test]
fn test_index_build_performance() {
    use std::time::Instant;

    let mut index = TrieIndex::new();

    let start = Instant::now();
    for i in 0..5000 {
        let name = format!("file_{}.txt", i);
        let path = format!("/home/dir{}/file_{}.txt", i / 100, i);
        index.add(create_test_record(&name, &path, 1024, false));
    }
    let build_time = start.elapsed();

    println!("构建 5000 个文件的索引耗时：{:?}", build_time);

    // 构建应该在合理时间内完成（< 5 秒）
    assert!(
        build_time.as_secs() < 5,
        "Index build took too long: {:?}",
        build_time
    );
}

#[test]
fn test_search_performance_various_patterns() {
    use std::time::Instant;

    let mut index = TrieIndex::new();

    // 构建索引
    for i in 0..10000 {
        let name = format!("file_{:05}.txt", i);
        let path = format!("/home/dir{}/{}", i / 100, name);
        index.add(create_test_record(&name, &path, 1024, false));
    }

    // 测试不同搜索模式的性能
    let patterns = vec![
        ("file_5", "prefix"),
        ("*.txt", "glob"),
        ("file_.*\\.txt", "regex"),
        ("file", "fuzzy"),
    ];

    for (pattern, search_type) in patterns {
        let start = Instant::now();
        let results = match search_type {
            "prefix" => index.search_prefix(pattern),
            "glob" => index.search_glob(pattern),
            "regex" => {
                let regex = regex::Regex::new(pattern).unwrap();
                index.search_regex(&regex)
            }
            "fuzzy" => index.search_fuzzy(pattern),
            _ => vec![],
        };
        let elapsed = start.elapsed();

        println!(
            "{} search '{}' returned {} results in {:?}",
            search_type,
            pattern,
            results.len(),
            elapsed
        );

        // 搜索应该在合理时间内完成（正则搜索允许更长时间）
        let max_time = if search_type == "regex" { 200 } else { 100 };
        assert!(
            elapsed.as_millis() < max_time,
            "{} search took too long: {:?}",
            search_type,
            elapsed
        );
    }
}

// ============================================
// 统计信息测试
// ============================================

#[test]
fn test_index_stats_accuracy() {
    let mut index = TrieIndex::new();

    // 添加不同大小的文件
    index.add(create_test_record("small.txt", "/small.txt", 100, false));
    index.add(create_test_record("medium.txt", "/medium.txt", 1000, false));
    index.add(create_test_record("large.txt", "/large.txt", 10000, false));

    let stats = index.stats.clone();

    assert!(stats.total_files >= 3);
    // 验证统计信息存在（index_size_mb 只在 build 时设置，这里只验证基本统计）
    assert_eq!(stats.total_files, 3);
    assert_eq!(stats.total_dirs, 0);
}
