use anyhow::{anyhow, Result};
use chrono::Utc;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use super::github::{clone_repo, parse_github_url};
use super::marketplace::{find_plugin, search_plugin};
use super::registry::{InstalledItem, Registry};
use crate::config::get_plugins_dir;

/// Install a plugin from a GitHub URL
pub fn install_plugin_from_url(url: &str) -> Result<String> {
    let (owner, repo) = parse_github_url(url)?;
    let name = repo.clone();

    println!(
        "{} Installing plugin from {}/{}...",
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

    // Clone to plugins cache directory
    let plugins_dir = get_plugins_dir();
    let cache_dir = plugins_dir.join("cache");
    std::fs::create_dir_all(&cache_dir)?;
    let dest = cache_dir.join(&name);

    clone_repo(url, &dest)?;

    pb.set_message("Validating plugin...");

    // Validate that it has a .claude-plugin/plugin.json
    let plugin_json = dest.join(".claude-plugin").join("plugin.json");
    if !plugin_json.exists() {
        std::fs::remove_dir_all(&dest)?;
        return Err(anyhow!(
            "Invalid plugin: missing .claude-plugin/plugin.json"
        ));
    }

    // Register in registry
    let mut registry = Registry::load()?;
    registry.add(
        "plugin",
        InstalledItem {
            name: name.clone(),
            source: url.to_string(),
            installed_at: Utc::now(),
            version: None,
            install_path: Some(dest.clone()),
        },
    );
    registry.save()?;

    pb.finish_with_message(format!(
        "{} Plugin '{}' installed successfully!",
        style("✓").green(),
        name
    ));

    Ok(name)
}

/// Install a plugin from a marketplace
pub fn install_plugin_from_marketplace(name: &str) -> Result<String> {
    println!(
        "{} Searching for plugin '{}'...",
        style("→").cyan(),
        name
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Searching marketplaces...");

    // Find the plugin in marketplaces
    let (marketplace_name, plugin_info, _marketplace_path) = find_plugin(name, None)?;

    pb.set_message(format!(
        "Found '{}' in marketplace '{}'",
        plugin_info.name, marketplace_name
    ));

    // Get the source URL
    let source_url = plugin_info
        .source
        .as_ref()
        .ok_or_else(|| anyhow!("Plugin '{}' has no source URL", name))?;

    pb.set_message("Cloning plugin...");

    // Clone to plugins cache directory
    let plugins_dir = get_plugins_dir();
    let cache_dir = plugins_dir.join("cache");
    std::fs::create_dir_all(&cache_dir)?;
    let dest = cache_dir.join(&plugin_info.name);

    clone_repo(source_url, &dest)?;

    pb.set_message("Validating plugin...");

    // Validate that it has a .claude-plugin/plugin.json
    let plugin_json = dest.join(".claude-plugin").join("plugin.json");
    if !plugin_json.exists() {
        std::fs::remove_dir_all(&dest)?;
        return Err(anyhow!(
            "Invalid plugin: missing .claude-plugin/plugin.json"
        ));
    }

    // Register in registry
    let mut registry = Registry::load()?;
    registry.add(
        "plugin",
        InstalledItem {
            name: plugin_info.name.clone(),
            source: format!("{}@{}", plugin_info.name, marketplace_name),
            installed_at: Utc::now(),
            version: plugin_info.version.clone(),
            install_path: Some(dest.clone()),
        },
    );
    registry.save()?;

    pb.finish_with_message(format!(
        "{} Plugin '{}' installed from marketplace '{}'!",
        style("✓").green(),
        plugin_info.name,
        marketplace_name
    ));

    Ok(plugin_info.name)
}

/// Search for plugins across all marketplaces
pub fn search_plugins(query: &str) -> Result<()> {
    println!(
        "{} Searching for '{}'...",
        style("→").cyan(),
        query
    );

    let results = search_plugin(query)?;

    if results.is_empty() {
        println!("No plugins found matching '{}'.", query);
        return Ok(());
    }

    println!();
    println!(
        "{} Found {} plugin(s):",
        style("✓").green(),
        results.len()
    );
    println!();

    for (marketplace, plugin) in results {
        println!(
            "  {} {} (from {})",
            style("•").green(),
            style(&plugin.name).cyan().bold(),
            marketplace
        );
        if let Some(desc) = &plugin.description {
            println!("    {}", desc);
        }
        if let Some(version) = &plugin.version {
            println!("    Version: {}", version);
        }
        if let Some(author) = &plugin.author {
            if let Some(name) = author.as_str() {
                println!("    Author: {}", name);
            } else if let Some(obj) = author.as_object() {
                if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                    println!("    Author: {}", name);
                }
            }
        }
        println!();
    }

    println!("Install with: claude-guide plugin install <name>");

    Ok(())
}

/// List all installed plugins
pub fn list_plugins() -> Result<()> {
    let registry = Registry::load()?;
    let plugins = registry.list("plugins");

    if plugins.is_empty() {
        println!("No plugins installed.");
        println!();
        println!("Install one with:");
        println!("  claude-guide plugin install <name>          # From marketplace");
        println!("  claude-guide install plugin <github-url>    # From GitHub");
        return Ok(());
    }

    println!("{}", style("Installed Plugins:").bold());
    println!();

    for plugin in plugins {
        println!(
            "  {} {}",
            style("•").green(),
            style(&plugin.name).cyan().bold()
        );
        println!("    Source: {}", plugin.source);
        if let Some(version) = &plugin.version {
            println!("    Version: {}", version);
        }
        println!(
            "    Installed: {}",
            plugin.installed_at.format("%Y-%m-%d %H:%M")
        );
        if let Some(path) = &plugin.install_path {
            println!("    Path: {}", path.display());
        }
        println!();
    }

    Ok(())
}

/// Remove an installed plugin
pub fn remove_plugin(name: &str) -> Result<()> {
    let mut registry = Registry::load()?;

    let item = registry
        .remove("plugin", name)
        .ok_or_else(|| anyhow!("Plugin '{}' not found", name))?;

    // Remove the directory if it exists
    if let Some(path) = &item.install_path {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
    } else {
        // Try default location
        let plugins_dir = get_plugins_dir();
        let path = plugins_dir.join("cache").join(name);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }
    }

    registry.save()?;

    println!(
        "{} Plugin '{}' removed.",
        style("✓").green(),
        name
    );

    Ok(())
}
