use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::github::{clone_repo, parse_github_url, pull_repo};
use super::registry::{KnownMarketplace, KnownMarketplaces, MarketplaceSource};
use crate::config::get_plugins_dir;

/// Plugin information from marketplace manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<serde_json::Value>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

/// Marketplace manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub plugins: Vec<PluginInfo>,
}

/// Add a new marketplace from a GitHub URL
pub fn add_marketplace(url: &str) -> Result<String> {
    let (owner, repo) = parse_github_url(url)?;
    let name = repo.clone();

    println!(
        "{} Adding marketplace from {}/{}...",
        style("").cyan(),
        owner,
        repo
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Cloning repository...");

    // Clone to marketplaces directory
    let marketplaces_dir = get_plugins_dir().join("marketplaces");
    let dest = marketplaces_dir.join(&name);

    clone_repo(url, &dest)?;

    pb.set_message("Validating marketplace...");

    // Validate that it has a marketplace.json
    let manifest_path = dest.join(".claude-plugin").join("marketplace.json");
    if !manifest_path.exists() {
        std::fs::remove_dir_all(&dest)?;
        return Err(anyhow!(
            "Invalid marketplace: missing .claude-plugin/marketplace.json"
        ));
    }

    // Read manifest to count plugins
    let manifest = load_marketplace_manifest(&dest)?;
    let plugin_count = manifest.plugins.len();

    // Register in known_marketplaces.json
    let mut known = KnownMarketplaces::load()?;
    known.add(
        name.clone(),
        KnownMarketplace {
            source: MarketplaceSource {
                source: "github".to_string(),
                repo: format!("{}/{}", owner, repo),
            },
            install_location: dest,
            last_updated: Utc::now(),
        },
    );
    known.save()?;

    pb.finish_with_message(format!(
        "{} Marketplace '{}' added with {} plugins!",
        style("").green(),
        name,
        plugin_count
    ));

    Ok(name)
}

/// Update a marketplace (or all if name is None)
pub fn update_marketplace(name: Option<&str>) -> Result<()> {
    let mut known = KnownMarketplaces::load()?;

    if let Some(name) = name {
        // Update specific marketplace
        let marketplace = known
            .marketplaces
            .get(name)
            .ok_or_else(|| anyhow!("Marketplace '{}' not found", name))?;

        let install_location = marketplace.install_location.clone();

        println!("{} Updating marketplace '{}'...", style("").cyan(), name);

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Pulling latest changes...");

        pull_repo(&install_location)?;

        // Update the last_updated timestamp
        if let Some(mp) = known.marketplaces.get_mut(name) {
            mp.last_updated = Utc::now();
        }
        known.save()?;

        pb.finish_with_message(format!(
            "{} Marketplace '{}' updated!",
            style("").green(),
            name
        ));
    } else {
        // Update all marketplaces
        let names: Vec<String> = known.names().iter().map(|s| s.to_string()).collect();

        if names.is_empty() {
            println!("No marketplaces registered.");
            return Ok(());
        }

        println!(
            "{} Updating {} marketplace(s)...",
            style("").cyan(),
            names.len()
        );

        for name in names {
            if let Err(e) = update_marketplace(Some(&name)) {
                println!("{} Failed to update '{}': {}", style("").red(), name, e);
            }
        }
    }

    Ok(())
}

/// List all registered marketplaces
pub fn list_marketplaces() -> Result<()> {
    let known = KnownMarketplaces::load()?;

    if known.marketplaces.is_empty() {
        println!("No marketplaces registered.");
        println!();
        println!("Add one with:");
        println!("  claude-guide marketplace add https://github.com/anthropics/claude-plugins-official");
        return Ok(());
    }

    println!("{}", style("Registered Marketplaces:").bold());
    println!();

    for (name, marketplace) in &known.marketplaces {
        let manifest = load_marketplace_manifest(&marketplace.install_location).ok();
        let plugin_count = manifest.map(|m| m.plugins.len()).unwrap_or(0);

        println!(
            "  {} {} ({} plugins)",
            style("").green(),
            style(name).cyan().bold(),
            plugin_count
        );
        println!("    Source: github.com/{}", marketplace.source.repo);
        println!("    Updated: {}", marketplace.last_updated.format("%Y-%m-%d %H:%M"));
        println!();
    }

    Ok(())
}

/// Remove a marketplace
pub fn remove_marketplace(name: &str) -> Result<()> {
    let mut known = KnownMarketplaces::load()?;

    let marketplace = known
        .remove(name)
        .ok_or_else(|| anyhow!("Marketplace '{}' not found", name))?;

    // Remove the directory
    if marketplace.install_location.exists() {
        std::fs::remove_dir_all(&marketplace.install_location)?;
    }

    known.save()?;

    println!(
        "{} Marketplace '{}' removed.",
        style("").green(),
        name
    );

    Ok(())
}

/// Load marketplace manifest from a directory
pub fn load_marketplace_manifest(marketplace_dir: &Path) -> Result<MarketplaceManifest> {
    let manifest_path = marketplace_dir.join(".claude-plugin").join("marketplace.json");

    let content = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", manifest_path.display()))
}

/// Search for a plugin across all marketplaces
pub fn search_plugin(query: &str) -> Result<Vec<(String, PluginInfo)>> {
    let known = KnownMarketplaces::load()?;
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for (marketplace_name, marketplace) in &known.marketplaces {
        if let Ok(manifest) = load_marketplace_manifest(&marketplace.install_location) {
            for plugin in manifest.plugins {
                let name_match = plugin.name.to_lowercase().contains(&query_lower);
                let desc_match = plugin
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

                if name_match || desc_match {
                    results.push((marketplace_name.clone(), plugin));
                }
            }
        }
    }

    Ok(results)
}

/// Find a plugin by name, optionally from a specific marketplace
pub fn find_plugin(name: &str, marketplace: Option<&str>) -> Result<(String, PluginInfo, PathBuf)> {
    let known = KnownMarketplaces::load()?;

    // Parse name@marketplace format
    let (plugin_name, marketplace_name) = if name.contains('@') {
        let parts: Vec<&str> = name.split('@').collect();
        (parts[0], Some(parts[1]))
    } else {
        (name, marketplace)
    };

    // Build list of marketplaces to search
    let marketplaces_to_search: Vec<(String, &KnownMarketplace)> = if let Some(mp) = marketplace_name
    {
        known
            .get(mp)
            .map(|m| vec![(mp.to_string(), m)])
            .ok_or_else(|| anyhow!("Marketplace '{}' not found", mp))?
    } else {
        known
            .marketplaces
            .iter()
            .map(|(k, v)| (k.clone(), v))
            .collect()
    };

    for (mp_name, marketplace) in marketplaces_to_search {
        if let Ok(manifest) = load_marketplace_manifest(&marketplace.install_location) {
            for plugin in manifest.plugins {
                if plugin.name == plugin_name {
                    return Ok((
                        mp_name,
                        plugin,
                        marketplace.install_location.clone(),
                    ));
                }
            }
        }
    }

    Err(anyhow!("Plugin '{}' not found in any marketplace", plugin_name))
}
