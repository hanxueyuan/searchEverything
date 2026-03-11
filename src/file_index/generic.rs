/// 通用索引构建器 (用于不支持的平台)
/// 
/// 使用标准文件系统扫描：
/// - 跨平台兼容
/// - 初始扫描建立索引
/// - 无实时监控

use super::{Index, IndexBuilder, IndexStats, FileRecord};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 通用索引构建器
pub struct GenericIndexBuilder;

impl GenericIndexBuilder {
    pub fn new() -> Self {
        Self
    }
    
    /// 扫描目录建立索引
    fn scan_directory(&self, path: &Path, exclude_paths: &[String]) -> Result<Vec<FileRecord>> {
        use walkdir::WalkDir;
        
        let mut records = Vec::new();
        
        if should_exclude(&path.to_string_lossy(), exclude_paths) {
            return Ok(records);
        }
        
        println!("扫描目录：{}", path.display());
        
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !should_exclude(&e.path().to_string_lossy(), exclude_paths))
            .filter_map(|e| e.ok())
        {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let path_buf = entry.path().to_path_buf();
            
            let record = FileRecord {
                path: path_buf,
                name: entry.file_name().to_string_lossy().to_string(),
                size: if metadata.is_file() { metadata.len() } else { 0 },
                is_dir: metadata.is_dir(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                created: metadata.created().ok(),
            };
            
            records.push(record);
        }
        
        Ok(records)
    }
}

impl IndexBuilder for GenericIndexBuilder {
    fn build(&self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<Index> {
        let mut index = Index::new();
        let start_time = std::time::Instant::now();
        
        // 扫描所有路径
        for path in paths {
            let records = self.scan_directory(path, exclude_paths)?;
            for record in records {
                index.add(record);
            }
        }
        
        // 更新统计信息
        let duration = start_time.elapsed();
        index.stats.built_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        index.stats.updated_at = index.stats.built_at.clone();
        index.stats.index_method = self.platform_name().to_string();
        index.stats.index_size_mb = (index.len() * 100) as f64 / 1024.0 / 1024.0;
        
        println!(
            "索引构建完成：{} 文件，{} 目录，耗时 {:.2}s",
            index.stats.total_files,
            index.stats.total_dirs,
            duration.as_secs_f64()
        );
        
        Ok(index)
    }
    
    fn platform_name(&self) -> &str {
        "Generic (scan)"
    }
    
    fn supports_realtime(&self) -> bool {
        false
    }
}

impl Default for GenericIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 检查路径是否应该被排除
fn should_exclude(path: &str, exclude_paths: &[String]) -> bool {
    let path_lower = path.to_lowercase();
    
    for exclude in exclude_paths {
        let exclude_lower = exclude.to_lowercase();
        
        if path_lower == exclude_lower {
            return true;
        }
        
        if exclude_lower.starts_with("**/") {
            let pattern = &exclude_lower[3..];
            if path_lower.contains(&pattern) {
                return true;
            }
        }
        
        if exclude_lower.ends_with("*") {
            let prefix = &exclude_lower[..exclude_lower.len() - 1];
            if path_lower.starts_with(prefix) {
                return true;
            }
        }
    }
    
    false
}
