use crate::output::{OutputFormat, SearchResult, StreamOutput};
use anyhow::Result;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

/// 搜索执行函数
///
/// 支持三种搜索模式：
/// - 通配符（默认）：*.pdf, report*.docx
/// - 正则表达式：--regex ".*\.rs$"
/// - 模糊搜索：--fuzzy cargo
///
/// 支持流式输出：
/// - --stream：边匹配边输出
/// - --page-size N：每页显示 N 条结果
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
    // 编译正则表达式（如果需要）
    let regex_pattern = if use_regex {
        Some(Regex::new(pattern)?)
    } else {
        None
    };

    // 创建流式输出器
    let mut stream_output = StreamOutput::new(format.clone(), page_size);

    let mut results = Vec::new();
    let mut scanned = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        scanned += 1;

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
            let result = SearchResult::from_path(entry.path(), &metadata);

            if stream {
                // 流式输出：立即输出结果
                stream_output.write(&result)?;
            } else {
                // 批量输出：先收集结果
                results.push(result);
            }

            // 检查是否达到限制
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

        // 定期输出进度（每 1000 个文件）
        if stream && scanned % 1000 == 0 {
            stream_output.write_progress(scanned, stream_output.count())?;
        }
    }

    // 如果不是流式输出，现在输出所有结果
    if !stream {
        // 模糊搜索时按相关度排序
        if use_fuzzy {
            results.sort_by(|a, b| {
                let a_name = a.path.to_lowercase();
                let b_name = b.path.to_lowercase();
                let pattern_lower = pattern.to_lowercase();

                // 包含匹配度排序
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
                    "scanned": scanned,
                    "error": null
                });
                println!("{}", output);
            }
        }

        eprintln!(
            "搜索完成：共找到 {} 个结果（扫描了 {} 个文件）",
            results.len(),
            scanned
        );
    } else {
        // 流式输出完成
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
