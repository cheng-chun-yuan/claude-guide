pub mod version;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-guide")]
#[command(about = "A TUI and CLI for managing Claude Code configurations", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a skill, plugin, or other resource
    Install(InstallCommand),
    /// List installed items
    List(ListCommand),
    /// Remove an installed item
    Remove(RemoveCommand),
    /// Manage marketplaces
    Marketplace(MarketplaceCommand),
    /// Manage versions
    Version(version::VersionCli),
    /// Initialize a new project or snapshot
    Init(InitCommand),
    /// Snapshot current configuration
    Snapshot(SnapshotCommand),
}

#[derive(Parser)]
pub struct InstallCommand {
    /// Type of item to install
    #[arg(value_enum)]
    pub item_type: InstallType,
    /// URL of the repository or item
    pub url: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum InstallType {
    Skill,
    Plugin,
}

#[derive(Parser)]
pub struct ListCommand {
    /// Type of items to list
    #[arg(value_enum)]
    pub item_type: ListType,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ListType {
    Skills,
    Plugins,
    Commands,
}

#[derive(Parser)]
pub struct RemoveCommand {
    /// Type of item to remove
    #[arg(value_enum)]
    pub item_type: InstallType,
    /// Name of the item
    pub name: String,
}

#[derive(Parser)]
pub struct MarketplaceCommand {
    #[command(subcommand)]
    pub action: MarketplaceAction,
}

#[derive(Parser)]
pub struct InitCommand {
    /// Name for the project/snapshot
    pub name: String,
    /// Target platform (claude, gemini, opencode, etc.)
    #[arg(short, long)]
    pub platform: Option<String>,
    /// Install globally instead of in project
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Parser)]
pub struct SnapshotCommand {
    /// Name for the snapshot
    pub name: String,
    /// Target platform (claude, gemini, opencode, etc.)
    #[arg(short, long)]
    pub platform: Option<String>,
}

#[derive(Subcommand)]
pub enum MarketplaceAction {
    /// Add a marketplace
    Add { url: String },
    /// List marketplaces
    List,
    /// Update a marketplace
    Update { name: Option<String> },
    /// Remove a marketplace
    Remove { name: String },
}

/// Execute CLI commands
pub fn execute_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install(install) => {
            crate::installer::install_skill(&install.url)?;
            Ok(())
        }
        Commands::List(list) => {
            match list.item_type {
                ListType::Skills => {
                    let skills = crate::config::scan_skills(&crate::config::get_skills_dir())?;
                    println!("Installed Skills:");
                    for skill in skills {
                        println!("  - {}", skill.display_name());
                    }
                }
                ListType::Plugins => {
                    let plugins = crate::config::scan_plugins(
                        &crate::config::get_plugins_dir(),
                        &std::collections::HashMap::new(),
                    )?;
                    println!("Installed Plugins:");
                    for plugin in plugins {
                        println!("  - {}", plugin.display_name());
                    }
                }
                ListType::Commands => {
                    let commands =
                        crate::config::scan_commands(&crate::config::get_commands_dir())?;
                    println!("Available Commands:");
                    for cmd in commands {
                        println!("  - {}", cmd.name);
                    }
                }
            }
            Ok(())
        }
        Commands::Remove(remove) => {
            match remove.item_type {
                InstallType::Skill => {
                    let skill_dir = crate::config::get_skills_dir().join(&remove.name);
                    std::fs::remove_dir_all(&skill_dir)?;
                    println!("Removed skill '{}'", remove.name);
                }
                InstallType::Plugin => {
                    println!("Removing plugins is not yet implemented");
                }
            }
            Ok(())
        }
        Commands::Marketplace(marketplace) => {
            match marketplace.action {
                MarketplaceAction::Add { url } => {
                    crate::installer::add_marketplace(&url)?;
                    println!("Added marketplace: {}", url);
                }
                MarketplaceAction::List => {
                    crate::installer::list_marketplaces()?;
                }
                MarketplaceAction::Update { name } => {
                    if let Some(name) = name {
                        crate::installer::update_marketplace(Some(&name))?;
                        println!("Updated marketplace: {}", name);
                    } else {
                        crate::installer::list_marketplaces()?;
                        println!("Update all marketplaces manually");
                    }
                }
                MarketplaceAction::Remove { name } => {
                    crate::installer::remove_marketplace(&name, false)?;
                    println!("Removed marketplace: {}", name);
                }
            }
            Ok(())
        }
        Commands::Version(version_cmd) => version::execute_version_command(version_cmd),
        Commands::Init(init) => execute_init_command(init),
        Commands::Snapshot(snapshot) => execute_snapshot_command(snapshot),
    }
}

fn execute_init_command(cmd: InitCommand) -> Result<()> {
    use crate::config::skills::InstallScope;
    use crate::config::{get_config_dir, AgentType};

    let scope = if cmd.global {
        InstallScope::Global
    } else {
        InstallScope::Local
    };

    let platform = match cmd.platform.as_deref().unwrap_or("claude").to_lowercase().as_str() {
        "claude" | "claude-code" => AgentType::ClaudeCode,
        "cursor" => AgentType::Cursor,
        "codex" => AgentType::Codex,
        "opencode" | "open" => AgentType::OpenCode,
        "amp" => AgentType::Amp,
        "roo" | "roocode" | "roo-code" => AgentType::Roo,
        "goose" => AgentType::Goose,
        "kilo" | "kilocode" | "kilo-code" => AgentType::Kilo,
        "antigravity" => AgentType::Antigravity,
        "github" | "github-copilot" | "copilot" => AgentType::GitHubCopilot,
        _ => anyhow::bail!("Unsupported platform '{}'. Supported platforms: claude, cursor, codex, opencode, amp, roo, goose, kilo, antigravity, github-copilot", cmd.platform.as_deref().unwrap_or("claude")),
    };

    let current_config = if scope == InstallScope::Global {
        get_config_dir()
    } else {
        std::path::PathBuf::from(".claude-guide")
    };

    let snapshot_path = current_config
        .join("snapshots")
        .join(format!("{}.json", &cmd.name));

    if snapshot_path.exists() {
        anyhow::bail!(
            "Snapshot '{}' already exists. Use snapshot command to update.",
            cmd.name
        );
    }

    std::fs::create_dir_all(snapshot_path.parent().unwrap())?;

    let snapshot_data = serde_json::json!({
        "name": &cmd.name,
        "platform": platform.display_name(),
        "scope": if scope == InstallScope::Global { "global" } else { "project" },
        "created_at": chrono::Utc::now().to_rfc3339(),
        "agent_type": serde_json::to_value(platform)?,
    });

    std::fs::write(
        &snapshot_path,
        serde_json::to_string_pretty(&snapshot_data)?,
    )?;

    println!(
        "Initialized project/snapshot '{}' for platform '{}'",
        cmd.name,
        platform.display_name()
    );
    println!("Configuration location: {}", current_config.display());

    Ok(())
}

fn execute_snapshot_command(cmd: SnapshotCommand) -> Result<()> {
    use crate::config::{get_config_dir, AgentType};
    use std::collections::HashMap;

    let config_dir = get_config_dir();
    let skills_dir = crate::config::get_skills_dir();
    let plugins_dir = crate::config::get_plugins_dir();

    let platform = match cmd.platform.as_deref().unwrap_or("claude").to_lowercase().as_str() {
        "claude" | "claude-code" => AgentType::ClaudeCode,
        "cursor" => AgentType::Cursor,
        "codex" => AgentType::Codex,
        "opencode" | "open" => AgentType::OpenCode,
        "amp" => AgentType::Amp,
        "roo" | "roocode" | "roo-code" => AgentType::Roo,
        "goose" => AgentType::Goose,
        "kilo" | "kilocode" | "kilo-code" => AgentType::Kilo,
        "antigravity" => AgentType::Antigravity,
        "github" | "github-copilot" | "copilot" => AgentType::GitHubCopilot,
        _ => anyhow::bail!("Unsupported platform '{}'. Supported platforms: claude, cursor, codex, opencode, amp, roo, goose, kilo, antigravity, github-copilot", cmd.platform.as_deref().unwrap_or("claude")),
    };

    let snapshot_path = config_dir
        .join("snapshots")
        .join(format!("{}.json", &cmd.name));

    if snapshot_path.exists() {
        anyhow::bail!(
            "Snapshot '{}' already exists. Use a different name.",
            cmd.name
        );
    }

    std::fs::create_dir_all(snapshot_path.parent().unwrap())?;

    let skills = crate::config::scan_skills(&skills_dir)?;
    let plugins = crate::config::scan_plugins(&plugins_dir, &HashMap::new())?;

    let snapshot_data = serde_json::json!({
        "name": &cmd.name,
        "platform": platform.display_name(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "skills": skills.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
        "plugins": plugins.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
        "agent_type": serde_json::to_value(platform)?,
    });

    std::fs::write(
        &snapshot_path,
        serde_json::to_string_pretty(&snapshot_data)?,
    )?;

    println!(
        "Created snapshot '{}' for platform '{}' with {} skills and {} plugins",
        cmd.name,
        platform.display_name(),
        skills.len(),
        plugins.len()
    );
    println!("Snapshot location: {}", snapshot_path.display());

    Ok(())
}
