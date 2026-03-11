use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

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

/// 审计日志管理器
pub struct AuditLogger {
    log_file: PathBuf,
    enabled: bool,
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
