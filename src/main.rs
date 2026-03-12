use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

mod audit;
mod commands;
mod config;
mod context;
mod error;
mod file_index;
pub mod output;
mod skill_test;
pub mod tests;

use audit::{AuditLogEntry, AuditLogger};
use commands::{cat, copy, delete, index as index_cmd, info, move_file, search};
use config::ConfigManager;
pub use output::OutputFormat;

#[derive(Parser)]
#[command(name = "searchEverything")]
#[command(about = "searchEverything - Local file search tool", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for files
    Search {
        /// Search pattern (supports wildcards * and ?)
        pattern: String,

        /// Search path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Maximum number of results
        #[arg(short, long, default_value = "100")]
        limit: usize,

        /// Output format
        #[arg(short, long, value_enum, default_value = "json")]
        format: output::OutputFormat,

        /// File type filter
        #[arg(short, long, value_enum, default_value = "both")]
        type_: FileType,

        /// Use regular expression
        #[arg(long)]
        regex: bool,

        /// Use fuzzy search
        #[arg(long)]
        fuzzy: bool,

        /// Streaming output (display results in real-time)
        #[arg(long)]
        stream: bool,

        /// Page size (number of results per page in streaming mode)
        #[arg(long)]
        page_size: Option<usize>,
    },

    /// View file information
    Info {
        /// File path
        file: PathBuf,

        /// JSON output
        #[arg(short, long)]
        json: bool,
    },

    /// Read file content
    Cat {
        /// File path
        file: PathBuf,

        /// Display number of lines
        #[arg(short, long)]
        lines: Option<usize>,

        /// Display last N lines (similar to tail)
        #[arg(long)]
        tail: bool,
    },

    /// Copy files
    Copy {
        /// Source file path (supports wildcards)
        source: String,

        /// Destination path
        dest: PathBuf,
    },

    /// Move files
    Move {
        /// Source file path
        source: String,

        /// Destination path
        dest: PathBuf,
    },

    /// Delete files
    Delete {
        /// File path (supports wildcards)
        path: String,

        /// Force delete without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Index management
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },

    /// Quick index status
    IndexStatus,

    /// Audit log management
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Skill test
    #[command(hide = true)]
    SkillTest,

    /// Display context information
    #[command(hide = true)]
    Context,
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Edit configuration file
    Edit,

    /// Reset to default configuration
    Reset,

    /// Validate configuration file
    Validate,
}

#[derive(Subcommand, Debug)]
enum AuditAction {
    /// List recent logs
    List {
        /// Maximum number to display
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Command filter
        #[arg(long)]
        command: Option<String>,
    },

    /// Search logs
    Search {
        /// Command filter
        #[arg(long)]
        command: Option<String>,

        /// Start time (RFC3339 format)
        #[arg(long)]
        after: Option<String>,

        /// End time (RFC3339 format)
        #[arg(long)]
        before: Option<String>,

        /// Result filter (success/error)
        #[arg(long)]
        result: Option<String>,

        /// Maximum number to display
        #[arg(short, long, default_value = "100")]
        limit: usize,
    },

    /// Export logs
    Export {
        /// Export format (json/csv)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Display statistics
    Stats,

    /// Clean old logs
    Cleanup {
        /// Retention days
        #[arg(long, default_value = "30")]
        days: u32,
    },
}

#[derive(Clone, ValueEnum, Debug)]
enum FileType {
    File,
    Dir,
    Both,
}

#[derive(Subcommand, Debug)]
enum IndexAction {
    /// Display index status
    Status,

    /// Rebuild index
    Rebuild {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Start real-time monitoring
    Watch {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Add index path
    Add {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// Remove index path
    Remove {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// List index paths
    List,

    /// Exclude path management
    Exclude {
        #[command(subcommand)]
        action: ExcludeAction,
    },
}

#[derive(Subcommand, Debug)]
enum ExcludeAction {
    /// Add exclude path
    Add {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Remove exclude path
    Remove {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// List exclude paths
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
    let _command_name = format!("{:?}", cli.command);

    match cli.command {
        Commands::Search {
            pattern,
            path,
            limit,
            format,
            type_,
            regex,
            fuzzy,
            stream,
            page_size,
        } => {
            let entry =
                AuditLogEntry::new("search", vec![pattern.clone(), path.display().to_string()]);
            search::execute(
                &pattern, &path, limit, &format, &type_, regex, fuzzy, stream, page_size,
            )?;
            audit_logger
                .log_success(&entry.with_duration(start_time.elapsed().as_millis() as u64))?;
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
        Commands::Audit { action } => {
            use commands::audit::AuditAction as AuditCmdAction;
            commands::audit::execute(&match action {
                crate::AuditAction::List { limit, command } => {
                    AuditCmdAction::List { limit, command }
                }
                crate::AuditAction::Search {
                    command,
                    after,
                    before,
                    result,
                    limit,
                } => AuditCmdAction::Search {
                    command,
                    after,
                    before,
                    result,
                    limit,
                },
                crate::AuditAction::Export { format, output } => {
                    AuditCmdAction::Export { format, output }
                }
                crate::AuditAction::Stats => AuditCmdAction::Stats,
                crate::AuditAction::Cleanup { days } => AuditCmdAction::Cleanup { days },
            })?;
        }
        Commands::SkillTest => {
            skill_test::run_skill_test()?;
        }
        Commands::Context => {
            context::print_context()?;
        }
    }

    Ok(())
}

/// Print JSON format help
#[allow(dead_code)]
fn print_help_json(command: &str) -> Result<()> {
    use serde_json::json;

    let help = if command.is_empty() {
        json!({
            "name": "searchEverything",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Cross-platform file search CLI tool",
            "commands": [
                {"name": "search", "description": "Search for files", "aliases": ["s"]},
                {"name": "info", "description": "View file information"},
                {"name": "cat", "description": "Read file content"},
                {"name": "copy", "description": "Copy files"},
                {"name": "move", "description": "Move files"},
                {"name": "delete", "description": "Delete files"},
                {"name": "index", "description": "Index management"},
                {"name": "config", "description": "Configuration management"},
                {"name": "context", "description": "Display context information"},
                {"name": "help", "description": "Display help"},
            ]
        })
    } else {
        // TODO: Return detailed help for specific command
        json!({
            "command": command,
            "description": "Command details",
            "usage": format!("searchEverything {} [OPTIONS]", command),
            "options": []
        })
    };

    println!("{}", serde_json::to_string_pretty(&help)?);
    Ok(())
}

/// Print text format help
#[allow(dead_code)]
fn print_help_text(command: &str) -> Result<()> {
    if command.is_empty() {
        println!("searchEverything v{}", env!("CARGO_PKG_VERSION"));
        println!("Cross-platform file search CLI tool\n");
        println!("Usage: searchEverything <COMMAND>\n");
        println!("Commands:");
        println!("  search    Search for files");
        println!("  info      View file information");
        println!("  cat       Read file content");
        println!("  copy      Copy files");
        println!("  move      Move files");
        println!("  delete    Delete files");
        println!("  index     Index management");
        println!("  config    Configuration management");
        println!("  context   Display context information");
        println!("  help      Display help");
        println!("\nUse 'searchEverything help <command>' for detailed command help");
    } else {
        println!("Command: {}", command);
        println!(
            "Use 'searchEverything {} --help' for detailed options",
            command
        );
    }

    Ok(())
}

/// Handle configuration command
fn handle_config(action: crate::ConfigAction) -> Result<()> {
    match action {
        crate::ConfigAction::Show => {
            let config_mgr = ConfigManager::load()?;
            let config = config_mgr.get();

            println!(
                "Configuration file path: {}",
                config_mgr.get_config_path().display()
            );
            println!();
            println!("Current configuration:");
            println!("  Search mode: {}", config.search.default_mode);
            println!("  Default result limit: {}", config.search.default_limit);
            println!("  Output format: {}", config.output.default_format);
            println!("  Auto index: {}", config.index.auto_index);
            println!(
                "  Delete confirmation: {}",
                config.file_operations.delete_confirm
            );
            println!("  Debug mode: {}", config.advanced.debug);

            if !config.aliases.is_empty() {
                println!();
                println!("Defined aliases:");
                for (name, pattern) in &config.aliases {
                    println!("  {}: {}", name, pattern);
                }
            }
        }

        crate::ConfigAction::Edit => {
            let config_mgr = ConfigManager::load()?;
            let config_path = config_mgr.get_config_path();

            println!("Configuration file: {}", config_path.display());
            println!("Please open this file in your editor");
            println!("For example: nano {}", config_path.display());
        }

        crate::ConfigAction::Reset => {
            let config_mgr = ConfigManager::load()?;

            // Create default configuration
            let default_config = config::Config::default();
            let new_mgr = ConfigManager {
                config: default_config,
                config_path: config_mgr.get_config_path().clone(),
            };
            new_mgr.save()?;

            println!("Configuration has been reset to defaults");
            println!(
                "Configuration file: {}",
                new_mgr.get_config_path().display()
            );
        }

        crate::ConfigAction::Validate => match ConfigManager::load() {
            Ok(config_mgr) => {
                println!("Configuration file is valid");
                println!("Path: {}", config_mgr.get_config_path().display());
            }
            Err(e) => {
                eprintln!("Configuration file is invalid: {}", e);
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
