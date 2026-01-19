use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledItem {
    pub name: String,
    pub source: String,
    pub installed_at: DateTime<Utc>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub install_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub skills: Vec<InstalledItem>,
    #[serde(default)]
    pub plugins: Vec<InstalledItem>,
    #[serde(default)]
    pub agents: Vec<InstalledItem>,
    #[serde(default)]
    pub commands: Vec<InstalledItem>,
}

impl Registry {
    pub fn path() -> PathBuf {
        crate::config::get_claude_dir().join("claude-guide-registry.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read registry at {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse registry at {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write registry to {}", path.display()))?;
        Ok(())
    }

    pub fn add(&mut self, item_type: &str, item: InstalledItem) {
        let list = match item_type {
            "skill" | "skills" => &mut self.skills,
            "plugin" | "plugins" => &mut self.plugins,
            "agent" | "agents" => &mut self.agents,
            "command" | "commands" => &mut self.commands,
            _ => return,
        };

        list.retain(|i| i.name != item.name);
        list.push(item);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSource {
    pub source: String,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownMarketplace {
    pub source: MarketplaceSource,
    pub install_location: PathBuf,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownMarketplaces {
    #[serde(flatten)]
    pub marketplaces: HashMap<String, KnownMarketplace>,
}

impl KnownMarketplaces {
    pub fn path() -> PathBuf {
        crate::config::get_plugins_dir().join("known_marketplaces.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read known marketplaces at {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse known marketplaces at {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write known marketplaces to {}", path.display()))?;
        Ok(())
    }

    pub fn add(&mut self, name: String, marketplace: KnownMarketplace) {
        self.marketplaces.insert(name, marketplace);
    }

    pub fn remove(&mut self, name: &str) -> Option<KnownMarketplace> {
        self.marketplaces.remove(name)
    }

    pub fn names(&self) -> Vec<&String> {
        self.marketplaces.keys().collect()
    }
}
