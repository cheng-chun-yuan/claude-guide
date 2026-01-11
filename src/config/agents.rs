use anyhow::{Context, Result};
use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default, rename = "allowedTools")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(default, rename = "disallowedTools")]
    pub disallowed_tools: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub path: PathBuf,
    pub frontmatter: AgentFrontmatter,
    pub content: String,
}

impl Agent {
    pub fn load(path: &Path) -> Result<Self> {
        let file_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(&file_content);

        let frontmatter: AgentFrontmatter = parsed
            .data
            .map(|d| d.deserialize())
            .transpose()
            .unwrap_or(None)
            .unwrap_or_default();

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            name,
            path: path.to_path_buf(),
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

    pub fn model(&self) -> &str {
        self.frontmatter.model.as_deref().unwrap_or("default")
    }
}

pub fn scan_agents(agents_dir: &Path) -> Result<Vec<Agent>> {
    let mut agents = Vec::new();

    if !agents_dir.exists() {
        return Ok(agents);
    }

    for entry in WalkDir::new(agents_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            match Agent::load(path) {
                Ok(agent) => agents.push(agent),
                Err(e) => eprintln!("Warning: Failed to load agent at {}: {}", path.display(), e),
            }
        }
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(agents)
}

pub fn create_agent_template(name: &str) -> String {
    format!(
        r#"---
name: "{}"
description: "Description of what this agent does"
model: "sonnet"
allowedTools:
  - Read
  - Write
  - Edit
  - Bash
---

# {}

Instructions for the agent go here.
"#,
        name, name
    )
}
