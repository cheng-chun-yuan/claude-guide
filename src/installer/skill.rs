use anyhow::{anyhow, Result};
use chrono::Utc;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use super::github::{clone_repo, parse_github_url};
use super::registry::{InstalledItem, Registry};
use crate::config::get_skills_dir;

/// Install a skill from a GitHub URL
pub fn install_skill(url: &str) -> Result<String> {
    let (owner, repo) = parse_github_url(url)?;
    let name = repo.clone();

    println!(
        "{} Installing skill from {}/{}...",
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

    // Clone to skills directory
    let skills_dir = get_skills_dir();
    let dest = skills_dir.join(&name);

    clone_repo(url, &dest)?;

    pb.set_message("Validating skill...");

    // Validate that it has a SKILL.md
    let skill_md = dest.join("SKILL.md");
    if !skill_md.exists() {
        std::fs::remove_dir_all(&dest)?;
        return Err(anyhow!("Invalid skill: missing SKILL.md"));
    }

    // Register in registry
    let mut registry = Registry::load()?;
    registry.add(
        "skill",
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
        "{} Skill '{}' installed successfully!",
        style("✓").green(),
        name
    ));

    Ok(name)
}

/// List all installed skills
pub fn list_skills() -> Result<()> {
    let registry = Registry::load()?;
    let skills = registry.list("skills");

    if skills.is_empty() {
        println!("No skills installed.");
        println!();
        println!("Install one with:");
        println!("  claude-guide install skill https://github.com/user/skill-repo");
        return Ok(());
    }

    println!("{}", style("Installed Skills:").bold());
    println!();

    for skill in skills {
        println!(
            "  {} {}",
            style("•").green(),
            style(&skill.name).cyan().bold()
        );
        println!("    Source: {}", skill.source);
        println!(
            "    Installed: {}",
            skill.installed_at.format("%Y-%m-%d %H:%M")
        );
        if let Some(path) = &skill.install_path {
            println!("    Path: {}", path.display());
        }
        println!();
    }

    Ok(())
}

/// Remove an installed skill
pub fn remove_skill(name: &str) -> Result<()> {
    let mut registry = Registry::load()?;

    let item = registry
        .remove("skill", name)
        .ok_or_else(|| anyhow!("Skill '{}' not found", name))?;

    // Remove the directory if it exists
    if let Some(path) = &item.install_path {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
    } else {
        // Try default location
        let skills_dir = get_skills_dir();
        let path = skills_dir.join(name);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }
    }

    registry.save()?;

    println!(
        "{} Skill '{}' removed.",
        style("✓").green(),
        name
    );

    Ok(())
}
