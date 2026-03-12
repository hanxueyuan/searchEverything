use anyhow::Result;
use glob::glob;

pub fn execute(path: &str, force: bool) -> Result<()> {
    let paths: Vec<_> = glob(path)?.filter_map(|p| p.ok()).collect();

    if paths.is_empty() {
        // Single file
        if force {
            std::fs::remove_file(path)?;
            println!("Deleted: {}", path);
        } else {
            print!("Confirm deletion of {}? (y/N): ", path);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                std::fs::remove_file(path)?;
                println!("Deleted: {}", path);
            }
        }
    } else {
        // Multiple files
        for file_path in paths {
            if force {
                std::fs::remove_file(&file_path)?;
                println!("Deleted: {}", file_path.display());
            } else {
                print!("Confirm deletion of {}? (y/N): ", file_path.display());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() == "y" {
                    std::fs::remove_file(&file_path)?;
                    println!("Deleted: {}", file_path.display());
                }
            }
        }
    }

    Ok(())
}
