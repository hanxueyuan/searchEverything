/// Linux inotify 索引构建器
///
/// 使用 inotify 实时监控文件变更：
/// - 初始扫描建立索引
/// - inotify 监控实时更新
/// - 低开销，内核级通知
use super::{FileRecord, IndexBuilder, TrieIndex};
use anyhow::Result;
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

/// Linux 索引构建器
pub struct LinuxIndexBuilder {
    /// 是否使用 inotify
    use_inotify: bool,
}

impl LinuxIndexBuilder {
    pub fn new() -> Self {
        Self {
            use_inotify: cfg!(target_os = "linux"),
        }
    }

    /// 扫描目录建立索引
    fn scan_directory(&self, path: &Path, exclude_paths: &[String]) -> Result<Vec<FileRecord>> {
        use walkdir::WalkDir;

        let mut records = Vec::new();
        let mut file_id_counter: usize = 0;

        if should_exclude(&path.to_string_lossy(), exclude_paths) {
            return Ok(records);
        }

        tracing::debug!("扫描目录：{}", path.display());

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !should_exclude(&e.path().to_string_lossy(), exclude_paths))
            .filter_map(|e| e.ok())
        {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue, // 跳过无法访问的文件
            };

            let path_buf = entry.path().to_path_buf();

            let record = FileRecord {
                path: path_buf,
                name: entry.file_name().to_string_lossy().to_string(),
                size: if metadata.is_file() {
                    metadata.len()
                } else {
                    0
                },
                is_dir: metadata.is_dir(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                created: metadata.created().ok(),
                id: file_id_counter,
            };
            file_id_counter += 1;

            records.push(record);
        }

        Ok(records)
    }
}

impl IndexBuilder for LinuxIndexBuilder {
    fn build(&self, paths: &[PathBuf], exclude_paths: &[String]) -> Result<TrieIndex> {
        let mut index = TrieIndex::new();
        let start_time = std::time::Instant::now();

        // 初始扫描
        for path in paths {
            let records = self.scan_directory(path, exclude_paths)?;
            tracing::debug!("扫描到 {} 个文件", records.len());
            for record in records {
                index.add(record);
            }
        }

        // 更新统计信息
        let duration = start_time.elapsed();
        index.stats.built_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        index.stats.updated_at = index.stats.built_at.clone();
        index.stats.index_method = self.platform_name().to_string();
        index.stats.index_size_mb = (index.len() * 100) as f64 / 1024.0 / 1024.0;

        println!(
            "索引构建完成：{} 文件，{} 目录，耗时 {:.2}s",
            index.stats.total_files,
            index.stats.total_dirs,
            duration.as_secs_f64()
        );

        Ok(index)
    }

    fn platform_name(&self) -> &str {
        if cfg!(target_os = "linux") {
            "Linux inotify"
        } else {
            "Generic (scan)"
        }
    }

    fn supports_realtime(&self) -> bool {
        cfg!(target_os = "linux") && self.use_inotify
    }
}

impl Default for LinuxIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 启动 inotify 实时监控
pub fn start_watch(
    paths: &[PathBuf],
    exclude_paths: &[String],
    index: &mut TrieIndex,
) -> Result<()> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    println!("启动 inotify 实时监控...");
    println!("监控路径:");
    for path in paths {
        println!("  - {}", path.display());
    }
    println!("排除路径:");
    for path in exclude_paths {
        println!("  - {}", path);
    }
    println!("按 Ctrl+C 停止监控");

    // 创建停止标志
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 设置 Ctrl+C 处理
    let handler = move || {
        println!("\n停止监控...");
        r.store(false, Ordering::SeqCst);
    };
    ctrlc_handler(handler)?;

    // 创建通道
    let (tx, rx) = channel();

    // 创建 watcher
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    // 添加监控路径
    for path in paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            tracing::info!("开始监控：{}", path.display());
        }
    }

    // 处理事件
    while running.load(Ordering::SeqCst) {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            handle_event(event, index, exclude_paths);
        }

        // 定期保存（TODO: 实现持久化）
    }

    println!("监控已停止");
    Ok(())
}

/// 处理 inotify 事件
fn handle_event(event: Event, index: &mut TrieIndex, exclude_paths: &[String]) {
    let path = match event.paths.first() {
        Some(p) => p,
        None => return,
    };

    // 检查是否应该排除
    if should_exclude(&path.to_string_lossy(), exclude_paths) {
        return;
    }

    match event.kind {
        EventKind::Create(_) => {
            // 文件创建
            if let Ok(metadata) = std::fs::metadata(path) {
                let record = FileRecord {
                    path: path.clone(),
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    size: if metadata.is_file() {
                        metadata.len()
                    } else {
                        0
                    },
                    is_dir: metadata.is_dir(),
                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    created: metadata.created().ok(),
                    id: 0, // 将由索引分配
                };
                index.add(record);
                tracing::debug!("文件创建：{}", path.display());
            }
        }
        EventKind::Modify(_) => {
            // 文件修改
            tracing::debug!("文件修改：{}", path.display());
        }
        EventKind::Remove(_) => {
            // 文件删除
            index.remove(path);
            tracing::debug!("文件删除：{}", path.display());
        }
        _ => {}
    }
}

/// 设置 Ctrl+C 处理程序
fn ctrlc_handler<F>(f: F) -> Result<()>
where
    F: FnMut() + Send + 'static,
{
    ctrlc::set_handler(f)?;
    Ok(())
}

/// 检查路径是否应该被排除
pub fn should_exclude(path: &str, exclude_paths: &[String]) -> bool {
    let path_lower = path.to_lowercase();

    // 检查自定义排除路径
    for exclude in exclude_paths {
        let exclude_lower = exclude.to_lowercase();

        // 精确匹配
        if path_lower == exclude_lower {
            return true;
        }

        // 支持 **/ 通配符 (匹配任意深度的目录)
        if let Some(pattern) = exclude_lower.strip_prefix("**/") {
            if path_lower.contains(&format!("/{}", pattern)) || path_lower.ends_with(pattern) {
                return true;
            }
        }

        // 支持 * 通配符在末尾
        if exclude_lower.ends_with("*") {
            let prefix = &exclude_lower[..exclude_lower.len() - 1];
            if path_lower.starts_with(prefix) {
                return true;
            }
        }

        // 支持 *.ext 模式 (文件扩展名匹配)
        if exclude_lower.starts_with("*.") {
            let ext = &exclude_lower[1..]; // 包括 .
            if path_lower.ends_with(ext) {
                return true;
            }
        }

        // 路径前缀匹配 (排除目录及其子目录)
        if path_lower.starts_with(&format!("{}/", exclude_lower)) {
            return true;
        }
    }

    // Linux 默认排除系统目录
    let default_excludes = ["/proc", "/sys", "/dev", "/run", "/snap", "/lost+found"];

    default_excludes.iter().any(|&e| {
        path_lower == e
            || path_lower.starts_with(&format!("{}/", e))
            || path_lower.starts_with(&format!("{}\\", e))
    })
}

/// Linux 系统目录排除优化
pub fn get_linux_system_excludes() -> Vec<String> {
    vec![
        // 虚拟文件系统
        "/proc".to_string(),
        "/sys".to_string(),
        "/dev".to_string(),
        // 运行时数据
        "/run".to_string(),
        "/var/run".to_string(),
        // 快照
        "/snap".to_string(),
        // 文件系统检查
        "/lost+found".to_string(),
        // 临时文件
        "/tmp".to_string(),
        "/var/tmp".to_string(),
        // 缓存（可选排除）
        // "/var/cache".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_exclude_system_paths() {
        let exclude_paths: Vec<String> = vec![];

        assert!(should_exclude("/proc/1", &exclude_paths));
        assert!(should_exclude("/sys/class", &exclude_paths));
        assert!(should_exclude("/dev/null", &exclude_paths));
        assert!(should_exclude("/run/lock", &exclude_paths));
        assert!(should_exclude("/snap/core", &exclude_paths));
    }

    #[test]
    fn test_should_exclude_custom_paths() {
        let exclude_paths = vec![
            "/home/user/.cache".to_string(),
            "**/node_modules".to_string(),
            "/tmp/*".to_string(),
        ];

        assert!(should_exclude("/home/user/.cache/npm", &exclude_paths));
        assert!(should_exclude(
            "/home/project/node_modules/lodash",
            &exclude_paths
        ));
        assert!(should_exclude("/tmp/test.txt", &exclude_paths));

        assert!(!should_exclude("/home/user/documents", &exclude_paths));
    }

    #[test]
    fn test_get_linux_system_excludes() {
        let excludes = get_linux_system_excludes();

        assert!(excludes.contains(&"/proc".to_string()));
        assert!(excludes.contains(&"/sys".to_string()));
        assert!(excludes.contains(&"/dev".to_string()));
        assert!(excludes.contains(&"/run".to_string()));
    }
}
