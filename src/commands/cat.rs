use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(file: &Path, lines: Option<usize>, tail: bool) -> Result<()> {
    let content = fs::read_to_string(file)?;

    let output = if let Some(num_lines) = lines {
        let lines_vec: Vec<&str> = content.lines().collect();

        if tail {
            // Display last N lines
            let start = lines_vec.len().saturating_sub(num_lines);
            lines_vec[start..].join("\n")
        } else {
            // Display first N lines
            lines_vec[..num_lines.min(lines_vec.len())].join("\n")
        }
    } else {
        content
    };

    println!("{}", output);
    Ok(())
}
