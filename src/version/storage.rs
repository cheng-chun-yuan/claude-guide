use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::settings::get_skills_dir;
use crate::version::SkillVersion;

fn get_versions_dir(skill_name: &str) -> Result<PathBuf> {
    let skills_dir = get_skills_dir();
    Ok(skills_dir.join(skill_name).join("versions"))
}

fn get_active_version_file(skill_name: &str) -> PathBuf {
    let skills_dir = get_skills_dir();
    skills_dir.join(skill_name).join(".active-version")
}

pub fn save_version(version_info: &SkillVersion) -> Result<()> {
    let versions_dir = get_versions_dir(&version_info.name)?;
    let version_dir = versions_dir.join(format!("v{}", version_info.version));

    std::fs::create_dir_all(&version_dir)
        .with_context(|| format!("Failed to create version dir {}", version_dir.display()))?;

    // Backup files
    let skills_dir = get_skills_dir();
    let source_dir = skills_dir.join(&version_info.name);

    if source_dir.exists() {
        copy_directory_recursive(&source_dir, &version_dir)?;
    }

    // Save metadata
    let version_file = versions_dir.join(format!("v{}.json", version_info.version));
    let json_data = serde_json::to_string_pretty(version_info)?;

    std::fs::write(&version_file, json_data)
        .with_context(|| format!("Failed to write {}", version_file.display()))?;

    Ok(())
}

pub fn list_versions(skill_name: &str) -> Result<Vec<SkillVersion>> {
    let versions_dir = get_versions_dir(skill_name)?;

    if !versions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();

    for entry in std::fs::read_dir(&versions_dir)
        .with_context(|| format!("Failed to read {}", versions_dir.display()))?
    {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();

        if filename.starts_with("v") && filename.ends_with(".json") {
            let content = std::fs::read_to_string(entry.path())?;

            if let Ok(version_data) = serde_json::from_str::<SkillVersion>(&content) {
                versions.push(version_data);
            }
        }
    }

    versions.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));

    Ok(versions)
}

pub fn remove_version(skill_name: &str, version: &str) -> Result<()> {
    let versions_dir = get_versions_dir(skill_name)?;
    let version_file = versions_dir.join(format!("v{}.json", version));
    let version_dir = versions_dir.join(format!("v{}", version));

    if version_file.exists() {
        std::fs::remove_file(&version_file)?;
    }

    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir)?;
    }

    Ok(())
}

pub fn activate_version(skill_name: &str, version: &str) -> Result<()> {
    let skills_dir = get_skills_dir();
    let active_file = get_active_version_file(skill_name);
    let versions_dir = get_versions_dir(skill_name)?;

    std::fs::write(&active_file, version)
        .with_context(|| format!("Failed to write {}", active_file.display()))?;

    let version_dir = versions_dir.join(format!("v{}", version));

    if version_dir.exists() {
        let target_dir = skills_dir.join(skill_name);

        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }

        copy_directory_recursive(&version_dir, &target_dir)?;
    }

    Ok(())
}

fn copy_directory_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        // Skip the 'versions' directory itself to avoid infinite recursion if it's inside
        if src_path.file_name().map_or(false, |n| n == "versions") {
            continue;
        }

        if src_path.is_dir() {
            copy_directory_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
