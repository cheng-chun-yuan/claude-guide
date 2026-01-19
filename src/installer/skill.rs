use anyhow::{anyhow, Result};
use chrono::Utc;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use super::github::{clone_repo_validated, parse_github_url};
use super::registry::{InstalledItem, Registry};
use crate::config::get_skills_dir;

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

    let skills_dir = get_skills_dir();
    let dest = skills_dir.join(&name);

    pb.set_message("Cloning and validating skill...");

    clone_repo_validated(
        url,
        &dest,
        Some(|temp_dir: &std::path::Path| {
            let skill_md = temp_dir.join("SKILL.md");
            if !skill_md.exists() {
                return Err(anyhow!("Invalid skill: missing SKILL.md"));
            }
            Ok(())
        }),
    )?;

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
