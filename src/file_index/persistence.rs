/// 索引持久化模块
///
/// 实现索引的二进制序列化，支持快速保存和加载：
/// - 启动时快速加载索引（秒级）
/// - 定期自动保存
/// - 增量更新支持
use crate::file_index::trie::{FileRecord, IndexStats, TrieIndex};
use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// 索引文件魔数
const INDEX_MAGIC: &[u8; 4] = b"SEIX";

/// 索引文件格式版本
const INDEX_VERSION: u32 = 1;

/// 索引持久化管理器
pub struct IndexPersistence {
    /// 索引文件路径
    index_path: PathBuf,
    /// 自动保存间隔（秒）
    auto_save_interval: u64,
    /// 上次保存时间
    last_save: Option<std::time::Instant>,
}

impl IndexPersistence {
    /// 创建持久化管理器
    pub fn new(index_path: PathBuf, auto_save_interval: u64) -> Self {
        Self {
            index_path,
            auto_save_interval,
            last_save: None,
        }
    }

    /// 保存索引到文件
    pub fn save(&mut self, index: &TrieIndex) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.index_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let file = File::create(&self.index_path).with_context(|| {
            format!("Failed to create index file: {}", self.index_path.display())
        })?;
        let mut writer = BufWriter::new(file);

        // 写入魔数
        writer.write_all(INDEX_MAGIC)?;

        // 写入版本
        writer.write_all(&INDEX_VERSION.to_le_bytes())?;

        // 序列化索引数据
        let serialized = bincode::serialize(index)
            .map_err(|e| anyhow::anyhow!("Failed to serialize index: {}", e))?;

        // 写入数据长度
        let len = serialized.len() as u64;
        writer.write_all(&len.to_le_bytes())?;

        // 写入数据
        writer.write_all(&serialized)?;

        writer.flush()?;

        self.last_save = Some(std::time::Instant::now());

        tracing::info!("索引已保存到：{}", self.index_path.display());

        Ok(())
    }

    /// 从文件加载索引
    pub fn load(&self) -> Result<Option<TrieIndex>> {
        if !self.index_path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.index_path)
            .with_context(|| format!("Failed to open index file: {}", self.index_path.display()))?;
        let mut reader = BufReader::new(file);

        // 读取并验证魔数
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if &magic != INDEX_MAGIC {
            bail!("Invalid index file format: wrong magic number");
        }

        // 读取版本
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        if version != INDEX_VERSION {
            bail!("Unsupported index file version: {}", version);
        }

        // 读取数据长度
        let mut len_bytes = [0u8; 8];
        reader.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        // 读取数据
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data)?;

        // 反序列化
        let index: TrieIndex = bincode::deserialize(&data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize index: {}", e))?;

        tracing::info!(
            "索引已从 {} 加载，包含 {} 个文件",
            self.index_path.display(),
            index.len()
        );

        Ok(Some(index))
    }

    /// 检查是否需要自动保存
    pub fn should_auto_save(&self) -> bool {
        if let Some(last) = self.last_save {
            last.elapsed().as_secs() >= self.auto_save_interval
        } else {
            true
        }
    }

    /// 获取索引文件路径
    pub fn index_path(&self) -> &Path {
        &self.index_path
    }

    /// 获取索引文件大小
    pub fn index_size(&self) -> Result<u64> {
        if self.index_path.exists() {
            Ok(fs::metadata(&self.index_path)?.len())
        } else {
            Ok(0)
        }
    }
}

/// 默认的索引文件路径
pub fn get_default_index_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("searchEverything")
        .join("index.bin")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_index() -> TrieIndex {
        let mut index = TrieIndex::new();

        let record = FileRecord {
            path: PathBuf::from("/home/test.rs"),
            name: "test.rs".to_string(),
            size: 1024,
            is_dir: false,
            modified: SystemTime::now(),
            created: None,
            id: 0,
        };

        index.add(record);
        index
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let index_path = temp_dir.join("test_index.bin");

        // 创建测试索引
        let original_index = create_test_index();

        // 保存
        let mut persistence = IndexPersistence::new(index_path.clone(), 300);
        persistence.save(&original_index).expect("Failed to save");

        // 加载
        let loaded_index = persistence
            .load()
            .expect("Failed to load")
            .expect("No index found");

        // 验证
        assert_eq!(original_index.len(), loaded_index.len());

        // 清理
        fs::remove_file(&index_path).ok();
    }
}
