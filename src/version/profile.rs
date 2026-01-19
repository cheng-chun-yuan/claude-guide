//! Profile management for skill version presets
//! Allows saving named profiles (e.g., "mobile", "web") and exporting to local project folders

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::agent_types::AgentType;
use crate::config::settings::get_config_dir;
use crate::config::skills::scan_skills;

/// A saved profile containing skill presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub platform: AgentType,
    pub skills: Vec<ProfileSkill>,
    pub metadata: HashMap<String, String>,
}

/// Skill entry in a profile (stores skill content, not just reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSkill {
    pub name: String,
    pub source_path: PathBuf,
    pub content: String,
    pub version: Option<String>,
}

/// Get the profiles directory
pub fn get_profiles_dir() -> PathBuf {
    get_config_dir().join("profiles")
}

/// Get path to a specific profile
pub fn get_profile_path(name: &str) -> PathBuf {
    get_profiles_dir().join(format!("{}.json", name))
}

/// Save a profile to disk
pub fn save_profile(profile: &SkillProfile) -> Result<()> {
    let profiles_dir = get_profiles_dir();
    std::fs::create_dir_all(&profiles_dir).with_context(|| {
        format!(
            "Failed to create profiles directory: {}",
            profiles_dir.display()
        )
    })?;

    let profile_path = get_profile_path(&profile.name);
    let json = serde_json::to_string_pretty(profile).context("Failed to serialize profile")?;

    std::fs::write(&profile_path, json)
        .with_context(|| format!("Failed to write profile: {}", profile_path.display()))?;

    Ok(())
}

/// Load a profile from disk
pub fn load_profile(name: &str) -> Result<SkillProfile> {
    let profile_path = get_profile_path(name);

    if !profile_path.exists() {
        return Err(anyhow!("Profile '{}' not found", name));
    }

    let content = std::fs::read_to_string(&profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;

    let profile: SkillProfile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse profile: {}", profile_path.display()))?;

    Ok(profile)
}

/// List all available profiles
pub fn list_profiles() -> Result<Vec<SkillProfile>> {
    let profiles_dir = get_profiles_dir();

    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();

    for entry in std::fs::read_dir(&profiles_dir).with_context(|| {
        format!(
            "Failed to read profiles directory: {}",
            profiles_dir.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(profile) = serde_json::from_str::<SkillProfile>(&content) {
                    profiles.push(profile);
                }
            }
        }
    }

    profiles.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(profiles)
}

/// Delete a profile
pub fn delete_profile(name: &str) -> Result<()> {
    let profile_path = get_profile_path(name);

    if !profile_path.exists() {
        return Err(anyhow!("Profile '{}' not found", name));
    }

    std::fs::remove_file(&profile_path)
        .with_context(|| format!("Failed to delete profile: {}", profile_path.display()))?;

    Ok(())
}

/// Create a profile from current global skills
pub fn create_profile_from_current(
    name: &str,
    description: Option<String>,
    platform: AgentType,
) -> Result<SkillProfile> {
    let skills_dir = platform.global_dir();
    let skills = scan_skills(&skills_dir)?;

    let profile_skills: Vec<ProfileSkill> = skills
        .iter()
        .map(|skill| {
            let skill_file = skill.path.join("SKILL.md");
            let content = std::fs::read_to_string(&skill_file).unwrap_or_default();

            ProfileSkill {
                name: skill.name.clone(),
                source_path: skill.path.clone(),
                content,
                version: skill.frontmatter.version.clone(),
            }
        })
        .collect();

    let now = Utc::now();

    let profile = SkillProfile {
        name: name.to_string(),
        description,
        created_at: now,
        updated_at: now,
        platform,
        skills: profile_skills,
        metadata: HashMap::new(),
    };

    save_profile(&profile)?;

    Ok(profile)
}

/// Export a profile to a target directory with platform-specific structure
pub fn export_profile(
    profile_name: &str,
    target_dir: &Path,
    target_platform: AgentType,
) -> Result<PathBuf> {
    let profile = load_profile(profile_name)?;

    // Determine the skills directory based on target platform
    let skills_subdir = target_platform.project_dir();
    let target_skills_dir = target_dir.join(skills_subdir);

    std::fs::create_dir_all(&target_skills_dir).with_context(|| {
        format!(
            "Failed to create target directory: {}",
            target_skills_dir.display()
        )
    })?;

    let mut exported_count = 0;

    for skill in &profile.skills {
        let skill_dir = target_skills_dir.join(&skill.name);
        std::fs::create_dir_all(&skill_dir).with_context(|| {
            format!("Failed to create skill directory: {}", skill_dir.display())
        })?;

        let skill_file = skill_dir.join("SKILL.md");
        std::fs::write(&skill_file, &skill.content)
            .with_context(|| format!("Failed to write skill file: {}", skill_file.display()))?;

        // Also copy any additional files from source if they exist
        if skill.source_path.exists() {
            copy_skill_files(&skill.source_path, &skill_dir)?;
        }

        exported_count += 1;
    }

    // Create a manifest file to track the export
    let manifest = serde_json::json!({
        "profile_name": profile_name,
        "source_platform": profile.platform.short_name(),
        "target_platform": target_platform.short_name(),
        "exported_at": Utc::now().to_rfc3339(),
        "skills_count": exported_count,
    });

    let manifest_path = target_skills_dir.join(".profile-manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(target_skills_dir)
}

/// Copy additional skill files (excluding SKILL.md which is handled separately)
fn copy_skill_files(source_dir: &Path, target_dir: &Path) -> Result<()> {
    if !source_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default();

        // Skip SKILL.md as it's already written
        if file_name == "SKILL.md" {
            continue;
        }

        // Skip version directories and hidden files (except important ones)
        let name_str = file_name.to_string_lossy();
        if name_str == "versions" || (name_str.starts_with('.') && name_str != ".active-version") {
            continue;
        }

        let target_path = target_dir.join(file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &target_path)?;
        } else {
            std::fs::copy(&path, &target_path)?;
        }
    }

    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_path() {
        let path = get_profile_path("mobile");
        assert!(path.ends_with("profiles/mobile.json"));
    }
}
