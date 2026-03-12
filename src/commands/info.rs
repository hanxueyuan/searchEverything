use anyhow::Result;
use std::path::Path;

pub fn execute(file: &Path, json: bool) -> Result<()> {
    let metadata = std::fs::metadata(file)?;

    let info = serde_json::json!({
        "path": file.to_string_lossy(),
        "size": metadata.len(),
        "is_dir": metadata.is_dir(),
        "is_file": metadata.is_file(),
        "modified": format!("{:?}", metadata.modified()?),
        "created": format!("{:?}", metadata.created()?),
        "accessed": format!("{:?}", metadata.accessed()?),
    });

    if json {
        println!("{}", info);
    } else {
        println!("Path: {}", file.to_string_lossy());
        println!("Size: {} bytes", metadata.len());
        println!(
            "Type: {}",
            if metadata.is_dir() {
                "Directory"
            } else {
                "File"
            }
        );
        println!("Modified: {:?}", metadata.modified()?);
    }

    Ok(())
}
