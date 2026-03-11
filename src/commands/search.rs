use crate::error::SearchError;
use crate::output::{OutputFormat, SearchResult};
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

pub fn execute(
    pattern: &str,
    path: &Path,
    limit: usize,
    format: &OutputFormat,
    _type_: &crate::FileType,
) -> Result<()> {
    let mut results = Vec::new();
    
    // 简单的 glob 匹配
    let pattern_lower = pattern.to_lowercase();
    
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(limit * 2)
    {
        let file_name = entry.file_name().to_string_lossy();
        
        // 简单的通配符匹配
        if matches_pattern(&pattern_lower, &file_name.to_lowercase()) {
            let metadata = entry.metadata()?;
            results.push(SearchResult {
                path: entry.path().to_string_lossy().to_string(),
                size: metadata.len(),
                modified: format!("{}", entry.metadata()?.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs()),
                is_dir: metadata.is_dir(),
            });
            
            if results.len() >= limit {
                break;
            }
        }
    }
    
    // 输出结果
    match format {
        OutputFormat::Text => {
            for result in &results {
                println!("{}", result.path);
            }
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "results": results,
                "total": results.len(),
                "error": null
            });
            println!("{}", output);
        }
    }
    
    Ok(())
}

fn matches_pattern(pattern: &str, text: &str) -> bool {
    // 简单的通配符匹配实现
    if pattern == "*" {
        return true;
    }
    
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let inner = &pattern[1..pattern.len() - 1];
        return text.contains(inner);
    }
    
    if pattern.starts_with('*') {
        return text.ends_with(&pattern[1..]);
    }
    
    if pattern.ends_with('*') {
        return text.starts_with(&pattern[..pattern.len() - 1]);
    }
    
    // 支持？通配符
    if pattern.contains('?') {
        return glob_match(pattern, text);
    }
    
    pattern == text
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars();
    let mut text_chars = text.chars();
    
    while let Some(p_char) = pattern_chars.next() {
        match p_char {
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            '*' => {
                // 简化处理
                return true;
            }
            t_char => {
                if Some(t_char) != text_chars.next() {
                    return false;
                }
            }
        }
    }
    
    text_chars.next().is_none()
}
