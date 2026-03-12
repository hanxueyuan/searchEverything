#![allow(dead_code)]

use clap::ValueEnum;
use serde::Serialize;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/// 搜索结果
#[derive(Serialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub size: u64,
    pub size_human: String,     // 新增：人类可读的文件大小
    pub modified: String,       // 修改为：ISO 8601 格式
    pub modified_human: String, // 新增：人类可读的时间
    pub file_type: String,      // 新增：文件类型
    pub is_dir: bool,
}

/// 输出格式
#[derive(Clone, ValueEnum, Serialize, Debug)]
pub enum OutputFormat {
    Text,
    Json,
}

/// 流式输出器
///
/// 支持边匹配边输出，减少用户等待时间
pub struct StreamOutput {
    /// 输出格式
    format: OutputFormat,
    /// 是否已开始输出
    started: bool,
    /// 已输出结果数
    count: usize,
    /// 页大小（用于分页）
    page_size: Option<usize>,
    /// 当前页结果数
    current_page_count: usize,
}

impl StreamOutput {
    /// 创建新的流式输出器
    pub fn new(format: OutputFormat, page_size: Option<usize>) -> Self {
        Self {
            format,
            started: false,
            count: 0,
            page_size,
            current_page_count: 0,
        }
    }

    /// 输出单个结果
    pub fn write(&mut self, result: &SearchResult) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        match &self.format {
            OutputFormat::Text => {
                writeln!(handle, "{}", result.path)?;
            }
            OutputFormat::Json => {
                // JSONL 格式：每行一个 JSON 对象
                let json_line = serde_json::to_string(result)?;
                writeln!(handle, "{}", json_line)?;
            }
        }

        handle.flush()?;

        self.count += 1;
        self.current_page_count += 1;

        // 检查是否需要分页暂停
        if let Some(page_size) = self.page_size {
            if self.current_page_count >= page_size {
                self.current_page_count = 0;
                // 提示用户继续
                eprint!("-- 按 Enter 继续，Ctrl+C 退出 --");
                let _ = io::stderr().flush();

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
            }
        }

        Ok(())
    }

    /// Output progress information
    pub fn write_progress(&self, scanned: usize, found: usize) -> io::Result<()> {
        let stderr = io::stderr();
        let mut handle = stderr.lock();

        write!(
            handle,
            "\rScanned: {} files, Found: {} results",
            scanned, found
        )?;
        handle.flush()?;

        Ok(())
    }

    /// Finish output
    pub fn finish(&self) -> io::Result<()> {
        let stderr = io::stderr();
        let mut handle = stderr.lock();

        // Clear progress line
        write!(handle, "\r")?;

        // Display total
        eprintln!("Search complete: found {} results", self.count);
        handle.flush()?;

        Ok(())
    }

    /// 获取已输出结果数
    pub fn count(&self) -> usize {
        self.count
    }
}

/// 格式化文件大小为人类可读格式
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    match bytes {
        b if b < KB => format!("{} B", b),
        b if b < MB => format!("{:.1} KB", b as f64 / KB as f64),
        b if b < GB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b < TB => format!("{:.1} GB", b as f64 / GB as f64),
        b => format!("{:.1} TB", b as f64 / TB as f64),
    }
}

/// Format time to human-readable format
fn format_time_human(secs_since_epoch: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let diff = now.saturating_sub(secs_since_epoch);

    match diff {
        0..=5 => "just now".to_string(),
        6..=59 => format!("{} seconds ago", diff),
        60..=3599 => format!("{} minutes ago", diff / 60),
        3600..=86399 => format!("{} hours ago", diff / 3600),
        86400..=604799 => format!("{} days ago", diff / 86400),
        604800..=2591999 => format!("{} weeks ago", diff / 604800),
        2592000..=31535999 => format!("{} months ago", diff / 2592000),
        _ => format!("{} years ago", diff / 31536000),
    }
}

/// 从路径提取文件类型（扩展名）
fn extract_file_type(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// 格式化时间为 ISO 8601 格式
fn format_iso8601(secs_since_epoch: u64) -> String {
    use chrono::{DateTime, Utc};

    DateTime::from_timestamp(secs_since_epoch as i64, 0)
        .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
        .unwrap_or_default()
}

impl SearchResult {
    pub fn format(&self, format: &OutputFormat) -> String {
        match format {
            OutputFormat::Text => self.path.clone(),
            OutputFormat::Json => serde_json::to_string(self).unwrap_or_default(),
        }
    }

    /// 从文件元数据创建 SearchResult
    pub fn from_path(path: &std::path::Path, metadata: &std::fs::Metadata) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let size = metadata.len();
        let is_dir = metadata.is_dir();

        // 获取修改时间
        let modified_secs = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            path: path_str.clone(),
            size,
            size_human: format_size(size),
            modified: format_iso8601(modified_secs),
            modified_human: format_time_human(modified_secs),
            file_type: if is_dir {
                "directory".to_string()
            } else {
                extract_file_type(&path_str)
            },
            is_dir,
        }
    }
}
