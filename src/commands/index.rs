use anyhow::Result;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

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
            // 首次启动，自动索引所有系统路径
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
            // Windows: 检测所有盘符
            for drive in 'C'..='Z' {
                let path = format!("{}:/", drive);
                if Path::new(&path).exists() {
                    indexed_paths.push(path);
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux: 索引常用用户目录
            let user_dirs = [
                "/home",
                "/opt",
                "/var/www",
                "/srv",
            ];
            for dir in user_dirs.iter() {
                if Path::new(dir).exists() {
                    indexed_paths.push(dir.to_string());
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS: 索引用户目录
            let user_dirs = [
                "/Users",
                "/Applications",
            ];
            for dir in user_dirs.iter() {
                if Path::new(dir).exists() {
                    indexed_paths.push(dir.to_string());
                }
            }
        }
        
        Self {
            indexed_paths,
            excluded_paths: vec![
                // 默认排除系统目录
                #[cfg(target_os = "windows")]
                "C:/Windows".to_string(),
                #[cfg(target_os = "windows")]
                "C:/Program Files".to_string(),
                #[cfg(target_os = "linux")]
                "/proc".to_string(),
                #[cfg(target_os = "linux")]
                "/sys".to_string(),
            ],
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
            
            // 显示索引统计（TODO: 实现后添加）
            println!("索引统计:");
            println!("  文件总数：待实现");
            println!("  索引大小：待实现");
            println!("  最后更新：待实现");
        }
        
        crate::IndexAction::Rebuild { path } => {
            println!("正在重建索引...");
            
            let mut config = IndexConfig::load()?;
            
            if path.to_str() != Some(".") {
                // 重建特定路径
                println!("重建路径：{}", path.display());
                // TODO: 实现特定路径重建
            } else {
                // 重建所有路径
                for indexed_path in &config.indexed_paths {
                    println!("  索引：{}", indexed_path);
                    // TODO: 实现索引逻辑
                }
            }
            
            println!("索引完成！");
        }
        
        crate::IndexAction::Watch { path } => {
            println!("启动实时监控：{}", path.display());
            println!("按 Ctrl+C 停止监控");
            // TODO: 实现 inotify/USN 监控
        }
        
        crate::IndexAction::Add { path } => {
            let mut config = IndexConfig::load()?;
            let path_str = path.to_string_lossy().to_string();
            
            if !config.indexed_paths.contains(&path_str) {
                config.indexed_paths.push(path_str.clone());
                config.save()?;
                println!("已添加索引路径：{}", path_str);
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
        
        crate::IndexAction::Exclude { action: exclude_action } => {
            use crate::commands::index::ExcludeAction::*;
            
            let mut config = IndexConfig::load()?;
            
            match exclude_action {
                Add { path } => {
                    let path_str = path.to_string_lossy().to_string();
                    if !config.excluded_paths.contains(&path_str) {
                        config.excluded_paths.push(path_str.clone());
                        config.save()?;
                        println!("已添加排除路径：{}", path_str);
                    }
                }
                Remove { path } => {
                    let path_str = path.to_string_lossy().to_string();
                    if let Some(pos) = config.excluded_paths.iter().position(|p| p == &path_str) {
                        config.excluded_paths.remove(pos);
                        config.save()?;
                        println!("已移除排除路径：{}", path_str);
                    }
                }
                List => {
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

/// 排除操作
#[derive(clap::Subcommand)]
pub enum ExcludeAction {
    /// 添加排除路径
    Add {
        #[arg(short, long)]
        path: std::path::PathBuf,
    },
    /// 移除排除路径
    Remove {
        #[arg(short, long)]
        path: std::path::PathBuf,
    },
    /// 列出排除路径
    List,
}
