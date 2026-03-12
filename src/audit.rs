#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// 时间戳
    pub timestamp: String,
    /// 命令名称
    pub command: String,
    /// 命令参数
    pub arguments: Vec<String>,
    /// 执行结果
    pub result: String,
    /// 影响的文件数
    pub files_affected: Option<usize>,
    /// 执行时间 (毫秒)
    pub duration_ms: Option<u64>,
    /// 用户
    pub user: String,
    /// 工作目录
    pub cwd: String,
}

impl AuditLogEntry {
    pub fn new(command: &str, arguments: Vec<String>) -> Self {
        Self {
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
            command: command.to_string(),
            arguments,
            result: "success".to_string(),
            files_affected: None,
            duration_ms: None,
            user: whoami::username(),
            cwd: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }

    pub fn with_result(mut self, result: &str) -> Self {
        self.result = result.to_string();
        self
    }

    pub fn with_files_affected(mut self, count: usize) -> Self {
        self.files_affected = Some(count);
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }
}

/// 日志查询条件
#[derive(Debug, Clone, Default)]
pub struct LogQuery {
    /// 命令名称过滤
    pub command: Option<String>,
    /// 开始时间
    pub after: Option<DateTime<Local>>,
    /// 结束时间
    pub before: Option<DateTime<Local>>,
    /// 结果过滤（success/error）
    pub result: Option<String>,
    /// 最大返回数量
    pub limit: usize,
}

/// 审计日志统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// 总日志数
    pub total_entries: usize,
    /// 成功次数
    pub success_count: usize,
    /// 失败次数
    pub error_count: usize,
    /// 各命令执行次数
    pub command_counts: std::collections::HashMap<String, usize>,
    /// 平均执行时间 (ms)
    pub avg_duration_ms: Option<f64>,
    /// 总执行时间 (ms)
    pub total_duration_ms: u64,
}

/// 审计日志管理器
pub struct AuditLogger {
    log_file: PathBuf,
    enabled: bool,
    /// 日志文件最大大小（字节），0 表示不限制
    max_size: u64,
    /// 日志保留天数，0 表示不限制
    retention_days: u32,
}

impl AuditLogger {
    pub fn new(log_file: PathBuf, enabled: bool) -> Self {
        // 确保日志目录存在
        if let Some(parent) = log_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        Self {
            log_file,
            enabled,
            max_size: 10 * 1024 * 1024, // 默认 10MB
            retention_days: 30,         // 默认保留 30 天
        }
    }

    /// 创建带配置的审计日志器
    pub fn with_config(
        log_file: PathBuf,
        enabled: bool,
        max_size: u64,
        retention_days: u32,
    ) -> Self {
        if let Some(parent) = log_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        Self {
            log_file,
            enabled,
            max_size,
            retention_days,
        }
    }

    /// 记录日志
    pub fn log(&self, entry: &AuditLogEntry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let json = serde_json::to_string(entry)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        writeln!(file, "{}", json)?;

        // 检查是否需要轮转
        self.maybe_rotate()?;

        Ok(())
    }

    /// 记录成功执行
    pub fn log_success(&self, entry: &AuditLogEntry) -> Result<()> {
        self.log(&entry.clone().with_result("success"))
    }

    /// 记录失败执行
    pub fn log_error(&self, entry: &AuditLogEntry, error: &str) -> Result<()> {
        self.log(&entry.clone().with_result(&format!("error: {}", error)))
    }

    /// 获取日志文件路径
    pub fn get_log_file(&self) -> &PathBuf {
        &self.log_file
    }

    /// 查询日志
    pub fn query(&self, query: &LogQuery) -> Result<Vec<AuditLogEntry>> {
        if !self.log_file.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_file)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // 解析日志条目
            let entry: AuditLogEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue, // 跳过无效的日志行
            };

            // 应用过滤条件
            if !self.matches_query(&entry, query) {
                continue;
            }

            results.push(entry);

            // 检查数量限制
            if results.len() >= query.limit {
                break;
            }
        }

        // 按时间倒序排序（最新的在前）
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(results)
    }

    /// 检查日志条目是否匹配查询条件
    fn matches_query(&self, entry: &AuditLogEntry, query: &LogQuery) -> bool {
        // 命令过滤
        if let Some(ref cmd) = query.command {
            if entry.command != *cmd {
                return false;
            }
        }

        // 时间过滤
        if let Some(after) = query.after {
            let entry_time = DateTime::parse_from_rfc3339(&entry.timestamp)
                .map(|t| t.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now());
            if entry_time < after {
                return false;
            }
        }

        if let Some(before) = query.before {
            let entry_time = DateTime::parse_from_rfc3339(&entry.timestamp)
                .map(|t| t.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now());
            if entry_time > before {
                return false;
            }
        }

        // 结果过滤
        if let Some(ref result) = query.result {
            if !entry.result.contains(result) {
                return false;
            }
        }

        true
    }

    /// 导出日志
    pub fn export(&self, format: &str, output_path: &PathBuf) -> Result<()> {
        if !self.log_file.exists() {
            anyhow::bail!("日志文件不存在");
        }

        match format {
            "json" => {
                // JSON 数组格式
                let entries = self.query(&LogQuery {
                    limit: 10000,
                    ..Default::default()
                })?;
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "exported_at": Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                    "total": entries.len(),
                    "entries": entries
                }))?;
                fs::write(output_path, json)?;
            }
            "csv" => {
                // CSV 格式
                let mut output = File::create(output_path)?;
                writeln!(
                    output,
                    "timestamp,command,arguments,result,files_affected,duration_ms,user,cwd"
                )?;

                let entries = self.query(&LogQuery {
                    limit: 10000,
                    ..Default::default()
                })?;
                for entry in entries {
                    let args = entry.arguments.join(";");
                    writeln!(
                        output,
                        "{},{},{},{},{},{},{},{}",
                        entry.timestamp,
                        entry.command,
                        args.replace(',', ";"),
                        entry.result,
                        entry.files_affected.unwrap_or(0),
                        entry.duration_ms.unwrap_or(0),
                        entry.user,
                        entry.cwd.replace(',', ";")
                    )?;
                }
            }
            _ => {
                anyhow::bail!("不支持的导出格式：{}", format);
            }
        }

        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> Result<AuditStats> {
        if !self.log_file.exists() {
            return Ok(AuditStats {
                total_entries: 0,
                success_count: 0,
                error_count: 0,
                command_counts: std::collections::HashMap::new(),
                avg_duration_ms: None,
                total_duration_ms: 0,
            });
        }

        let file = File::open(&self.log_file)?;
        let reader = BufReader::new(file);

        let mut stats = AuditStats {
            total_entries: 0,
            success_count: 0,
            error_count: 0,
            command_counts: std::collections::HashMap::new(),
            avg_duration_ms: None,
            total_duration_ms: 0,
        };

        let mut total_duration = 0u64;
        let mut duration_count = 0u64;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditLogEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            stats.total_entries += 1;

            if entry.result == "success" {
                stats.success_count += 1;
            } else {
                stats.error_count += 1;
            }

            *stats
                .command_counts
                .entry(entry.command.clone())
                .or_insert(0) += 1;

            if let Some(duration) = entry.duration_ms {
                total_duration += duration;
                duration_count += 1;
            }
        }

        stats.total_duration_ms = total_duration;
        if duration_count > 0 {
            stats.avg_duration_ms = Some(total_duration as f64 / duration_count as f64);
        }

        Ok(stats)
    }

    /// 清理旧日志
    pub fn cleanup_old_logs(&self) -> Result<usize> {
        if !self.log_file.exists() {
            return Ok(0);
        }

        if self.retention_days == 0 {
            return Ok(0); // 不限制保留时间
        }

        let cutoff = Local::now() - chrono::Duration::days(self.retention_days as i64);
        let mut removed = 0;

        // 读取所有日志
        let file = File::open(&self.log_file)?;
        let reader = BufReader::new(file);
        let mut kept_entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditLogEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // 检查时间
            let entry_time = DateTime::parse_from_rfc3339(&entry.timestamp)
                .map(|t| t.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now());

            if entry_time >= cutoff {
                kept_entries.push(line);
            } else {
                removed += 1;
            }
        }

        // 写回保留的日志
        if removed > 0 {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&self.log_file)?;

            for line in kept_entries {
                writeln!(file, "{}", line)?;
            }
        }

        Ok(removed)
    }

    /// 检查是否需要日志轮转
    fn maybe_rotate(&self) -> Result<()> {
        if self.max_size == 0 {
            return Ok(()); // 不限制大小
        }

        if let Ok(metadata) = fs::metadata(&self.log_file) {
            if metadata.len() > self.max_size {
                // 轮转日志
                let rotated_path = format!("{}.1", self.log_file.display());
                fs::rename(&self.log_file, &rotated_path)?;
                tracing::info!(
                    "日志已轮转：{} -> {}",
                    self.log_file.display(),
                    rotated_path
                );
            }
        }

        Ok(())
    }
}

/// 获取默认日志文件路径
pub fn get_default_log_path() -> PathBuf {
    let config_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    };

    std::path::PathBuf::from(config_dir)
        .join(".config")
        .join("searchEverything")
        .join("audit.log")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry() {
        let entry = AuditLogEntry::new("search", vec!["*.txt".to_string()]);

        assert_eq!(entry.command, "search");
        assert_eq!(entry.arguments.len(), 1);
        assert_eq!(entry.result, "success");
    }

    #[test]
    fn test_audit_logger_log_and_query() {
        use std::time::SystemTime;

        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join(format!(
            "audit_test_{}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        let logger = AuditLogger::new(log_file.clone(), true);

        // 记录一些日志
        let entry1 = AuditLogEntry::new("search", vec!["*.txt".to_string()]);
        logger.log_success(&entry1).unwrap();

        let entry2 = AuditLogEntry::new("info", vec!["/test".to_string()]);
        logger.log_success(&entry2).unwrap();

        // 查询日志
        let query = LogQuery {
            limit: 10,
            ..Default::default()
        };
        let results = logger.query(&query).unwrap();

        assert!(results.len() >= 2);

        // 清理
        fs::remove_file(&log_file).ok();
    }

    #[test]
    fn test_audit_stats() {
        use std::time::SystemTime;

        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join(format!(
            "audit_stats_{}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        let logger = AuditLogger::new(log_file.clone(), true);

        // 记录一些日志
        let entry1 = AuditLogEntry::new("search", vec![]).with_duration(100);
        logger.log_success(&entry1).unwrap();

        let entry2 = AuditLogEntry::new("search", vec![]).with_duration(200);
        logger.log_success(&entry2).unwrap();

        let entry3 = AuditLogEntry::new("info", vec![]).with_duration(50);
        logger.log_success(&entry3).unwrap();

        // 获取统计
        let stats = logger.get_stats().unwrap();

        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.success_count, 3);
        assert_eq!(stats.error_count, 0);
        assert!(stats.avg_duration_ms.is_some());

        // 清理
        fs::remove_file(&log_file).ok();
    }
}
