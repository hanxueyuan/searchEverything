use crate::output::{OutputFormat, SearchResult, StreamOutput};
use anyhow::Result;
use regex::Regex;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

/// Search execution function
///
/// Supports three search modes:
/// - Wildcard (default): *.pdf, report*.docx
/// - Regular expression: --regex ".*\.rs$"
/// - Fuzzy search: --fuzzy cargo
///
/// Supports streaming output:
/// - --stream: Display results in real-time
/// - --page-size N: Display N results per page
#[allow(clippy::too_many_arguments)]
pub fn execute(
    pattern: &str,
    path: &Path,
    limit: usize,
    format: &OutputFormat,
    _type_: &crate::FileType,
    use_regex: bool,
    use_fuzzy: bool,
    stream: bool,
    page_size: Option<usize>,
) -> Result<()> {
    let start_time = Instant::now();
    
    // Compile regex pattern (if needed)
    let regex_pattern = if use_regex {
        Some(Regex::new(pattern)?)
    } else {
        None
    };

    // Create streaming output
    let mut stream_output = StreamOutput::new(format.clone(), page_size);

    let mut results = Vec::new();
    let mut scanned = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        scanned += 1;

        let file_name = entry.file_name().to_string_lossy();
        let mut matched = false;

        // Match based on mode
        if use_regex {
            // Regex mode
            if let Some(ref re) = regex_pattern {
                matched = re.is_match(&file_name);
            }
        } else if use_fuzzy {
            // Fuzzy search (simple implementation: contains match)
            matched = file_name.to_lowercase().contains(&pattern.to_lowercase());
        } else {
            // Wildcard mode (default)
            matched = matches_pattern(&pattern.to_lowercase(), &file_name.to_lowercase());
        }

        if matched {
            let metadata = entry.metadata()?;
            let result = SearchResult::from_path(entry.path(), &metadata);

            if stream {
                // Streaming output: display results immediately
                stream_output.write(&result)?;
            } else {
                // Batch output: collect results first
                results.push(result);
            }

            // Check if limit reached
            if stream {
                if stream_output.count() >= limit {
                    break;
                }
            } else {
                if results.len() >= limit {
                    break;
                }
            }
        }

        // Output progress periodically (every 1000 files)
        if stream && scanned % 1000 == 0 {
            stream_output.write_progress(scanned, stream_output.count())?;
        }
    }

    // If not streaming output, output all results now
    if !stream {
        // Sort by relevance for fuzzy search
        if use_fuzzy {
            results.sort_by(|a, b| {
                let a_name = a.path.to_lowercase();
                let b_name = b.path.to_lowercase();
                let pattern_lower = pattern.to_lowercase();

                // Containment relevance sort
                let a_score = if a_name.starts_with(&pattern_lower) {
                    2
                } else if a_name.contains(&pattern_lower) {
                    1
                } else {
                    0
                };
                let b_score = if b_name.starts_with(&pattern_lower) {
                    2
                } else if b_name.contains(&pattern_lower) {
                    1
                } else {
                    0
                };

                b_score.cmp(&a_score)
            });
        }

        // Output results
        match format {
            OutputFormat::Text => {
                for result in &results {
                    println!("{}", result.path);
                }
            }
            OutputFormat::Json => {
                // Calculate summary information
                let total_size: u64 = results.iter().map(|r| r.size).sum();
                let file_types: std::collections::HashMap<String, usize> = {
                    let mut map = std::collections::HashMap::new();
                    for r in &results {
                        *map.entry(r.file_type.clone()).or_insert(0) += 1;
                    }
                    map
                };
                let file_types_summary: Vec<String> = file_types
                    .iter()
                    .map(|(ft, count)| format!("{}: {}", ft, count))
                    .collect();
                
                let search_time_ms = start_time.elapsed().as_millis() as u64;
                
                let output = serde_json::json!({
                    "results": results,
                    "total": results.len(),
                    "scanned": scanned,
                    "search_time_ms": search_time_ms,
                    "summary": {
                        "total_size": total_size,
                        "total_size_human": crate::output::format_size(total_size),
                        "file_types": file_types_summary.join(", "),
                        "oldest_file": results.iter().min_by_key(|r| r.modified.clone()).map(|r| r.path.clone()).unwrap_or_default(),
                        "newest_file": results.iter().max_by_key(|r| r.modified.clone()).map(|r| r.path.clone()).unwrap_or_default()
                    },
                    "error": null
                });
                println!("{}", output);
            }
        }

        eprintln!(
            "Search complete: found {} results (scanned {} files)",
            results.len(),
            scanned
        );
    } else {
        // Streaming output complete
        stream_output.finish()?;
    }

    Ok(())
}

fn matches_pattern(pattern: &str, text: &str) -> bool {
    // 简单的通配符匹配实现
    if pattern == "*" {
        return true;
    }

    if let Some(inner) = pattern.strip_prefix('*').and_then(|p| p.strip_suffix('*')) {
        return text.contains(inner);
    }

    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }

    // 支持？通配符
    if pattern.contains('?') {
        return glob_match(pattern, text);
    }

    pattern == text
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars = pattern.chars();
    let mut text_chars = text.chars();

    for p_char in pattern_chars {
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
