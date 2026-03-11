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
pub mod trie;
pub mod persistence;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub use trie::{TrieIndex, FileRecord, IndexStats};
pub use persistence::IndexPersistence;

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
    fn build(&self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<TrieIndex>;
    
    /// 获取平台名称
    fn platform_name(&self) -> &str;
    
    /// 是否支持实时索引
    fn supports_realtime(&self) -> bool;
}

/// 索引管理器
pub struct IndexManager {
    index: Option<TrieIndex>,
    builder: Box<dyn IndexBuilder>,
    persistence: Option<IndexPersistence>,
}

impl IndexManager {
    /// 创建索引管理器
    pub fn new() -> Result<Self> {
        let builder = create_platform_builder()?;
        
        Ok(Self {
            index: None,
            builder,
            persistence: None,
        })
    }
    
    /// 创建带持久化的索引管理器
    pub fn with_persistence(index_path: PathBuf, auto_save_interval: u64) -> Result<Self> {
        let builder = create_platform_builder()?;
        let persistence = IndexPersistence::new(index_path, auto_save_interval);
        
        Ok(Self {
            index: None,
            builder,
            persistence: Some(persistence),
        })
    }
    
    /// 构建索引
    pub fn build_index(&mut self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<&TrieIndex> {
        println!("正在使用 {} 构建索引...", self.builder.platform_name());
        
        let index = self.builder.build(paths, exclude_paths)?;
        self.index = Some(index);
        
        // 自动保存
        if let Some(ref mut persistence) = self.persistence {
            if let Some(ref index) = self.index {
                persistence.save(index)?;
            }
        }
        
        Ok(self.index.as_ref().unwrap())
    }
    
    /// 尝试从持久化加载索引
    pub fn load_or_build(&mut self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<&TrieIndex> {
        // 尝试从持久化加载
        if let Some(ref persistence) = self.persistence {
            if let Ok(Some(loaded_index)) = persistence.load() {
                println!("已从持久化加载索引 ({} 个文件)", loaded_index.len());
                self.index = Some(loaded_index);
                
                // 增量更新（TODO: 实现）
                return Ok(self.index.as_ref().unwrap());
            }
        }
        
        // 构建新索引
        self.build_index(paths, exclude_paths)
    }
    
    /// 获取索引
    pub fn get_index(&self) -> Option<&TrieIndex> {
        self.index.as_ref()
    }
    
    /// 获取可变索引引用
    pub fn get_index_mut(&mut self) -> Option<&mut TrieIndex> {
        self.index.as_mut()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> Option<&IndexStats> {
        self.index.as_ref().map(|i| &i.stats)
    }
    
    /// 是否支持实时索引
    pub fn supports_realtime(&self) -> bool {
        self.builder.supports_realtime()
    }
    
    /// 保存索引
    pub fn save_index(&mut self) -> Result<()> {
        if let Some(ref mut persistence) = self.persistence {
            if let Some(ref index) = self.index {
                return persistence.save(index);
            }
        }
        Ok(())
    }
    
    /// 自动保存检查
    pub fn maybe_auto_save(&mut self) -> Result<()> {
        if let Some(ref mut persistence) = self.persistence {
            if persistence.should_auto_save() {
                if let Some(ref index) = self.index {
                    persistence.save(index)?;
                }
            }
        }
        Ok(())
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
