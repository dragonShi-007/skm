use anyhow::{Context, Result};
use std::path::{Path, PathBuf};


pub fn cc_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    #[cfg(target_os = "windows")]
    let base = {
        let appdata = dirs::data_dir().context("Could not determine %APPDATA%")?;
        appdata.join("Claude")
    };

    #[cfg(not(target_os = "windows"))]
    let base = home.join(".claude");

    Ok(base.join("skills"))
}

pub fn list_skills(dir: &Path) -> Result<()> {
    if !dir.exists() {
        println!("No skills installed (directory not found: {})", dir.display());
        return Ok(());
    }

    let mut entries: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    entries.sort();

    if entries.is_empty() {
        println!("No skills installed in {}", dir.display());
    } else {
        println!("Installed skills in {}:", dir.display());
        for name in entries {
            println!("  - {}", name);
        }
    }
    Ok(())
}

pub fn uninstall(name: &str, dir: &Path) -> Result<()> {
    let skill_path = dir.join(name);

    if skill_path.is_dir() {
        std::fs::remove_dir_all(&skill_path)?;
    } else if skill_path.is_file() {
        std::fs::remove_file(&skill_path)?;
    } else {
        anyhow::bail!("Skill '{}' is not installed in {}", name, dir.display());
    }

    println!("Uninstalled '{}'", name);
    Ok(())
}
