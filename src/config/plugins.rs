use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub id: String,
    pub path: PathBuf,
    pub manifest: Option<PluginManifest>,
    pub enabled: bool,
}

impl Plugin {
    pub fn display_name(&self) -> &str {
        self.manifest
            .as_ref()
            .and_then(|m| m.name.as_deref())
            .unwrap_or(&self.id)
    }

    pub fn description(&self) -> &str {
        self.manifest
            .as_ref()
            .and_then(|m| m.description.as_deref())
            .unwrap_or("No description")
    }
}

pub fn scan_plugins(plugins_dir: &Path, enabled_plugins: &HashMap<String, bool>) -> Result<Vec<Plugin>> {
    let mut plugins = Vec::new();
    let marketplaces_dir = plugins_dir.join("marketplaces");

    if marketplaces_dir.exists() {
        for entry in WalkDir::new(&marketplaces_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                let plugin_dir = path.join(".claude-plugin");
                let manifest_path = plugin_dir.join("marketplace.json");

                let manifest = if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path)
                        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
                    serde_json::from_str(&content).ok()
                } else {
                    None
                };

                let id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Check if plugin is enabled - look for exact match or pattern match
                let enabled = enabled_plugins.iter().any(|(key, &val)| {
                    val && (key == &id || key.contains(&id) || id.contains(key))
                });

                plugins.push(Plugin {
                    id,
                    path: path.to_path_buf(),
                    manifest,
                    enabled,
                });
            }
        }
    }

    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}
