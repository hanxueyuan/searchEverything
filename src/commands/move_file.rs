use anyhow::Result;
use std::path::Path;

pub fn execute(source: &str, dest: &Path) -> Result<()> {
    std::fs::rename(source, dest)?;
    println!("已移动：{} -> {}", source, dest.display());
    Ok(())
}
