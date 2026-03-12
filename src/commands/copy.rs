use anyhow::Result;
use glob::glob;
use std::path::Path;

pub fn execute(source: &str, dest: &Path) -> Result<()> {
    // Handle wildcards
    let paths: Vec<_> = glob(source)?.filter_map(|p| p.ok()).collect();

    if paths.is_empty() {
        // Single file
        std::fs::copy(source, dest)?;
        println!("Copied: {} -> {}", source, dest.display());
    } else {
        // Multiple files
        for src_path in paths {
            let file_name = src_path.file_name().unwrap_or_default();
            let dest_path = dest.join(file_name);
            std::fs::copy(&src_path, &dest_path)?;
            println!("Copied: {} -> {}", src_path.display(), dest_path.display());
        }
    }

    Ok(())
}
