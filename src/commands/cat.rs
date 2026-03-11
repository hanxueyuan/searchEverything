use anyhow::Result;
use std::path::Path;
use std::fs;

pub fn execute(file: &Path, lines: Option<usize>, tail: bool) -> Result<()> {
    let content = fs::read_to_string(file)?;
    
    let output = if let Some(num_lines) = lines {
        let mut lines_vec: Vec<&str> = content.lines().collect();
        
        if tail {
            // 显示末尾 N 行
            let start = lines_vec.len().saturating_sub(num_lines);
            lines_vec[start..].join("\n")
        } else {
            // 显示前 N 行
            lines_vec[..num_lines.min(lines_vec.len())].join("\n")
        }
    } else {
        content
    };
    
    println!("{}", output);
    Ok(())
}
