use anyhow::{Context, Result};
use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default, rename = "invocationHint")]
    pub invocation_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
    pub frontmatter: SkillFrontmatter,
    pub content: String,
}

impl Skill {
    pub fn load(skill_dir: &Path) -> Result<Self> {
        let skill_file = skill_dir.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_file)
            .with_context(|| format!("Failed to read {}", skill_file.display()))?;

        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(&content);

        let frontmatter: SkillFrontmatter = parsed
            .data
            .map(|d| d.deserialize())
            .transpose()
            .unwrap_or(None)
            .unwrap_or_default();

        let name = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            name,
            path: skill_dir.to_path_buf(),
            frontmatter,
            content: parsed.content,
        })
    }

    pub fn display_name(&self) -> &str {
        self.frontmatter.name.as_deref().unwrap_or(&self.name)
    }

    pub fn description(&self) -> &str {
        self.frontmatter
            .description
            .as_deref()
            .unwrap_or("No description")
    }
}

pub fn scan_skills(skills_dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();

    if !skills_dir.exists() {
        return Ok(skills);
    }

    for entry in WalkDir::new(skills_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                match Skill::load(path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => {
                        eprintln!("Warning: Failed to load skill at {}: {}", path.display(), e)
                    }
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn create_skill_template(name: &str) -> String {
    format!(
        r#"---
name: "{}"
description: "Description of what this skill does"
version: "1.0.0"
invocationHint: "Use when..."
---

# {}

Instructions for the skill go here.
"#,
        name, name
    )
}
