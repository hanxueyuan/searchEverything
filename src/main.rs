use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

mod commands;
mod config;
mod error;
mod audit;
mod file_index;
pub mod output;

use commands::{search, info, cat, copy, move_file, delete, index as index_cmd};
use config::ConfigManager;
use audit::{AuditLogger, AuditLogEntry};
pub use output::OutputFormat;

#[derive(Parser)]
#[command(name = "se")]
#[command(about = "searchEverything - 本地文件搜索工具", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// 启用详细日志
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 搜索文件
    Search {
        /// 搜索模式（支持通配符 * 和 ?）
        pattern: String,
        
        /// 搜索路径
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// 最大结果数
        #[arg(short, long, default_value = "100")]
        limit: usize,
        
        /// 输出格式
        #[arg(short, long, value_enum, default_value = "json")]
        format: output::OutputFormat,
        
        /// 文件类型过滤
        #[arg(short, long, value_enum, default_value = "both")]
        type_: FileType,
        
        /// 使用正则表达式
        #[arg(long)]
        regex: bool,
        
        /// 使用模糊搜索
        #[arg(long)]
        fuzzy: bool,
        
        /// 流式输出（实时显示结果）
        #[arg(long)]
        stream: bool,
    },
    
    /// 查看文件信息
    Info {
        /// 文件路径
        file: PathBuf,
        
        /// JSON 输出
        #[arg(short, long)]
        json: bool,
    },
    
    /// 读取文件内容
    Cat {
        /// 文件路径
        file: PathBuf,
        
        /// 显示行数
        #[arg(short, long)]
        lines: Option<usize>,
        
        /// 显示末尾 N 行（类似 tail）
        #[arg(long)]
        tail: bool,
    },
    
    /// 复制文件
    Copy {
        /// 源文件路径（支持通配符）
        source: String,
        
        /// 目标路径
        dest: PathBuf,
    },
    
    /// 移动文件
    Move {
        /// 源文件路径
        source: String,
        
        /// 目标路径
        dest: PathBuf,
    },
    
    /// 删除文件
    Delete {
        /// 文件路径（支持通配符）
        path: String,
        
        /// 强制删除，不确认
        #[arg(short, long)]
        force: bool,
    },
    
    /// 索引管理
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },
    
    /// 快速索引状态
    IndexStatus,
    
    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// 显示当前配置
    Show,
    
    /// 编辑配置文件
    Edit,
    
    /// 重置为默认配置
    Reset,
    
    /// 验证配置文件
    Validate,
}



#[derive(Clone, ValueEnum, Debug)]
enum FileType {
    File,
    Dir,
    Both,
}

#[derive(Subcommand, Debug)]
enum IndexAction {
    /// 显示索引状态
    Status,
    
    /// 重建索引
    Rebuild {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 启动实时监控
    Watch {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 添加索引路径
    Add {
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// 移除索引路径
    Remove {
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// 列出索引路径
    List,
    
    /// 排除路径管理
    Exclude {
        #[command(subcommand)]
        action: ExcludeAction,
    },
}

#[derive(Subcommand, Debug)]
enum ExcludeAction {
    /// 添加排除路径
    Add {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// 移除排除路径
    Remove {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// 列出排除路径
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start_time = std::time::Instant::now();
    
    // 初始化日志
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
    
    // 加载配置和审计日志
    let config_mgr = ConfigManager::load()?;
    let audit_logger = AuditLogger::new(
        audit::get_default_log_path(),
        config_mgr.get().file_operations.enable_log,
    );
    
    // 获取命令名称用于审计
    let command_name = format!("{:?}", cli.command);
    
    match cli.command {
        Commands::Search { pattern, path, limit, format, type_, regex, fuzzy, stream } => {
            use output::OutputFormat;
            let entry = AuditLogEntry::new("search", vec![pattern.clone(), path.display().to_string()]);
            search::execute(&pattern, &path, limit, &format, &type_, regex, fuzzy, stream)?;
            audit_logger.log_success(&entry.with_duration(start_time.elapsed().as_millis() as u64))?;
        }
        Commands::Info { file, json } => {
            info::execute(&file, json)?;
        }
        Commands::Cat { file, lines, tail } => {
            cat::execute(&file, lines, tail)?;
        }
        Commands::Copy { source, dest } => {
            copy::execute(&source, &dest)?;
        }
        Commands::Move { source, dest } => {
            move_file::execute(&source, &dest)?;
        }
        Commands::Delete { path, force } => {
            delete::execute(&path, force)?;
        }
        Commands::Index { action } => {
            index_cmd::execute(&action)?;
        }
        Commands::IndexStatus => {
            index_cmd::execute(&crate::IndexAction::Status)?;
        }
        Commands::Config { action } => {
            handle_config(action)?;
        }
    }
    
    Ok(())
}

/// 处理配置命令
fn handle_config(action: crate::ConfigAction) -> Result<()> {
    match action {
        crate::ConfigAction::Show => {
            let config_mgr = ConfigManager::load()?;
            let config = config_mgr.get();
            
            println!("配置文件路径：{}", config_mgr.get_config_path().display());
            println!();
            println!("当前配置:");
            println!("  搜索模式：{}", config.search.default_mode);
            println!("  默认结果数：{}", config.search.default_limit);
            println!("  输出格式：{}", config.output.default_format);
            println!("  自动索引：{}", config.index.auto_index);
            println!("  删除确认：{}", config.file_operations.delete_confirm);
            println!("  调试模式：{}", config.advanced.debug);
            
            if !config.aliases.is_empty() {
                println!();
                println!("已定义别名:");
                for (name, pattern) in &config.aliases {
                    println!("  {}: {}", name, pattern);
                }
            }
        }
        
        crate::ConfigAction::Edit => {
            let config_mgr = ConfigManager::load()?;
            let config_path = config_mgr.get_config_path();
            
            println!("配置文件：{}", config_path.display());
            println!("请使用编辑器打开此文件进行编辑");
            println!("例如：nano {}", config_path.display());
        }
        
        crate::ConfigAction::Reset => {
            let config_mgr = ConfigManager::load()?;
            
            // 创建默认配置
            let default_config = config::Config::default();
            let new_mgr = ConfigManager {
                config: default_config,
                config_path: config_mgr.get_config_path().clone(),
            };
            new_mgr.save()?;
            
            println!("配置已重置为默认值");
            println!("配置文件：{}", new_mgr.get_config_path().display());
        }
        
        crate::ConfigAction::Validate => {
            match ConfigManager::load() {
                Ok(config_mgr) => {
                    println!("配置文件有效");
                    println!("路径：{}", config_mgr.get_config_path().display());
                }
                Err(e) => {
                    eprintln!("配置文件无效：{}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}
