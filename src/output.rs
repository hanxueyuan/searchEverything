use serde::Serialize;
use clap::ValueEnum;
use std::io::{self, Write};

/// 搜索结果
#[derive(Serialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub size: u64,
    pub modified: String,
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
    
    /// 输出进度信息
    pub fn write_progress(&self, scanned: usize, found: usize) -> io::Result<()> {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        
        write!(handle, "\r已搜索：{} 个文件，找到：{} 个结果", scanned, found)?;
        handle.flush()?;
        
        Ok(())
    }
    
    /// 完成输出
    pub fn finish(&self) -> io::Result<()> {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        
        // 清除进度行
        write!(handle, "\r")?;
        
        // 显示总计
        eprintln!("搜索完成：共找到 {} 个结果", self.count);
        handle.flush()?;
        
        Ok(())
    }
    
    /// 获取已输出结果数
    pub fn count(&self) -> usize {
        self.count
    }
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
        use std::time::UNIX_EPOCH;
        
        Self {
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
            is_dir: metadata.is_dir(),
        }
    }
}
