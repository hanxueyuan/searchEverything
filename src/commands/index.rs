use crate::file_index::persistence::get_default_index_path;
use crate::file_index::IndexManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use crate::file_index::linux::{get_linux_system_excludes, start_watch};

#[cfg(target_os = "windows")]
use crate::file_index::windows::{get_windows_system_excludes, start_usn_watch};

/// Index configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct IndexConfig {
    /// List of indexed paths
    pub indexed_paths: Vec<String>,
    /// List of excluded paths
    pub excluded_paths: Vec<String>,
    /// Whether initialized
    pub initialized: bool,
}

impl IndexConfig {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            Ok(serde_yaml::from_str(&content)?)
        } else {
            // First launch, auto-initialize
            let config = Self::auto_init();
            config.save()?;
            Ok(config)
        }
    }

    pub fn auto_init() -> Self {
        let mut indexed_paths = Vec::new();

        // Auto-detect system paths
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
            // Linux: Index user directories and common paths
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

        // Default exclusion paths
        let mut excluded_paths = vec![
            #[cfg(target_os = "windows")]
            "C:/Windows".to_string(),
            #[cfg(target_os = "windows")]
            "C:/Program Files".to_string(),
        ];

        // Add Linux system excludes
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

            println!("Index Status");
            println!("═══════════════════════════════════════");
            println!(
                "Initialized: {}",
                if config.initialized { "Yes" } else { "No" }
            );
            println!();

            println!("Indexed paths ({}):", config.indexed_paths.len());
            for path in &config.indexed_paths {
                println!("  ✓ {}", path);
            }
            println!();

            if !config.excluded_paths.is_empty() {
                println!("Excluded paths ({}):", config.excluded_paths.len());
                for path in &config.excluded_paths {
                    println!("  ✗ {}", path);
                }
                println!();
            }

            // Try to load index statistics
            let index_path = get_default_index_path();
            if index_path.exists() {
                if let Ok(metadata) = fs::metadata(&index_path) {
                    println!("Index file:");
                    println!("  Path: {}", index_path.display());
                    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);
                    println!(
                        "  Modified: {}",
                        metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| format!("{:?} ago", d.as_secs()))
                            .unwrap_or_else(|| "Unknown".to_string())
                    );
                }
            }
        }

        crate::IndexAction::Rebuild { path } => {
            println!("Rebuilding index...");

            let config = IndexConfig::load()?;
            let paths_to_index = if path.to_str() != Some(".") {
                vec![path.clone()]
            } else {
                config.indexed_paths.iter().map(PathBuf::from).collect()
            };

            // Create index manager (with persistence)
            let mut manager = IndexManager::with_persistence(
                get_default_index_path(),
                300, // 5 minutes auto-save
            )?;

            // Build index
            let index = manager.build_index(&paths_to_index, &config.excluded_paths)?;

            println!("Indexing complete!");
            println!("  Total files: {}", index.stats.total_files);
            println!("  Total directories: {}", index.stats.total_dirs);
            println!("  Index size: {:.2} KB", index.stats.index_size_mb * 1024.0);
            println!("  Built at: {}", index.stats.built_at);
        }

        crate::IndexAction::Watch { path } => {
            let config = IndexConfig::load()?;

            // Determine watch paths
            let paths_to_watch = if path.to_str() != Some(".") {
                vec![path.clone()]
            } else {
                config.indexed_paths.iter().map(PathBuf::from).collect()
            };

            // Try to load existing index
            let mut manager = IndexManager::with_persistence(
                get_default_index_path(),
                60, // 1 minute auto-save
            )?;

            // Load or build index
            let _ = manager.load_or_build(&paths_to_watch, &config.excluded_paths);

            // Get mutable index reference and start monitoring (cross-platform)
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
                println!("Added index path: {}", path_str);

                // Prompt user to rebuild index
                println!("Tip: Run 'searchEverything index rebuild' to rebuild index");
            } else {
                println!("Path already indexed: {}", path_str);
            }
        }

        crate::IndexAction::Remove { path } => {
            let mut config = IndexConfig::load()?;
            let path_str = path.to_string_lossy().to_string();

            if let Some(pos) = config.indexed_paths.iter().position(|p| p == &path_str) {
                config.indexed_paths.remove(pos);
                config.save()?;
                println!("Removed index path: {}", path_str);
            } else {
                println!("Path not in index: {}", path_str);
            }
        }

        crate::IndexAction::List => {
            let config = IndexConfig::load()?;

            println!("Indexed paths:");
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
                        println!("Added exclusion path: {}", path_str);
                    }
                }
                crate::ExcludeAction::Remove { ref path } => {
                    let path_str = path.to_string_lossy().to_string();
                    if let Some(pos) = config.excluded_paths.iter().position(|p| p == &path_str) {
                        config.excluded_paths.remove(pos);
                        config.save()?;
                        println!("Removed exclusion path: {}", path_str);
                    }
                }
                crate::ExcludeAction::List => {
                    println!("Exclusion paths:");
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
