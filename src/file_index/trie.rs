use serde::{Deserialize, Serialize};
/// Trie 树索引引擎
///
/// 使用高效的 Trie 树结构实现文件名索引：
/// - 前缀搜索 O(m) 时间复杂度
/// - 支持自动补全
/// - 内存效率高（共享前缀）
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 文件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    /// 完整路径
    pub path: PathBuf,
    /// 文件名
    pub name: String,
    /// 文件大小
    pub size: u64,
    /// 是否目录
    pub is_dir: bool,
    /// 修改时间
    pub modified: SystemTime,
    /// 创建时间
    pub created: Option<SystemTime>,
    /// 文件 ID（用于索引引用）
    pub id: usize,
}

/// Trie 树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrieNode {
    /// 子节点
    children: HashMap<char, Box<TrieNode>>,
    /// 以此节点结尾的文件 ID 列表
    file_ids: Vec<usize>,
    /// 是否是某个文件名的结尾
    is_end: bool,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            file_ids: Vec::new(),
            is_end: false,
        }
    }

    /// 插入文件名
    fn insert(&mut self, name: &str, file_id: usize) {
        let chars: Vec<char> = name.chars().collect();
        self.insert_chars(&chars, file_id);
    }

    fn insert_chars(&mut self, chars: &[char], file_id: usize) {
        if chars.is_empty() {
            self.is_end = true;
            if !self.file_ids.contains(&file_id) {
                self.file_ids.push(file_id);
            }
            return;
        }

        let ch = chars[0];
        let node = self
            .children
            .entry(ch)
            .or_insert_with(|| Box::new(TrieNode::new()));
        node.insert_chars(&chars[1..], file_id);
    }

    /// 搜索前缀匹配
    fn search_prefix(&self, prefix: &str) -> Vec<usize> {
        let chars: Vec<char> = prefix.chars().collect();
        match self.find_node(&chars) {
            Some(node) => node.collect_all_file_ids(),
            None => Vec::new(),
        }
    }

    /// 查找节点
    fn find_node(&self, chars: &[char]) -> Option<&TrieNode> {
        if chars.is_empty() {
            return Some(self);
        }

        let ch = chars[0];
        self.children
            .get(&ch)
            .and_then(|node| node.find_node(&chars[1..]))
    }

    /// 收集所有子节点的文件 ID
    fn collect_all_file_ids(&self) -> Vec<usize> {
        let mut ids = self.file_ids.clone();

        for child in self.children.values() {
            ids.extend(child.collect_all_file_ids());
        }

        ids
    }

    /// 删除文件 ID
    fn remove_file_id(&mut self, file_id: usize, chars: &[char]) -> bool {
        if chars.is_empty() {
            if let Some(pos) = self.file_ids.iter().position(|&id| id == file_id) {
                self.file_ids.remove(pos);
                return true;
            }
            return false;
        }

        let ch = chars[0];
        if let Some(child) = self.children.get_mut(&ch) {
            return child.remove_file_id(file_id, &chars[1..]);
        }
        false
    }
}

/// Trie 索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrieIndex {
    /// Trie 树根节点
    root: TrieNode,
    /// 文件记录存储（按 ID 索引）
    files: HashMap<usize, FileRecord>,
    /// 路径到 ID 的映射
    path_to_id: HashMap<String, usize>,
    /// 下一个文件 ID
    next_id: usize,
    /// 统计信息
    pub stats: IndexStats,
}

/// 索引统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexStats {
    /// 文件总数
    pub total_files: usize,
    /// 目录总数
    pub total_dirs: usize,
    /// 索引大小 (MB)
    pub index_size_mb: f64,
    /// 建立时间
    pub built_at: String,
    /// 最后更新时间
    pub updated_at: String,
    /// 索引方式
    pub index_method: String,
}

impl TrieIndex {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
            files: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 0,
            stats: IndexStats::default(),
        }
    }

    /// 添加文件记录
    pub fn add(&mut self, mut record: FileRecord) {
        let path_str = record.path.to_string_lossy().to_string();
        let is_dir = record.is_dir;

        // 如果已存在，先移除
        if let Some(&id) = self.path_to_id.get(&path_str) {
            self.remove_by_id(id);
        }

        // 分配新 ID
        record.id = self.next_id;
        self.next_id += 1;

        let file_id = record.id;
        let name_lower = record.name.to_lowercase();

        // 添加到 Trie 树
        self.root.insert(&name_lower, file_id);

        // 添加到文件存储
        self.files.insert(file_id, record);
        self.path_to_id.insert(path_str, file_id);

        // 更新统计
        if is_dir {
            self.stats.total_dirs += 1;
        } else {
            self.stats.total_files += 1;
        }
    }

    /// 移除文件
    pub fn remove(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();

        if let Some(&id) = self.path_to_id.get(&path_str) {
            self.remove_by_id(id);
        }
    }

    fn remove_by_id(&mut self, file_id: usize) {
        // 先获取需要的信息
        let (name_lower, path_str, is_dir) = if let Some(record) = self.files.get(&file_id) {
            (
                record.name.to_lowercase(),
                record.path.to_string_lossy().to_string(),
                record.is_dir,
            )
        } else {
            return;
        };

        // 从 Trie 树移除
        let chars: Vec<char> = name_lower.chars().collect();
        self.root.remove_file_id(file_id, &chars);

        // 从映射中移除
        self.files.remove(&file_id);
        self.path_to_id.remove(&path_str);

        // 更新统计
        if is_dir {
            if self.stats.total_dirs > 0 {
                self.stats.total_dirs -= 1;
            }
        } else {
            if self.stats.total_files > 0 {
                self.stats.total_files -= 1;
            }
        }
    }

    /// 前缀搜索
    pub fn search_prefix(&self, prefix: &str) -> Vec<&FileRecord> {
        let prefix_lower = prefix.to_lowercase();
        let file_ids = self.root.search_prefix(&prefix_lower);

        file_ids
            .iter()
            .filter_map(|&id| self.files.get(&id))
            .collect()
    }

    /// 通配符搜索
    pub fn search_glob(&self, pattern: &str) -> Vec<&FileRecord> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();

        for record in self.files.values() {
            let name_lower = record.name.to_lowercase();

            if matches_glob(&pattern_lower, &name_lower) {
                results.push(record);
            }
        }

        results
    }

    /// 正则搜索
    pub fn search_regex(&self, regex: &regex::Regex) -> Vec<&FileRecord> {
        let mut results = Vec::new();

        for record in self.files.values() {
            if regex.is_match(&record.name) {
                results.push(record);
            }
        }

        results
    }

    /// 模糊搜索
    pub fn search_fuzzy(&self, query: &str) -> Vec<&FileRecord> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&FileRecord> = self
            .files
            .values()
            .filter(|record| record.name.to_lowercase().contains(&query_lower))
            .collect();

        // 按相关度排序
        results.sort_by(|a, b| {
            let a_score = relevance_score(&a.name, &query_lower);
            let b_score = relevance_score(&b.name, &query_lower);
            b_score.cmp(&a_score)
        });

        results
    }

    /// 获取所有文件
    pub fn all_files(&self) -> Vec<&FileRecord> {
        self.files.values().collect()
    }

    /// 获取文件数量
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// 通过路径获取文件记录
    pub fn get_by_path(&self, path: &Path) -> Option<&FileRecord> {
        let path_str = path.to_string_lossy().to_string();
        self.path_to_id
            .get(&path_str)
            .and_then(|&id| self.files.get(&id))
    }
}

impl Default for TrieIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// 通配符匹配
pub fn matches_glob(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with('*') && pattern.ends_with('*') {
        let inner = &pattern[1..pattern.len() - 1];
        return text.contains(inner);
    }

    if pattern.starts_with('*') {
        return text.ends_with(&pattern[1..]);
    }

    if pattern.ends_with('*') {
        return text.starts_with(&pattern[..pattern.len() - 1]);
    }

    // 支持？通配符
    if pattern.contains('?') {
        return glob_match_exact(pattern, text);
    }

    pattern == text
}

fn glob_match_exact(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars();
    let mut text_chars = text.chars();

    while let Some(p_char) = pattern_chars.next() {
        match p_char {
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            '*' => {
                return true;
            }
            t_char => {
                if Some(t_char) != text_chars.next() {
                    return false;
                }
            }
        }
    }

    text_chars.next().is_none()
}

/// 计算相关度分数
fn relevance_score(name: &str, query: &str) -> u32 {
    let mut score = 0u32;

    // 完全匹配
    if name == query {
        score += 100;
    }

    // 前缀匹配
    if name.starts_with(query) {
        score += 50;
    }

    // 包含匹配
    if name.contains(query) {
        score += 20;
    }

    // 词边界匹配
    if name.split(&['_', '-', '.'][..]).any(|part| part == query) {
        score += 30;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // ==================== TrieNode 基础测试 ====================

    #[test]
    fn test_trie_node_insert() {
        let mut node = TrieNode::new();

        // 插入单个文件名
        node.insert("test", 0);

        // 插入多个文件名
        node.insert("testing", 1);
        node.insert("best", 2);

        // 验证插入后能搜索到
        let ids = node.search_prefix("test");
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));

        // 验证 best 也能搜索到
        let ids = node.search_prefix("best");
        assert!(ids.contains(&2));
    }

    #[test]
    fn test_trie_node_search_prefix() {
        let mut node = TrieNode::new();

        node.insert("apple", 0);
        node.insert("application", 1);
        node.insert("apply", 2);
        node.insert("banana", 3);

        // 前缀 "app" 应该匹配 apple, application, apply
        let ids = node.search_prefix("app");
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));

        // 前缀 "ban" 应该只匹配 banana
        let ids = node.search_prefix("ban");
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&3));

        // 不存在的前缀
        let ids = node.search_prefix("xyz");
        assert!(ids.is_empty());
    }

    #[test]
    fn test_trie_node_remove() {
        let mut node = TrieNode::new();

        node.insert("file1", 0);
        node.insert("file2", 1);
        node.insert("file3", 2);

        // 删除 file2
        let chars: Vec<char> = "file2".chars().collect();
        node.remove_file_id(1, &chars);

        let ids = node.search_prefix("file");
        assert!(ids.contains(&0));
        assert!(!ids.contains(&1));
        assert!(ids.contains(&2));
    }

    // ==================== TrieIndex 基础测试 ====================

    #[test]
    fn test_trie_index_basic() {
        let index = TrieIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_trie_insert_and_search() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("test.rs", "/home/test.rs"));
        index.add(create_test_record("testing.py", "/home/testing.py"));
        index.add(create_test_record("best.rs", "/home/best.rs"));

        // 前缀搜索
        let results = index.search_prefix("test");
        assert_eq!(results.len(), 2);

        // 通配符搜索
        let results = index.search_glob("*.rs");
        assert_eq!(results.len(), 2);

        // 模糊搜索
        let results = index.search_fuzzy("test");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_trie_remove() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("file1.txt", "/home/file1.txt"));
        index.add(create_test_record("file2.txt", "/home/file2.txt"));

        assert_eq!(index.len(), 2);

        index.remove(Path::new("/home/file1.txt"));

        assert_eq!(index.len(), 1);
        assert!(index.get_by_path(Path::new("/home/file1.txt")).is_none());
        assert!(index.get_by_path(Path::new("/home/file2.txt")).is_some());
    }

    // ==================== 边界测试 ====================

    #[test]
    fn test_trie_empty_prefix_search() {
        let mut index = TrieIndex::new();
        index.add(create_test_record("test.txt", "/test.txt"));

        // 空前缀应该返回所有结果
        let results = index.search_prefix("");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_trie_special_characters() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("test-file.txt", "/test-file.txt"));
        index.add(create_test_record("test_file.txt", "/test_file.txt"));
        index.add(create_test_record("test.file.txt", "/test.file.txt"));

        // 连字符匹配
        let results = index.search_prefix("test-");
        assert_eq!(results.len(), 1);

        // 下划线匹配
        let results = index.search_prefix("test_");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_trie_unicode_names() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("测试.txt", "/测试.txt"));
        index.add(create_test_record("テスト.txt", "/テスト.txt"));
        index.add(create_test_record("test.txt", "/test.txt"));

        // 中文前缀搜索
        let results = index.search_prefix("测试");
        assert_eq!(results.len(), 1);

        // 日文前缀搜索
        let results = index.search_prefix("テスト");
        assert_eq!(results.len(), 1);
    }

    // ==================== 通配符匹配测试 ====================

    #[test]
    fn test_glob_matching() {
        assert!(matches_glob("*.txt", "file.txt"));
        assert!(matches_glob("test*", "testing"));
        assert!(matches_glob("*ing", "testing"));
        assert!(matches_glob("*est*", "testing"));
        assert!(!matches_glob("*.rs", "file.txt"));
    }

    #[test]
    fn test_glob_wildcard_question() {
        // ? 匹配单个字符
        assert!(matches_glob("file?.txt", "file1.txt"));
        assert!(matches_glob("file?.txt", "fileA.txt"));
        assert!(!matches_glob("file?.txt", "file10.txt"));
        assert!(!matches_glob("file?.txt", "file.txt"));
    }

    #[test]
    fn test_glob_double_wildcard() {
        // 测试基本通配符功能
        // 注意：当前 glob 实现主要支持简单通配符
        assert!(matches_glob("*.txt", "file.txt"));
        assert!(matches_glob("*test*", "my_test_file.txt"));
        assert!(matches_glob("test*", "testing"));
        assert!(matches_glob("*ing", "testing"));
    }

    // ==================== 大小写不敏感测试 ====================

    #[test]
    fn test_trie_case_insensitive() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("TestFile.txt", "/home/TestFile.txt"));

        // 搜索应该不区分大小写
        let results = index.search_prefix("test");
        assert_eq!(results.len(), 1);

        let results = index.search_glob("*.TXT");
        assert_eq!(results.len(), 1);

        let results = index.search_fuzzy("TESTFILE");
        assert_eq!(results.len(), 1);
    }

    // ==================== 重复添加测试 ====================

    #[test]
    fn test_trie_duplicate_add() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("file.txt", "/home/file.txt"));
        index.add(create_test_record("file.txt", "/home/file.txt"));

        // 重复添加应该更新而不是新增
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_trie_update_existing() {
        let mut index = TrieIndex::new();

        // 添加初始文件
        let mut record = create_test_record("file.txt", "/home/file.txt");
        record.size = 100;
        index.add(record);

        // 更新文件（大小变化）
        let mut record = create_test_record("file.txt", "/home/file.txt");
        record.size = 200;
        index.add(record);

        // 验证更新
        let results = index.get_by_path(Path::new("/home/file.txt"));
        assert!(results.is_some());
        assert_eq!(results.unwrap().size, 200);
        assert_eq!(index.len(), 1);
    }

    // ==================== 模糊搜索相关度排序测试 ====================

    #[test]
    fn test_fuzzy_search_relevance() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("test", "/test"));
        index.add(create_test_record("testing", "/testing"));
        index.add(create_test_record("contest", "/contest"));
        index.add(create_test_record("attest", "/attest"));

        let results = index.search_fuzzy("test");

        // 完全匹配应该排在最前
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "test");
    }

    #[test]
    fn test_relevance_score() {
        // 完全匹配分数最高
        assert!(relevance_score("test", "test") >= 100);

        // 前缀匹配分数次高
        assert!(relevance_score("testing", "test") >= 50);

        // 包含匹配分数较低
        assert!(relevance_score("contest", "test") >= 20);

        // 词边界匹配
        assert!(relevance_score("test_file", "test") >= 30);
    }

    // ==================== 批量操作测试 ====================

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
    fn test_trie_index_mixed_types() {
        let mut index = TrieIndex::new();

        // 添加文件
        index.add(create_test_record("file.txt", "/home/file.txt"));

        // 添加目录
        let mut dir_record = create_test_record("dir", "/home/dir");
        dir_record.is_dir = true;
        index.add(dir_record);

        assert_eq!(index.stats.total_files, 1);
        assert_eq!(index.stats.total_dirs, 1);
        assert_eq!(index.len(), 2);
    }

    // ==================== 性能测试 ====================

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

        // 构建时间也应该合理（< 5 秒）
        assert!(build_time.as_secs() < 5);
    }

    #[test]
    fn test_trie_large_scale_search() {
        let mut index = TrieIndex::new();

        // 添加 50000 个文件
        for i in 0..50000 {
            let name = format!("document_{:05}.pdf", i);
            let path = format!("/docs/{:03}/document_{:05}.pdf", i / 100, i);
            index.add(create_test_record(&name, &path));
        }

        assert_eq!(index.len(), 50000);

        // 测试大规模搜索性能
        let start = std::time::Instant::now();
        let results = index.search_prefix("document_12");
        let elapsed = start.elapsed();

        println!("50000 文件中前缀搜索耗时：{:?}", elapsed);
        assert!(!results.is_empty());
        assert!(elapsed.as_millis() < 100); // 应该在 100ms 内完成
    }

    // ==================== 路径查找测试 ====================

    #[test]
    fn test_get_by_path() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("test.rs", "/home/project/test.rs"));
        index.add(create_test_record("main.rs", "/home/project/src/main.rs"));

        // 查找存在的路径
        let result = index.get_by_path(Path::new("/home/project/test.rs"));
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test.rs");

        // 查找不存在的路径
        let result = index.get_by_path(Path::new("/nonexistent"));
        assert!(result.is_none());
    }

    #[test]
    fn test_all_files() {
        let mut index = TrieIndex::new();

        index.add(create_test_record("file1.txt", "/file1.txt"));
        index.add(create_test_record("file2.txt", "/file2.txt"));
        index.add(create_test_record("file3.txt", "/file3.txt"));

        let all = index.all_files();
        assert_eq!(all.len(), 3);
    }
}
