use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use std::path::PathBuf;

use crate::config::agent_types::{parse_agent_type, AgentType};
use crate::config::skills::InstallScope;
use crate::version::{
    create_profile_from_current, delete_profile, export_profile, list_profiles, load_profile,
    VersionManager,
};

#[derive(Parser)]
#[command(name = "version")]
#[command(about = "Manage skill versions and profiles")]
pub struct VersionCli {
    #[command(subcommand)]
    pub action: VersionAction,
}

#[derive(Subcommand)]
pub enum VersionAction {
    List {
        skill_name: String,
    },

    Install {
        skill_name: String,
        url: String,
        #[arg(short, long)]
        scope: Option<InstallScope>,
    },

    Switch {
        skill_name: String,
        version: String,
    },

    Create {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    Export {
        profile_name: String,
        target_dir: PathBuf,
        #[arg(short, long, default_value = "claude")]
        platform: String,
    },

    Profiles,

    Delete {
        profile_name: String,
    },

    Show {
        profile_name: String,
    },
}

pub fn execute_version_command(cmd: VersionCli) -> Result<()> {
    match cmd.action {
        VersionAction::List { skill_name } => {
            let mut manager = VersionManager::new();
            manager.load()?;

            let versions = manager.list_versions(&skill_name)?;
            if versions.is_empty() {
                println!("No versions found for skill '{}'", skill_name);
                return Ok(());
            }

            println!("{}", style("Versions:").bold());
            for version in &versions {
                println!(
                    "  [v{}] {} {}",
                    style(version.version.clone()).cyan(),
                    style(version.installed_at.format("%Y-%m-%d")).dim(),
                    version.source_type
                );
            }
            Ok(())
        }

        VersionAction::Install {
            skill_name,
            url,
            scope,
        } => {
            let mut manager = VersionManager::new();
            manager.load()?;

            let scope = scope.unwrap_or(InstallScope::Global);

            manager.install_version(&skill_name, &url, "github", scope)?;

            println!("{} Version installed successfully!", style("✓").green());
            Ok(())
        }

        VersionAction::Switch {
            skill_name,
            version,
        } => {
            let mut manager = VersionManager::new();
            manager.load()?;

            manager.switch_version(&skill_name, &version)?;

            println!("{} Switched to version {}", style("✓").green(), version);
            Ok(())
        }

        VersionAction::Create { name, description } => {
            let agent_type = AgentType::ClaudeCode;
            let profile = create_profile_from_current(&name, description, agent_type)?;

            println!(
                "{} Created profile '{}' with {} skills",
                style("✓").green(),
                style(&profile.name).cyan(),
                profile.skills.len()
            );
            println!("  Platform: {}", style(agent_type.display_name()).yellow());

            if !profile.skills.is_empty() {
                println!("  Skills:");
                for skill in &profile.skills {
                    println!("    - {}", skill.name);
                }
            }

            Ok(())
        }

        VersionAction::Export {
            profile_name,
            target_dir,
            platform,
        } => {
            let agent_type = parse_agent_type(&platform)?;
            let exported_path = export_profile(&profile_name, &target_dir, agent_type)?;

            let profile = load_profile(&profile_name)?;

            println!(
                "{} Exported profile '{}' to local folder",
                style("✓").green(),
                style(&profile_name).cyan()
            );
            println!("  Target: {}", style(exported_path.display()).yellow());
            println!("  Platform: {}", style(agent_type.display_name()).yellow());
            println!("  Skills exported: {}", profile.skills.len());

            Ok(())
        }

        VersionAction::Profiles => {
            let profiles = list_profiles()?;

            if profiles.is_empty() {
                println!("No profiles found. Create one with: claude-guide version create <name>");
                return Ok(());
            }

            println!("{}", style("Saved Profiles:").bold());
            for profile in &profiles {
                println!(
                    "\n  {} {}",
                    style(&profile.name).cyan().bold(),
                    style(format!("({} skills)", profile.skills.len())).dim()
                );
                println!(
                    "    Platform: {}",
                    style(profile.platform.display_name()).yellow()
                );
                if let Some(desc) = &profile.description {
                    println!("    Description: {}", desc);
                }
                println!(
                    "    Created: {}",
                    style(profile.created_at.format("%Y-%m-%d %H:%M")).dim()
                );
            }

            Ok(())
        }

        VersionAction::Delete { profile_name } => {
            delete_profile(&profile_name)?;

            println!(
                "{} Deleted profile '{}'",
                style("✓").green(),
                style(&profile_name).cyan()
            );

            Ok(())
        }

        VersionAction::Show { profile_name } => {
            let profile = load_profile(&profile_name)?;

            println!(
                "{} {}",
                style("Profile:").bold(),
                style(&profile.name).cyan().bold()
            );
            println!(
                "  Platform: {}",
                style(profile.platform.display_name()).yellow()
            );
            if let Some(desc) = &profile.description {
                println!("  Description: {}", desc);
            }
            println!(
                "  Created: {}",
                style(profile.created_at.format("%Y-%m-%d %H:%M")).dim()
            );
            println!(
                "  Updated: {}",
                style(profile.updated_at.format("%Y-%m-%d %H:%M")).dim()
            );

            println!("\n  {} ({}):", style("Skills").bold(), profile.skills.len());
            for skill in &profile.skills {
                let version_str = skill
                    .version
                    .as_ref()
                    .map(|v| format!(" [v{}]", v))
                    .unwrap_or_default();
                println!("    - {}{}", skill.name, style(version_str).dim());
            }

            Ok(())
        }
    }
}
