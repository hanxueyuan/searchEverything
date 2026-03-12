use crate::file_index::persistence::get_default_index_path;
use crate::file_index::{IndexManager, IndexPersistence, TrieIndex};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use crate::file_index::linux::{get_linux_system_excludes, start_watch};

#[cfg(target_os = "windows")]
use crate::file_index::windows::{get_windows_system_excludes, start_usn_watch};

/// 索引配置
#[derive(Serialize, Deserialize, Clone)]
pub struct IndexConfig {
    /// 已索引的路径列表
    pub indexed_paths: Vec<String>,
    /// 排除的路径列表
    pub excluded_paths: Vec<String>,
    /// 是否已初始化
    pub initialized: bool,
}

impl IndexConfig {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            Ok(serde_yaml::from_str(&content)?)
        } else {
            // 首次启动，自动初始化
            let config = Self::auto_init();
            config.save()?;
            Ok(config)
        }
    }

    pub fn auto_init() -> Self {
        let mut indexed_paths = Vec::new();

        // 自动检测系统路径
        #[cfg(target_os = "windows")]
        {
            for drive in 'C'..='Z' {
                let path = format!("{}:/", drive);
                if Path::new(&path).exists() {
                    indexed_paths.push(path);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 索引用户目录和常用路径
            let user_dirs = ["/home", "/opt", "/var/www", "/srv"];
            for dir in user_dirs.iter() {
                if Path::new(dir).exists() {
                    indexed_paths.push(dir.to_string());
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let user_dirs = ["/Users", "/Applications"];
            for dir in user_dirs.iter() {
                if Path::new(dir).exists() {
                    indexed_paths.push(dir.to_string());
                }
            }
        }

        // 默认排除路径
        let mut excluded_paths = vec![
            #[cfg(target_os = "windows")]
            "C:/Windows".to_string(),
            #[cfg(target_os = "windows")]
            "C:/Program Files".to_string(),
        ];

        // 添加 Linux 系统排除
        #[cfg(target_os = "linux")]
        {
            excluded_paths.extend(get_linux_system_excludes());
        }

        Self {
            indexed_paths,
            excluded_paths,
            initialized: true,
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> std::path::PathBuf {
    let config_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    };

    std::path::PathBuf::from(config_dir)
        .join(".config")
        .join("searchEverything")
        .join("index-config.yaml")
}

pub fn execute(action: &crate::IndexAction) -> Result<()> {
    match action {
        crate::IndexAction::Status => {
            let config = IndexConfig::load()?;

            println!("索引状态");
            println!("═══════════════════════════════════════");
            println!("已初始化：{}", if config.initialized { "是" } else { "否" });
            println!();

            println!("已索引路径 ({} 个):", config.indexed_paths.len());
            for path in &config.indexed_paths {
                println!("  ✓ {}", path);
            }
            println!();

            if !config.excluded_paths.is_empty() {
                println!("排除路径 ({} 个):", config.excluded_paths.len());
                for path in &config.excluded_paths {
                    println!("  ✗ {}", path);
                }
                println!();
            }

            // 尝试加载索引统计
            let index_path = get_default_index_path();
            if index_path.exists() {
                if let Ok(metadata) = fs::metadata(&index_path) {
                    println!("索引文件:");
                    println!("  路径：{}", index_path.display());
                    println!("  大小：{:.2} KB", metadata.len() as f64 / 1024.0);
                    println!(
                        "  修改时间：{}",
                        metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| format!("{:?} 前", d.as_secs()))
                            .unwrap_or_else(|| "未知".to_string())
                    );
                }
            }
        }

        crate::IndexAction::Rebuild { path } => {
            println!("正在重建索引...");

            let config = IndexConfig::load()?;
            let paths_to_index = if path.to_str() != Some(".") {
                vec![path.clone()]
            } else {
                config.indexed_paths.iter().map(PathBuf::from).collect()
            };

            // 创建索引管理器（带持久化）
            let mut manager = IndexManager::with_persistence(
                get_default_index_path(),
                300, // 5 分钟自动保存
            )?;

            // 构建索引
            let index = manager.build_index(&paths_to_index, &config.excluded_paths)?;

            println!("索引完成！");
            println!("  文件总数：{}", index.stats.total_files);
            println!("  目录总数：{}", index.stats.total_dirs);
            println!("  索引大小：{:.2} KB", index.stats.index_size_mb * 1024.0);
            println!("  构建时间：{}", index.stats.built_at);
        }

        crate::IndexAction::Watch { path } => {
            let config = IndexConfig::load()?;

            // 确定监控路径
            let paths_to_watch = if path.to_str() != Some(".") {
                vec![path.clone()]
            } else {
                config.indexed_paths.iter().map(PathBuf::from).collect()
            };

            // 尝试加载现有索引
            let mut manager = IndexManager::with_persistence(
                get_default_index_path(),
                60, // 1 分钟自动保存
            )?;

            // 加载或构建索引
            let _ = manager.load_or_build(&paths_to_watch, &config.excluded_paths);

            // 获取索引可变引用并启动监控（跨平台）
            if let Some(index) = manager.get_index_mut() {
                #[cfg(target_os = "linux")]
                {
                    start_watch(&paths_to_watch, &config.excluded_paths, index)?;
                }

                #[cfg(target_os = "windows")]
                {
                    start_usn_watch(&paths_to_watch, &config.excluded_paths, index)?;
                }

                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                {
                    anyhow::bail!("Real-time monitoring is only available on Linux and Windows");
                }
            }
        }

        crate::IndexAction::Add { path } => {
            let mut config = IndexConfig::load()?;
            let path_str = path.to_string_lossy().to_string();

            if !config.indexed_paths.contains(&path_str) {
                config.indexed_paths.push(path_str.clone());
                config.save()?;
                println!("已添加索引路径：{}", path_str);

                // 提示用户重建索引
                println!("提示：运行 'searchEverything index rebuild' 以重建索引");
            } else {
                println!("路径已在索引中：{}", path_str);
            }
        }

        crate::IndexAction::Remove { path } => {
            let mut config = IndexConfig::load()?;
            let path_str = path.to_string_lossy().to_string();

            if let Some(pos) = config.indexed_paths.iter().position(|p| p == &path_str) {
                config.indexed_paths.remove(pos);
                config.save()?;
                println!("已移除索引路径：{}", path_str);
            } else {
                println!("路径不在索引中：{}", path_str);
            }
        }

        crate::IndexAction::List => {
            let config = IndexConfig::load()?;

            println!("已索引路径:");
            for path in &config.indexed_paths {
                println!("  {}", path);
            }
        }

        crate::IndexAction::Exclude {
            action: exclude_action,
        } => {
            let mut config = IndexConfig::load()?;

            match exclude_action {
                crate::ExcludeAction::Add { ref path } => {
                    let path_str = path.to_string_lossy().to_string();
                    if !config.excluded_paths.contains(&path_str) {
                        config.excluded_paths.push(path_str.clone());
                        config.save()?;
                        println!("已添加排除路径：{}", path_str);
                    }
                }
                crate::ExcludeAction::Remove { ref path } => {
                    let path_str = path.to_string_lossy().to_string();
                    if let Some(pos) = config.excluded_paths.iter().position(|p| p == &path_str) {
                        config.excluded_paths.remove(pos);
                        config.save()?;
                        println!("已移除排除路径：{}", path_str);
                    }
                }
                crate::ExcludeAction::List => {
                    println!("排除路径:");
                    for path in &config.excluded_paths {
                        println!("  {}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_and_save() {
        let config = IndexConfig::auto_init();

        assert!(config.initialized);
        assert!(!config.indexed_paths.is_empty());

        // 验证 Linux 排除路径
        #[cfg(target_os = "linux")]
        {
            assert!(config.excluded_paths.contains(&"/proc".to_string()));
            assert!(config.excluded_paths.contains(&"/sys".to_string()));
            assert!(config.excluded_paths.contains(&"/dev".to_string()));
        }
    }
}
