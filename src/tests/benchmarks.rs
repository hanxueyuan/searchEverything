/// 性能基准测试

#[cfg(test)]
mod tests {
    use crate::file_index::trie::{FileRecord, TrieIndex};
    use std::path::PathBuf;
    use std::time::Instant;
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

    #[test]
    fn benchmark_index_build_10k() {
        let mut index = TrieIndex::new();

        let start = Instant::now();
        for i in 0..10_000 {
            let name = format!("file_{}.txt", i);
            let path = format!("/home/dir{}/file_{}.txt", i / 100, i);
            index.add(create_test_record(&name, &path));
        }
        let duration = start.elapsed();

        println!("构建 10,000 个文件索引：{:?}", duration);
        assert!(duration.as_secs() < 10, "索引构建应该小于 10 秒");
    }

    #[test]
    fn benchmark_index_build_100k() {
        let mut index = TrieIndex::new();

        let start = Instant::now();
        for i in 0..100_000 {
            let name = format!("file_{}.txt", i);
            let path = format!("/home/dir{}/file_{}.txt", i / 1000, i);
            index.add(create_test_record(&name, &path));
        }
        let duration = start.elapsed();

        println!("构建 100,000 个文件索引：{:?}", duration);
        // 100k 文件应该在 60 秒内完成
        assert!(duration.as_secs() < 60, "索引构建应该小于 60 秒");
    }

    #[test]
    fn benchmark_prefix_search() {
        let mut index = TrieIndex::new();

        // 构建 10k 文件索引
        for i in 0..10_000 {
            let name = format!("file_{}.txt", i);
            let path = format!("/home/dir{}/file_{}.txt", i / 100, i);
            index.add(create_test_record(&name, &path));
        }

        // 基准测试前缀搜索
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = index.search_prefix("file_5");
        }
        let duration = start.elapsed();

        println!(
            "1000 次前缀搜索：{:?} (平均 {:?} 每次)",
            duration,
            duration / 1000
        );
        // Debug 模式下性能较低，release 模式下会快很多
        assert!(
            duration.as_secs() < 10,
            "1000 次搜索应该小于 10 秒 (debug 模式)"
        );
    }

    #[test]
    fn benchmark_glob_search() {
        let mut index = TrieIndex::new();

        // 构建 10k 文件索引
        for i in 0..10_000 {
            let name = format!("file_{}.txt", i);
            let path = format!("/home/dir{}/file_{}.txt", i / 100, i);
            index.add(create_test_record(&name, &path));
        }

        // 基准测试通配符搜索
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = index.search_glob("*.txt");
        }
        let duration = start.elapsed();

        println!(
            "1000 次通配符搜索：{:?} (平均 {:?} 每次)",
            duration,
            duration / 1000
        );
    }

    #[test]
    fn benchmark_fuzzy_search() {
        let mut index = TrieIndex::new();

        // 构建 10k 文件索引
        for i in 0..10_000 {
            let name = format!("file_{}.txt", i);
            let path = format!("/home/dir{}/file_{}.txt", i / 100, i);
            index.add(create_test_record(&name, &path));
        }

        // 基准测试模糊搜索
        let start = Instant::now();
        for _ in 0..100 {
            let _ = index.search_fuzzy("file_5");
        }
        let duration = start.elapsed();

        println!(
            "100 次模糊搜索：{:?} (平均 {:?} 每次)",
            duration,
            duration / 100
        );
    }

    #[test]
    fn benchmark_add_remove() {
        let mut index = TrieIndex::new();

        // 添加 1000 个文件
        for i in 0..1000 {
            let name = format!("file_{}.txt", i);
            let path = format!("/home/file_{}.txt", i);
            index.add(create_test_record(&name, &path));
        }

        // 基准测试移除操作
        let start = Instant::now();
        for i in 0..1000 {
            let path = format!("/home/file_{}.txt", i);
            index.remove(&PathBuf::from(path));
        }
        let duration = start.elapsed();

        println!(
            "1000 次移除操作：{:?} (平均 {:?} 每次)",
            duration,
            duration / 1000
        );
        assert!(index.is_empty(), "所有文件应该被移除");
    }
}
