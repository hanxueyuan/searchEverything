use anyhow::Result;
use std::path::Path;
use glob::glob;

pub fn execute(source: &str, dest: &Path) -> Result<()> {
    // 处理通配符
    let paths: Vec<_> = glob(source)?.filter_map(|p| p.ok()).collect();
    
    if paths.is_empty() {
        // 单个文件
        std::fs::copy(source, dest)?;
        println!("已复制：{} -> {}", source, dest.display());
    } else {
        // 多个文件
        for src_path in paths {
            let file_name = src_path.file_name().unwrap_or_default();
            let dest_path = dest.join(file_name);
            std::fs::copy(&src_path, &dest_path)?;
            println!("已复制：{} -> {}", src_path.display(), dest_path.display());
        }
    }
    
    Ok(())
}
