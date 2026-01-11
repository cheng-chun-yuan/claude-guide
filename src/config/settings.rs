use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{HooksConfig, McpServer, McpServersConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServer>,
    #[serde(default)]
    pub enabled_plugins: Vec<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let settings: Settings = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        // Create backup
        if path.exists() {
            let backup_path = path.with_extension("json.bak");
            std::fs::copy(path, &backup_path)
                .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize settings")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;

        Ok(())
    }

    pub fn mcp_servers_config(&self) -> McpServersConfig {
        McpServersConfig {
            servers: self.mcp_servers.clone(),
        }
    }
}

pub fn get_claude_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".claude"))
        .unwrap_or_else(|| PathBuf::from(".claude"))
}

pub fn get_settings_path() -> PathBuf {
    get_claude_dir().join("settings.json")
}

pub fn get_skills_dir() -> PathBuf {
    get_claude_dir().join("skills")
}

pub fn get_commands_dir() -> PathBuf {
    get_claude_dir().join("commands")
}

pub fn get_agents_dir() -> PathBuf {
    get_claude_dir().join("agents")
}

pub fn get_plugins_dir() -> PathBuf {
    get_claude_dir().join("plugins")
}
