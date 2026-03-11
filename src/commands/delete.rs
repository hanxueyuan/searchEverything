use anyhow::Result;
use glob::glob;

pub fn execute(path: &str, force: bool) -> Result<()> {
    let paths: Vec<_> = glob(path)?.filter_map(|p| p.ok()).collect();
    
    if paths.is_empty() {
        // 单个文件
        if force {
            std::fs::remove_file(path)?;
            println!("已删除：{}", path);
        } else {
            print!("确认删除 {}? (y/N): ", path);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                std::fs::remove_file(path)?;
                println!("已删除：{}", path);
            }
        }
    } else {
        // 多个文件
        for file_path in paths {
            if force {
                std::fs::remove_file(&file_path)?;
                println!("已删除：{}", file_path.display());
            } else {
                print!("确认删除 {}? (y/N): ", file_path.display());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() == "y" {
                    std::fs::remove_file(&file_path)?;
                    println!("已删除：{}", file_path.display());
                }
            }
        }
    }
    
    Ok(())
}
