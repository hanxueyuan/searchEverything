/// 跨平台文件索引模块
/// 
/// 根据不同平台使用最优策略：
/// - Windows: MFT + USN Journal (秒级索引)
/// - Linux: 扫描 + inotify (分钟级索引)
/// - macOS: 扫描 + FSEvents (分钟级索引)

pub mod windows;
pub mod linux;
pub mod macos;
pub mod generic;

use anyhow::Result;
use serde::{Deserialize, Serialize};
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

/// 文件变更事件
#[derive(Debug, Clone)]
pub enum FileChange {
    Created(FileRecord),
    Modified(FileRecord),
    Deleted(PathBuf),
    Renamed { old_path: PathBuf, new_path: FileRecord },
}

/// 索引构建器 trait
pub trait IndexBuilder: Send + Sync {
    /// 构建索引
    fn build(&self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<Index>;
    
    /// 获取平台名称
    fn platform_name(&self) -> &str;
    
    /// 是否支持实时索引
    fn supports_realtime(&self) -> bool;
}

/// 索引管理器
pub struct IndexManager {
    index: Option<Index>,
    builder: Box<dyn IndexBuilder>,
}

impl IndexManager {
    /// 创建索引管理器
    pub fn new() -> Result<Self> {
        let builder = create_platform_builder()?;
        
        Ok(Self {
            index: None,
            builder,
        })
    }
    
    /// 构建索引
    pub fn build_index(&mut self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<&Index> {
        println!("正在使用 {} 构建索引...", self.builder.platform_name());
        
        let index = self.builder.build(paths, exclude_paths)?;
        self.index = Some(index);
        
        Ok(self.index.as_ref().unwrap())
    }
    
    /// 获取索引
    pub fn get_index(&self) -> Option<&Index> {
        self.index.as_ref()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> Option<&IndexStats> {
        self.index.as_ref().map(|i| &i.stats)
    }
    
    /// 是否支持实时索引
    pub fn supports_realtime(&self) -> bool {
        self.builder.supports_realtime()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new().expect("Failed to create IndexManager")
    }
}

/// 创建平台特定的索引构建器
pub fn create_platform_builder() -> Result<Box<dyn IndexBuilder>> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::MftIndexBuilder::new()))
    }
    
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxIndexBuilder::new()))
    }
    
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOsIndexBuilder::new()))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(Box::new(generic::GenericIndexBuilder::new()))
    }
}

/// 内存索引 (Trie 树实现)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// 文件记录 (按路径索引)
    files: HashMap<String, FileRecord>,
    /// 按文件名索引
    name_index: HashMap<String, Vec<String>>,
    /// 统计信息
    pub stats: IndexStats,
}

impl Index {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            name_index: HashMap::new(),
            stats: IndexStats::default(),
        }
    }
    
    /// 添加文件记录
    pub fn add(&mut self, record: FileRecord) {
        let path_str = record.path.to_string_lossy().to_string();
        let name_lower = record.name.to_lowercase();
        
        // 添加到路径索引
        self.files.insert(path_str.clone(), record.clone());
        
        // 添加到名称索引
        self.name_index
            .entry(name_lower)
            .or_insert_with(Vec::new)
            .push(path_str);
        
        // 更新统计
        if record.is_dir {
            self.stats.total_dirs += 1;
        } else {
            self.stats.total_files += 1;
        }
    }
    
    /// 移除文件记录
    pub fn remove(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        
        if let Some(record) = self.files.remove(&path_str) {
            let name_lower = record.name.to_lowercase();
            
            if let Some(paths) = self.name_index.get_mut(&name_lower) {
                paths.retain(|p| p != &path_str);
                if paths.is_empty() {
                    self.name_index.remove(&name_lower);
                }
            }
            
            if record.is_dir {
                self.stats.total_dirs -= 1;
            } else {
                self.stats.total_files -= 1;
            }
        }
    }
    
    /// 搜索文件 (通配符匹配)
    pub fn search_glob(&self, pattern: &str) -> Vec<&FileRecord> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();
        
        for (path, record) in &self.files {
            let name = record.name.to_lowercase();
            
            if matches_glob(&pattern_lower, &name) {
                results.push(record);
            }
        }
        
        results
    }
    
    /// 搜索文件 (正则表达式)
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
        let mut results = Vec::new();
        
        for record in self.files.values() {
            let name_lower = record.name.to_lowercase();
            
            if name_lower.contains(&query_lower) {
                results.push(record);
            }
        }
        
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
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

/// 通配符匹配
fn matches_glob(pattern: &str, text: &str) -> bool {
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

/// 获取平台名称
pub fn get_platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Windows MFT/USN"
    }
    
    #[cfg(target_os = "linux")]
    {
        "Linux inotify"
    }
    
    #[cfg(target_os = "macos")]
    {
        "macOS FSEvents"
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "Generic (scan)"
    }
}
