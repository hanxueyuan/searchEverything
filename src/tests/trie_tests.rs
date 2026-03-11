/// Trie 索引引擎测试

use crate::file_index::trie::{TrieIndex, FileRecord, TrieNode};
use std::path::PathBuf;
use std::time::SystemTime;

fn create_test_record(name: &str, path: &str) -> FileRecord {
    FileRecord {
        path: PathBuf::from(path),
        name: name.to_string(),
        size: 1024,
        is_dir: false,
        modified: SystemTime::now(),
        created: None,
        id: 0,
    }
}

// TrieNode 测试已移至 TrieIndex 测试中，因为 TrieNode 是内部实现

#[test]
fn test_trie_index_basic() {
    let mut index = TrieIndex::new();
    
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
}

#[test]
fn test_trie_index_add_multiple() {
    let mut index = TrieIndex::new();
    
    for i in 0..100 {
        let name = format!("file{}.txt", i);
        let path = format!("/home/file{}.txt", i);
        index.add(create_test_record(&name, &path));
    }
    
    assert_eq!(index.len(), 100);
    assert_eq!(index.stats.total_files, 100);
}

#[test]
fn test_trie_index_duplicate_add() {
    let mut index = TrieIndex::new();
    
    index.add(create_test_record("file.txt", "/home/file.txt"));
    index.add(create_test_record("file.txt", "/home/file.txt"));
    
    // 重复添加应该更新而不是新增
    assert_eq!(index.len(), 1);
}

#[test]
fn test_trie_wildcard_matching() {
    // 测试各种通配符模式通过 TrieIndex 的 search_glob 方法
    let mut index = TrieIndex::new();
    index.add(create_test_record("file.txt", "/file.txt"));
    index.add(create_test_record("document.txt", "/document.txt"));
    index.add(create_test_record("file.rs", "/file.rs"));
    
    let results = index.search_glob("*.txt");
    assert_eq!(results.len(), 2);
    
    let results = index.search_glob("*.rs");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_trie_case_insensitive() {
    let mut index = TrieIndex::new();
    
    index.add(create_test_record("TestFile.txt", "/home/TestFile.txt"));
    
    // 搜索应该不区分大小写
    let results = index.search_prefix("test");
    assert_eq!(results.len(), 1);
    
    let results = index.search_glob("*.TXT");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_trie_performance() {
    use std::time::Instant;
    
    let mut index = TrieIndex::new();
    
    // 添加大量文件
    let start = Instant::now();
    for i in 0..10000 {
        let name = format!("file_{}.txt", i);
        let path = format!("/home/dir{}/file_{}.txt", i / 100, i);
        index.add(create_test_record(&name, &path));
    }
    let build_time = start.elapsed();
    
    println!("构建 10000 个文件的索引耗时：{:?}", build_time);
    
    // 测试搜索性能
    let start = Instant::now();
    for _ in 0..100 {
        let _ = index.search_prefix("file_5");
    }
    let search_time = start.elapsed();
    
    println!("100 次前缀搜索耗时：{:?}", search_time);
    
    // 搜索应该在合理时间内完成
    assert!(search_time.as_millis() < 1000);
}
