use anyhow::Result;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Format file size to human-readable format
fn format_size(bytes: u64) -> String {
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

/// Format time to ISO 8601 format
fn format_iso8601(secs_since_epoch: u64) -> String {
    use chrono::{DateTime, Utc};
    
    DateTime::from_timestamp(secs_since_epoch as i64, 0)
        .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
        .unwrap_or_default()
}

pub fn execute(file: &Path, json: bool) -> Result<()> {
    let metadata = std::fs::metadata(file)?;
    
    // Get file type
    let file_type = if metadata.is_dir() {
        "directory".to_string()
    } else {
        file.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase()
    };

    // Get timestamps
    let modified = metadata.modified()?;
    let modified_secs = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let modified_iso = format_iso8601(modified_secs);
    let modified_human = format_time_human(modified_secs);
    
    // Format size
    let size = metadata.len();
    let size_human = format_size(size);

    let info = serde_json::json!({
        "path": file.to_string_lossy(),
        "size": size,
        "formatted_size": size_human,
        "is_dir": metadata.is_dir(),
        "is_file": metadata.is_file(),
        "modified": modified_iso,
        "formatted_modified": modified_human,
        "file_type": file_type,
        "created": format!("{:?}", metadata.created()?),
        "accessed": format!("{:?}", metadata.accessed()?),
    });

    if json {
        println!("{}", info);
    } else {
        println!("Path: {}", file.to_string_lossy());
        println!("Size: {} ({})", size, size_human);
        println!("Type: {}", if metadata.is_dir() { "Directory" } else { "File" });
        println!("Modified: {} ({})", modified_iso, modified_human);
        println!();
        println!("Summary:");
        println!("  File type: {}", file_type);
        println!("  Size: {} ({})", size, size_human);
        println!("  Modified: {} ({})", modified_iso, modified_human);
    }

    Ok(())
}
