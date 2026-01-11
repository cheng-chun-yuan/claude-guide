use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// An installed item (skill, plugin, agent, or command)
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

/// Registry tracking all installed items
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
    /// Get the registry file path
    pub fn path() -> PathBuf {
        crate::config::get_claude_dir().join("claude-guide-registry.json")
    }

    /// Load the registry from disk
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

    /// Save the registry to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write registry to {}", path.display()))?;
        Ok(())
    }

    /// Add an item to the registry
    pub fn add(&mut self, item_type: &str, item: InstalledItem) {
        let list = match item_type {
            "skill" | "skills" => &mut self.skills,
            "plugin" | "plugins" => &mut self.plugins,
            "agent" | "agents" => &mut self.agents,
            "command" | "commands" => &mut self.commands,
            _ => return,
        };

        // Remove existing item with same name
        list.retain(|i| i.name != item.name);
        list.push(item);
    }

    /// Remove an item from the registry
    pub fn remove(&mut self, item_type: &str, name: &str) -> Option<InstalledItem> {
        let list = match item_type {
            "skill" | "skills" => &mut self.skills,
            "plugin" | "plugins" => &mut self.plugins,
            "agent" | "agents" => &mut self.agents,
            "command" | "commands" => &mut self.commands,
            _ => return None,
        };

        list.iter()
            .position(|i| i.name == name)
            .map(|pos| list.remove(pos))
    }

    /// Get a list of items by type
    pub fn list(&self, item_type: &str) -> &[InstalledItem] {
        match item_type {
            "skill" | "skills" => &self.skills,
            "plugin" | "plugins" => &self.plugins,
            "agent" | "agents" => &self.agents,
            "command" | "commands" => &self.commands,
            _ => &[],
        }
    }
}

/// Marketplace source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSource {
    pub source: String,
    pub repo: String,
}

/// Known marketplace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownMarketplace {
    pub source: MarketplaceSource,
    pub install_location: PathBuf,
    pub last_updated: DateTime<Utc>,
}

/// Registry of known marketplaces
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownMarketplaces {
    #[serde(flatten)]
    pub marketplaces: HashMap<String, KnownMarketplace>,
}

impl KnownMarketplaces {
    /// Get the known marketplaces file path
    pub fn path() -> PathBuf {
        crate::config::get_plugins_dir().join("known_marketplaces.json")
    }

    /// Load known marketplaces from disk
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

    /// Save known marketplaces to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write known marketplaces to {}", path.display()))?;
        Ok(())
    }

    /// Add a marketplace
    pub fn add(&mut self, name: String, marketplace: KnownMarketplace) {
        self.marketplaces.insert(name, marketplace);
    }

    /// Remove a marketplace
    pub fn remove(&mut self, name: &str) -> Option<KnownMarketplace> {
        self.marketplaces.remove(name)
    }

    /// Get a marketplace by name
    pub fn get(&self, name: &str) -> Option<&KnownMarketplace> {
        self.marketplaces.get(name)
    }

    /// List all marketplace names
    pub fn names(&self) -> Vec<&String> {
        self.marketplaces.keys().collect()
    }
}
