#![allow(dead_code)]

/// 上下文信息管理模块
///
/// 提供命令执行的上下文数据，帮助大模型理解当前状态
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// 当前工作目录
    pub cwd: String,
    /// 当前用户
    pub user: String,
    /// 平台信息
    pub platform: PlatformInfo,
    /// 系统信息
    pub system: SystemInfo,
    /// 搜索上下文
    pub search: SearchContext,
    /// 建议列表
    pub suggestions: Vec<String>,
}

/// 平台信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// 操作系统
    pub os: String,
    /// 架构
    pub arch: String,
    /// 索引策略
    pub index_strategy: String,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU 核心数
    pub cpu_cores: usize,
    /// 总内存 (MB)
    pub total_memory_mb: u64,
    /// 可用内存 (MB)
    pub available_memory_mb: u64,
}

/// 搜索上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    /// 当前搜索模式
    pub mode: String,
    /// 默认路径
    pub default_paths: Vec<String>,
    /// 排除路径
    pub exclude_paths: Vec<String>,
    /// 索引统计
    pub index_stats: Option<IndexStats>,
}

/// 索引统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// 文件总数
    pub total_files: usize,
    /// 索引大小 (MB)
    pub index_size_mb: f64,
    /// 最后更新时间
    pub last_updated: String,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new() -> Result<Self> {
        Ok(Self {
            cwd: std::env::current_dir()?.to_string_lossy().to_string(),
            user: whoami::username(),
            platform: PlatformInfo::new(),
            system: SystemInfo::new()?,
            search: SearchContext::new(),
            suggestions: Vec::new(),
        })
    }

    /// 添加建议
    pub fn add_suggestion(&mut self, suggestion: &str) {
        self.suggestions.push(suggestion.to_string());
    }

    /// 生成错误建议
    pub fn generate_error_suggestions(&mut self, error: &str) {
        let error_lower = error.to_lowercase();

        if error_lower.contains("权限") || error_lower.contains("permission") {
            self.add_suggestion("使用 sudo 或以管理员身份运行");
            self.add_suggestion("检查文件权限设置");
        }

        if error_lower.contains("未找到") || error_lower.contains("not found") {
            self.add_suggestion("使用 searchEverything search 搜索文件");
            self.add_suggestion("检查路径是否正确");
        }

        if error_lower.contains("索引") || error_lower.contains("index") {
            self.add_suggestion("运行 searchEverything index rebuild 重建索引");
            self.add_suggestion("使用 searchEverything index status 查看索引状态");
        }
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new().expect("Failed to create ExecutionContext")
    }
}

impl PlatformInfo {
    pub fn new() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            index_strategy: get_index_strategy(),
        }
    }
}

impl SystemInfo {
    pub fn new() -> Result<Self> {
        Ok(Self {
            cpu_cores: num_cpus::get(),
            total_memory_mb: get_total_memory()?,
            available_memory_mb: get_available_memory().unwrap_or(0),
        })
    }
}

impl SearchContext {
    pub fn new() -> Self {
        Self {
            mode: "glob".to_string(),
            default_paths: vec![".".to_string()],
            exclude_paths: Vec::new(),
            index_stats: None,
        }
    }
}

/// 获取索引策略名称
fn get_index_strategy() -> String {
    #[cfg(target_os = "windows")]
    {
        "MFT/USN".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "inotify".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "FSEvents".to_string()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "generic".to_string()
    }
}

/// 获取总内存 (MB)
fn get_total_memory() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().unwrap_or(0);
                    return Ok(kb / 1024);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 sysctl
        // 简化实现，返回 0
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 GlobalMemoryStatusEx
        // 简化实现，返回 0
    }

    Ok(0)
}

/// 获取可用内存 (MB)
fn get_available_memory() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut mem_available = 0u64;

        for line in meminfo.lines() {
            if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().unwrap_or(0);
                    mem_available = kb / 1024;
                    break;
                }
            }
        }

        Ok(mem_available)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(0)
    }
}

/// 获取当前上下文（用于命令输出）
pub fn get_context_json() -> Result<String> {
    let ctx = ExecutionContext::new()?;
    ctx.to_json()
}

/// 打印上下文信息
pub fn print_context() -> Result<()> {
    let ctx = ExecutionContext::new()?;

    println!("执行上下文:");
    println!("  工作目录：{}", ctx.cwd);
    println!("  用户：{}", ctx.user);
    println!("  平台：{} {}", ctx.platform.os, ctx.platform.arch);
    println!("  索引策略：{}", ctx.platform.index_strategy);
    println!("  CPU 核心：{}", ctx.system.cpu_cores);
    println!("  总内存：{} MB", ctx.system.total_memory_mb);
    println!("  可用内存：{} MB", ctx.system.available_memory_mb);

    if !ctx.suggestions.is_empty() {
        println!("\n建议:");
        for suggestion in &ctx.suggestions {
            println!("  - {}", suggestion);
        }
    }

    Ok(())
}
