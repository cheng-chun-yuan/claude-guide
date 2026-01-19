use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::github::{clone_repo_validated, parse_github_url, pull_repo};
use super::registry::{KnownMarketplace, KnownMarketplaces, MarketplaceSource};
use crate::config::get_plugins_dir;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub plugins: Vec<PluginInfo>,
}

pub fn add_marketplace(url: &str) -> Result<String> {
    let (owner, repo) = parse_github_url(url)?;
    let name = repo.clone();

    println!(
        "{} Adding marketplace from {}/{}...",
        style("→").cyan(),
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

    let marketplaces_dir = get_plugins_dir().join("marketplaces");
    let dest = marketplaces_dir.join(&name);

    pb.set_message("Cloning and validating marketplace...");

    clone_repo_validated(
        url,
        &dest,
        Some(|temp_dir: &std::path::Path| {
            let manifest_path = temp_dir.join(".claude-plugin").join("marketplace.json");
            if !manifest_path.exists() {
                return Err(anyhow!(
                    "Invalid marketplace: missing .claude-plugin/marketplace.json"
                ));
            }
            Ok(())
        }),
    )?;

    let manifest = load_marketplace_manifest(&dest)?;
    let plugin_count = manifest.plugins.len();

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
        style("✓").green(),
        name,
        plugin_count
    ));

    Ok(name)
}

pub fn update_marketplace(name: Option<&str>) -> Result<()> {
    let mut known = KnownMarketplaces::load()?;

    if let Some(name) = name {
        let marketplace = known
            .marketplaces
            .get(name)
            .ok_or_else(|| anyhow!("Marketplace '{}' not found", name))?;

        let install_location = marketplace.install_location.clone();

        println!("{} Updating marketplace '{}'...", style("→").cyan(), name);

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Pulling latest changes...");

        pull_repo(&install_location)?;

        if let Some(mp) = known.marketplaces.get_mut(name) {
            mp.last_updated = Utc::now();
        }
        known.save()?;

        pb.finish_with_message(format!(
            "{} Marketplace '{}' updated!",
            style("✓").green(),
            name
        ));
    } else {
        let names: Vec<String> = known.names().iter().map(|s| s.to_string()).collect();

        if names.is_empty() {
            println!("No marketplaces registered.");
            return Ok(());
        }

        println!(
            "{} Updating {} marketplace(s)...",
            style("→").cyan(),
            names.len()
        );

        let mut failures: Vec<(String, String)> = Vec::new();

        for name in &names {
            if let Err(e) = update_marketplace(Some(name)) {
                println!("{} Failed to update '{}': {}", style("✗").red(), name, e);
                failures.push((name.clone(), e.to_string()));
            }
        }

        if !failures.is_empty() {
            let failed_names: Vec<&str> = failures.iter().map(|(n, _)| n.as_str()).collect();
            return Err(anyhow!(
                "Failed to update {} of {} marketplace(s): {}",
                failures.len(),
                names.len(),
                failed_names.join(", ")
            ));
        }
    }

    Ok(())
}

pub fn list_marketplaces() -> Result<()> {
    let known = KnownMarketplaces::load()?;

    if known.marketplaces.is_empty() {
        println!("No marketplaces registered.");
        println!();
        println!("Add one with:");
        println!(
            "  claude-guide marketplace add https://github.com/anthropics/claude-plugins-official"
        );
        return Ok(());
    }

    println!("{}", style("Registered Marketplaces:").bold());
    println!();

    for (name, marketplace) in &known.marketplaces {
        let manifest = load_marketplace_manifest(&marketplace.install_location).ok();
        let plugin_count = manifest.map(|m| m.plugins.len()).unwrap_or(0);

        println!(
            "  {} {} ({} plugins)",
            style("•").green(),
            style(name).cyan().bold(),
            plugin_count
        );
        println!("    Source: github.com/{}", marketplace.source.repo);
        println!(
            "    Updated: {}",
            marketplace.last_updated.format("%Y-%m-%d %H:%M")
        );
        println!();
    }

    Ok(())
}

pub fn remove_marketplace(name: &str, choose: bool) -> Result<()> {
    let mut known = KnownMarketplaces::load()?;

    let marketplace = known
        .remove(name)
        .ok_or_else(|| anyhow!("Marketplace '{}' not found", name))?;

    if choose {
        show_marketplace_details(&marketplace)?;
        confirm_removal(name, "marketplace")?;
    }

    if marketplace.install_location.exists() {
        std::fs::remove_dir_all(&marketplace.install_location)?;
    }

    known.save()?;

    println!("{} Marketplace '{}' removed.", style("✓").green(), name);

    Ok(())
}

fn show_marketplace_details(marketplace: &KnownMarketplace) -> Result<()> {
    println!();
    println!("{}", style("Marketplace Details:").bold().underlined());
    println!("  Source:    {}", marketplace.source.repo);
    println!("  Path:      {}", marketplace.install_location.display());
    println!(
        "  Updated:   {}",
        marketplace.last_updated.format("%Y-%m-%d %H:%M")
    );
    Ok(())
}

fn confirm_removal(name: &str, item_type: &str) -> Result<()> {
    print!(
        "{} Are you sure you want to remove {} '{}'? [y/N]: ",
        style("?").yellow(),
        item_type,
        name
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let confirmation = input.trim().to_lowercase();
    if confirmation == "y" || confirmation == "yes" {
        Ok(())
    } else {
        Err(anyhow!("Removal cancelled"))
    }
}

pub fn load_marketplace_manifest(marketplace_dir: &Path) -> Result<MarketplaceManifest> {
    let manifest_path = marketplace_dir
        .join(".claude-plugin")
        .join("marketplace.json");

    let content = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", manifest_path.display()))
}
