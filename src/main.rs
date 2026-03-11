use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

mod commands;
mod error;
mod output;

use commands::{search, info, cat, copy, move_file, delete, index};

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

#[derive(Subcommand)]
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
        #[arg(short, long, value_enum, default_value = "text")]
        format: OutputFormat,
        
        /// 文件类型过滤
        #[arg(short, long, value_enum, default_value = "both")]
        type_: FileType,
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
}

#[derive(Clone, ValueEnum, Serialize)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, ValueEnum)]
enum FileType {
    File,
    Dir,
    Both,
}

#[derive(Subcommand)]
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 初始化日志
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
    
    match cli.command {
        Commands::Search { pattern, path, limit, format, type_ } => {
            search::execute(&pattern, &path, limit, &format, &type_)?;
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
            index::execute(&action)?;
        }
    }
    
    Ok(())
}
