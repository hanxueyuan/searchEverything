/// 搜索功能测试
use crate::file_index::trie::{FileRecord, TrieIndex};
use std::path::PathBuf;
use std::time::SystemTime;

fn create_test_record(name: &str, path: &str, size: u64) -> FileRecord {
    FileRecord {
        path: PathBuf::from(path),
        name: name.to_string(),
        size,
        is_dir: false,
        modified: SystemTime::now(),
        created: None,
        id: 0,
    }
}

#[test]
fn test_trie_prefix_search() {
    let mut index = TrieIndex::new();

    index.add(create_test_record("test.rs", "/home/test.rs", 1024));
    index.add(create_test_record("testing.py", "/home/testing.py", 2048));
    index.add(create_test_record("best.rs", "/home/best.rs", 512));
    index.add(create_test_record(
        "document.txt",
        "/home/document.txt",
        4096,
    ));

    // 前缀搜索
    let results = index.search_prefix("test");
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.name == "test.rs"));
    assert!(results.iter().any(|r| r.name == "testing.py"));
}

#[test]
fn test_trie_glob_search() {
    let mut index = TrieIndex::new();

    index.add(create_test_record("file1.rs", "/home/file1.rs", 1024));
    index.add(create_test_record("file2.rs", "/home/file2.rs", 2048));
    index.add(create_test_record("file.txt", "/home/file.txt", 512));
    index.add(create_test_record("data.json", "/home/data.json", 4096));

    // *.rs 搜索
    let results = index.search_glob("*.rs");
    assert_eq!(results.len(), 2);

    // file* 搜索
    let results = index.search_glob("file*");
    assert_eq!(results.len(), 3);

    // *.txt 搜索
    let results = index.search_glob("*.txt");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_trie_regex_search() {
    let mut index = TrieIndex::new();

    index.add(create_test_record("test.rs", "/home/test.rs", 1024));
    index.add(create_test_record("config.yaml", "/home/config.yaml", 2048));
    index.add(create_test_record("data.json", "/home/data.json", 512));

    let regex = regex::Regex::new(r".*\.(rs|yaml)$").unwrap();
    let results = index.search_regex(&regex);

    assert_eq!(results.len(), 2);
}

#[test]
fn test_trie_fuzzy_search() {
    let mut index = TrieIndex::new();

    index.add(create_test_record(
        "configuration.yaml",
        "/home/configuration.yaml",
        1024,
    ));
    index.add(create_test_record("config.json", "/home/config.json", 2048));
    index.add(create_test_record("readme.md", "/home/readme.md", 512));

    // 模糊搜索 "config"
    let results = index.search_fuzzy("config");

    assert!(!results.is_empty());
    // configuration 和 config.json 应该都匹配
    assert!(results.iter().any(|r| r.name.contains("config")));
}

#[test]
fn test_trie_add_remove() {
    let mut index = TrieIndex::new();

    index.add(create_test_record("file1.txt", "/home/file1.txt", 1024));
    index.add(create_test_record("file2.txt", "/home/file2.txt", 2048));

    assert_eq!(index.len(), 2);

    // 移除文件
    index.remove(&PathBuf::from("/home/file1.txt"));

    assert_eq!(index.len(), 1);
    assert!(index
        .get_by_path(&PathBuf::from("/home/file1.txt"))
        .is_none());
    assert!(index
        .get_by_path(&PathBuf::from("/home/file2.txt"))
        .is_some());
}

#[test]
fn test_trie_stats() {
    let mut index = TrieIndex::new();

    index.add(create_test_record("file1.txt", "/home/file1.txt", 1024));
    index.add(create_test_record("file2.txt", "/home/file2.txt", 2048));

    assert!(index.stats.total_files >= 2);
}

#[test]
fn test_relevance_score() {
    // 测试相关度排序
    let mut index = TrieIndex::new();

    index.add(create_test_record("test", "/home/test", 1024));
    index.add(create_test_record("testing", "/home/testing", 2048));
    index.add(create_test_record("attest", "/home/attest", 512));
    index.add(create_test_record("other", "/home/other", 4096));

    let results = index.search_fuzzy("test");

    // "test" 应该排在最前面（完全匹配）
    assert!(!results.is_empty());
    assert_eq!(results[0].name, "test");
}
