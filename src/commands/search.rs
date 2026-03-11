use crate::error::SearchError;
use crate::output::{OutputFormat, SearchResult};
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use std::io::{self, Write};

pub fn execute(
    pattern: &str,
    path: &Path,
    limit: usize,
    format: &OutputFormat,
    _type_: &crate::FileType,
    use_regex: bool,
    use_fuzzy: bool,
    stream: bool,
) -> Result<()> {
    let mut results = Vec::new();
    
    // 编译正则表达式（如果需要）
    let regex_pattern = if use_regex {
        Some(Regex::new(pattern)?)
    } else {
        None
    };
    
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(limit * 10)  // 多取一些用于模糊排序
    {
        let file_name = entry.file_name().to_string_lossy();
        let mut matched = false;
        
        // 根据模式匹配
        if use_regex {
            // 正则模式
            if let Some(ref re) = regex_pattern {
                matched = re.is_match(&file_name);
            }
        } else if use_fuzzy {
            // 模糊搜索（简单实现：包含匹配）
            matched = file_name.to_lowercase().contains(&pattern.to_lowercase());
        } else {
            // 通配符模式（默认）
            matched = matches_pattern(&pattern.to_lowercase(), &file_name.to_lowercase());
        }
        
        if matched {
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
    
    // 模糊搜索时按相关度排序
    if use_fuzzy {
        results.sort_by(|a, b| {
            let a_name = a.path.to_lowercase();
            let b_name = b.path.to_lowercase();
            let pattern_lower = pattern.to_lowercase();
            
            // 包含匹配度排序
            let a_score = if a_name.starts_with(&pattern_lower) { 2 } 
                         else if a_name.contains(&pattern_lower) { 1 } 
                         else { 0 };
            let b_score = if b_name.starts_with(&pattern_lower) { 2 } 
                         else if b_name.contains(&pattern_lower) { 1 } 
                         else { 0 };
            
            b_score.cmp(&a_score)
        });
    }
    
    // 输出结果
    match format {
        OutputFormat::Text => {
            if stream {
                // 流式输出 - 找到一个输出一个
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                for result in &results {
                    writeln!(handle, "{}", result.path)?;
                    handle.flush()?;
                }
            } else {
                for result in &results {
                    println!("{}", result.path);
                }
            }
        }
        OutputFormat::Json => {
            if stream {
                // 流式 JSON - JSONL 格式 (每行一个 JSON 对象)
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                for result in &results {
                    let json_line = serde_json::to_string(result)?;
                    writeln!(handle, "{}", json_line)?;
                    handle.flush()?;
                }
            } else {
                let output = serde_json::json!({
                    "results": results,
                    "total": results.len(),
                    "error": null
                });
                println!("{}", output);
            }
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
