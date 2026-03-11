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
        println!("路径：{}", file.to_string_lossy());
        println!("大小：{} 字节", metadata.len());
        println!("类型：{}", if metadata.is_dir() { "目录" } else { "文件" });
        println!("修改时间：{:?}", metadata.modified()?);
    }
    
    Ok(())
}
