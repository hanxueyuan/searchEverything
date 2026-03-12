use crate::audit::{get_default_log_path, AuditLogger, LogQuery};
/// 审计日志管理命令
///
/// 功能：
/// - list: 列出最近的日志
/// - search: 搜索日志
/// - export: 导出日志
/// - stats: 显示统计信息
/// - cleanup: 清理旧日志
use anyhow::Result;
use std::path::PathBuf;

/// 审计子命令
pub enum AuditAction {
    /// 列出日志
    List {
        /// 最大显示数量
        limit: usize,
        /// 命令过滤
        command: Option<String>,
    },
    /// 搜索日志
    Search {
        /// 命令过滤
        command: Option<String>,
        /// 开始时间
        after: Option<String>,
        /// 结束时间
        before: Option<String>,
        /// 结果过滤
        result: Option<String>,
        /// 最大显示数量
        limit: usize,
    },
    /// 导出日志
    Export {
        /// 导出格式（json/csv）
        format: String,
        /// 输出文件路径
        output: PathBuf,
    },
    /// 显示统计信息
    Stats,
    /// 清理旧日志
    Cleanup {
        /// 保留天数
        days: u32,
    },
}

/// 执行审计命令
pub fn execute(action: &AuditAction) -> Result<()> {
    let config = crate::config::ConfigManager::load()?;
    let logger = AuditLogger::with_config(
        get_default_log_path(),
        config.get().file_operations.enable_log,
        10 * 1024 * 1024, // 10MB
        30,               // 30 天
    );

    match action {
        AuditAction::List { limit, command } => {
            let query = LogQuery {
                command: command.clone(),
                limit: *limit,
                ..Default::default()
            };

            let entries = logger.query(&query)?;

            if entries.is_empty() {
                println!("没有找到日志记录");
                return Ok(());
            }

            println!("审计日志 (共 {} 条):\n", entries.len());
            println!("{:-<100}", "");

            for entry in entries {
                println!("[{}] {} {}", entry.timestamp, entry.command, entry.result);
                if !entry.arguments.is_empty() {
                    println!("  参数：{}", entry.arguments.join(" "));
                }
                if let Some(duration) = entry.duration_ms {
                    println!("  耗时：{}ms", duration);
                }
                if let Some(files) = entry.files_affected {
                    println!("  影响文件：{}", files);
                }
                println!();
            }
        }

        AuditAction::Search {
            command,
            after,
            before,
            result,
            limit,
        } => {
            let query = LogQuery {
                command: command.clone(),
                after: after.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .map(|t| t.with_timezone(&chrono::Local))
                        .ok()
                }),
                before: before.as_ref().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .map(|t| t.with_timezone(&chrono::Local))
                        .ok()
                }),
                result: result.clone(),
                limit: *limit,
            };

            let entries = logger.query(&query)?;

            if entries.is_empty() {
                println!("没有找到匹配的日志记录");
                return Ok(());
            }

            println!("搜索到 {} 条日志:\n", entries.len());

            for entry in entries {
                println!("[{}] {} - {}", entry.timestamp, entry.command, entry.result);
            }
        }

        AuditAction::Export { format, output } => {
            logger.export(format, output)?;
            println!("日志已导出到：{}", output.display());
        }

        AuditAction::Stats => {
            let stats = logger.get_stats()?;

            println!("审计日志统计");
            println!("{}", "=".repeat(50));
            println!("总记录数：{}", stats.total_entries);
            println!("成功：{}", stats.success_count);
            println!("失败：{}", stats.error_count);

            if stats.total_entries > 0 {
                let success_rate =
                    (stats.success_count as f64 / stats.total_entries as f64) * 100.0;
                println!("成功率：{:.1}%", success_rate);
            }

            if let Some(avg) = stats.avg_duration_ms {
                println!("平均耗时：{:.2}ms", avg);
            }

            println!("\n命令分布:");
            let mut commands: Vec<_> = stats.command_counts.iter().collect();
            commands.sort_by(|a, b| b.1.cmp(a.1));

            for (cmd, count) in commands {
                println!("  {}: {} 次", cmd, count);
            }
        }

        AuditAction::Cleanup { days } => {
            let removed = logger.cleanup_old_logs()?;
            println!("已清理 {} 条旧日志（保留 {} 天）", removed, days);
        }
    }

    Ok(())
}
