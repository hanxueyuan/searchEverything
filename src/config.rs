use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 搜索配置
    #[serde(default)]
    pub search: SearchConfig,

    /// 输出配置
    #[serde(default)]
    pub output: OutputConfig,

    /// 索引配置
    #[serde(default)]
    pub index: IndexConfig,

    /// 文件操作配置
    #[serde(default)]
    pub file_operations: FileOperationsConfig,

    /// 性能配置
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// 别名配置
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,

    /// OpenClaw 配置
    #[serde(default)]
    pub openclaw: OpenClawConfig,

    /// 高级配置
    #[serde(default)]
    pub advanced: AdvancedConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search: SearchConfig::default(),
            output: OutputConfig::default(),
            index: IndexConfig::default(),
            file_operations: FileOperationsConfig::default(),
            performance: PerformanceConfig::default(),
            aliases: std::collections::HashMap::new(),
            openclaw: OpenClawConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 默认搜索模式：glob, regex, fuzzy
    #[serde(default = "default_mode")]
    pub default_mode: String,

    /// 默认最大结果数
    #[serde(default = "default_limit")]
    pub default_limit: usize,

    /// 默认搜索路径
    #[serde(default = "default_paths")]
    pub default_paths: Vec<String>,

    /// 是否区分大小写
    #[serde(default)]
    pub case_sensitive: bool,

    /// 是否包含隐藏文件
    #[serde(default)]
    pub include_hidden: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_mode: default_mode(),
            default_limit: default_limit(),
            default_paths: default_paths(),
            case_sensitive: false,
            include_hidden: false,
        }
    }
}

fn default_mode() -> String {
    "glob".to_string()
}
fn default_limit() -> usize {
    100
}
fn default_paths() -> Vec<String> {
    vec![".".to_string()]
}

/// 输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 默认输出格式：json, text
    #[serde(default = "default_format")]
    pub default_format: String,

    /// JSON 输出是否美化
    #[serde(default)]
    pub pretty_print: bool,

    /// 是否显示文件详细信息
    #[serde(default = "default_true")]
    pub show_details: bool,

    /// 时间格式：iso, unix, local
    #[serde(default = "default_time_format")]
    pub time_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: default_format(),
            pretty_print: false,
            show_details: default_true(),
            time_format: default_time_format(),
        }
    }
}

fn default_format() -> String {
    "json".to_string()
}
fn default_time_format() -> String {
    "iso".to_string()
}
fn default_true() -> bool {
    true
}

/// 索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// 是否启用自动索引
    #[serde(default = "default_true")]
    pub auto_index: bool,

    /// 索引文件路径
    #[serde(default = "default_index_file")]
    pub index_file: String,

    /// 索引更新策略：auto, manual, interval
    #[serde(default = "default_update_strategy")]
    pub update_strategy: String,

    /// 定时更新间隔（秒）
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,

    /// 索引路径
    #[serde(default)]
    pub paths: Vec<String>,

    /// 排除路径
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            auto_index: default_true(),
            index_file: default_index_file(),
            update_strategy: default_update_strategy(),
            update_interval: default_update_interval(),
            paths: vec![],
            exclude_paths: vec!["/proc".to_string(), "/sys".to_string(), "/dev".to_string()],
        }
    }
}

fn default_index_file() -> String {
    "~/.config/searchEverything/index.bin".to_string()
}
fn default_update_strategy() -> String {
    "auto".to_string()
}
fn default_update_interval() -> u64 {
    300
}

/// 文件操作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationsConfig {
    /// 删除操作是否需要确认
    #[serde(default = "default_true")]
    pub delete_confirm: bool,

    /// 覆盖文件前是否需要确认
    #[serde(default = "default_true")]
    pub overwrite_confirm: bool,

    /// 批量操作最大文件数
    #[serde(default = "default_batch_limit")]
    pub batch_limit: usize,

    /// 是否记录操作日志
    #[serde(default = "default_true")]
    pub enable_log: bool,

    /// 日志文件路径
    #[serde(default = "default_log_file")]
    pub log_file: String,
}

impl Default for FileOperationsConfig {
    fn default() -> Self {
        Self {
            delete_confirm: default_true(),
            overwrite_confirm: default_true(),
            batch_limit: default_batch_limit(),
            enable_log: default_true(),
            log_file: default_log_file(),
        }
    }
}

fn default_batch_limit() -> usize {
    10
}
fn default_log_file() -> String {
    "~/.config/searchEverything/operations.log".to_string()
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 并发线程数（0 表示自动）
    #[serde(default)]
    pub threads: usize,

    /// 最大内存使用（MB）
    #[serde(default = "default_max_memory")]
    pub max_memory_mb: usize,

    /// 是否使用内存映射
    #[serde(default = "default_true")]
    pub use_mmap: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            threads: 0,
            max_memory_mb: default_max_memory(),
            use_mmap: default_true(),
        }
    }
}

fn default_max_memory() -> usize {
    512
}

/// OpenClaw 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    /// 是否启用 Skill 集成
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Skill 名称
    #[serde(default = "default_skill_name")]
    pub skill_name: String,

    /// 命令前缀
    #[serde(default = "default_command_prefix")]
    pub command_prefix: String,

    /// 是否记录调用日志
    #[serde(default = "default_true")]
    pub log_invocations: bool,
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            skill_name: default_skill_name(),
            command_prefix: default_command_prefix(),
            log_invocations: default_true(),
        }
    }
}

fn default_skill_name() -> String {
    "searchEverything".to_string()
}
fn default_command_prefix() -> String {
    "searchEverything".to_string()
}

/// 高级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// 是否启用调试模式
    #[serde(default)]
    pub debug: bool,

    /// 是否显示详细错误信息
    #[serde(default = "default_true")]
    pub verbose_errors: bool,

    /// 正则表达式引擎：rust, pcre2
    #[serde(default = "default_regex_engine")]
    pub regex_engine: String,

    /// 是否启用实验性功能
    #[serde(default)]
    pub experimental: bool,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            debug: false,
            verbose_errors: default_true(),
            regex_engine: default_regex_engine(),
            experimental: false,
        }
    }
}

fn default_regex_engine() -> String {
    "rust".to_string()
}

/// 配置管理器
pub struct ConfigManager {
    pub config: Config,
    pub config_path: PathBuf,
}

impl ConfigManager {
    /// 加载配置
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_yaml::from_str(&content)?
        } else {
            // 创建默认配置
            let config = Config::default();
            save_config_to(&config, &config_path)?;
            config
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    /// 获取配置
    pub fn get(&self) -> &Config {
        &self.config
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        save_config_to(&self.config, &self.config_path)
    }

    /// 获取配置路径
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

/// 保存配置到文件
fn save_config_to(config: &Config, path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_yaml::to_string(&config)?;

    // 添加配置说明头部
    let header = "# searchEverything 配置文件\n# 编辑此文件来自定义搜索行为、索引设置等\n\n";
    let full_content = format!("{}{}", header, content);

    fs::write(path, full_content)?;
    Ok(())
}

fn get_config_path() -> PathBuf {
    let config_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    };

    std::path::PathBuf::from(config_dir)
        .join(".config")
        .join("searchEverything")
        .join("config.yaml")
}
