use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::settings::get_skills_dir;
use crate::config::skills::InstallScope;

pub mod profile;
pub mod storage;

pub use profile::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersion {
    pub name: String,
    pub version: String,
    pub source: String,
    pub source_type: String,
    pub installed_at: DateTime<Utc>,
    pub skills: Vec<InstalledSkill>,
    pub dependencies: HashMap<String, String>,
    pub agent_compatibility: HashMap<String, bool>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub name: String,
    pub path: Option<PathBuf>,
    pub version: String,
}

pub struct VersionManager {
    versions: HashMap<String, SkillVersion>,
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    pub fn install_version(
        &mut self,
        skill_name: &str,
        url: &str,
        source_type: &str,
        scope: InstallScope,
    ) -> Result<()> {
        let _project_dir = match scope {
            InstallScope::Global => crate::config::get_skills_dir(),
            InstallScope::Local => std::env::current_dir()?,
        };

        let timestamp = Utc::now();

        let version_info = SkillVersion {
            name: skill_name.to_string(),
            version: timestamp.format("%Y.%m.%d.%H%M%S").to_string(),
            source: url.to_string(),
            source_type: source_type.to_string(),
            installed_at: timestamp,
            skills: Vec::new(),
            dependencies: HashMap::new(),
            agent_compatibility: HashMap::new(),
            metadata: HashMap::new(),
        };

        self.versions
            .insert(skill_name.to_string(), version_info.clone());

        storage::save_version(&version_info)?;

        Ok(())
    }

    pub fn list_versions(&self, skill_name: &str) -> Result<Vec<SkillVersion>> {
        storage::list_versions(skill_name)
    }

    pub fn remove_version(&mut self, skill_name: &str, version: &str) -> Result<()> {
        storage::remove_version(skill_name, version)?;

        if let Some(sv) = self.versions.get(skill_name) {
            if sv.version == version {
                self.versions.remove(skill_name);
            }
        }

        Ok(())
    }

    pub fn switch_version(&self, skill_name: &str, version: &str) -> Result<()> {
        if !self.versions.contains_key(skill_name) {
            return Err(anyhow!("Skill '{}' has no versions installed", skill_name));
        }

        let version_info = self.versions.get(skill_name).unwrap();

        if version_info.version != version {
            return Err(anyhow!(
                "Version '{}' is not installed for skill '{}'",
                version,
                skill_name
            ));
        }

        storage::activate_version(skill_name, version)?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        let skills_dir = get_skills_dir();

        if !skills_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&skills_dir)
            .with_context(|| format!("Failed to read {}", skills_dir.display()))?
        {
            let entry = entry?;
            if entry.path().is_dir() {
                let skill_name = entry.file_name().to_string_lossy().to_string();

                if let Ok(versions) = storage::list_versions(&skill_name) {
                    for version in versions {
                        self.versions.insert(skill_name.clone(), version);
                    }
                }
            }
        }

        Ok(())
    }
}
