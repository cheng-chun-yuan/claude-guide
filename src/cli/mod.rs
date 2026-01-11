use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;

use crate::installer::{
    add_marketplace, install_plugin_from_marketplace, install_plugin_from_url, install_skill,
    list_marketplaces, list_plugins, list_skills, remove_marketplace, remove_plugin, remove_skill,
    search_plugins, update_marketplace,
};

#[derive(Parser)]
#[command(name = "claude-guide")]
#[command(about = "A TUI and CLI for managing Claude Code configurations")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable debug mode
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install from GitHub URL
    Install {
        #[command(subcommand)]
        target: InstallTarget,
    },
    /// Manage marketplaces
    Marketplace {
        #[command(subcommand)]
        action: MarketplaceAction,
    },
    /// Plugin commands (install from marketplace, search)
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// List installed items
    List {
        /// Item type: skills, plugins, agents, commands, marketplaces
        item_type: String,
    },
    /// Remove an installed item
    Remove {
        /// Item type: skill, plugin, agent, command
        item_type: String,
        /// Name of the item to remove
        name: String,
    },
}

#[derive(Subcommand)]
pub enum InstallTarget {
    /// Install a skill from GitHub URL
    Skill {
        /// GitHub URL of the skill repository
        url: String,
    },
    /// Install a plugin from GitHub URL
    Plugin {
        /// GitHub URL of the plugin repository
        url: String,
    },
    /// Install an agent from GitHub URL (not yet implemented)
    Agent {
        /// GitHub URL of the agent repository
        url: String,
    },
    /// Install a command from GitHub URL (not yet implemented)
    Command {
        /// GitHub URL of the command repository
        url: String,
    },
}

#[derive(Subcommand)]
pub enum MarketplaceAction {
    /// Add a new marketplace from GitHub
    Add {
        /// GitHub URL of the marketplace repository
        url: String,
    },
    /// Update marketplace(s)
    Update {
        /// Name of specific marketplace to update (updates all if not specified)
        name: Option<String>,
    },
    /// List registered marketplaces
    List,
    /// Remove a marketplace
    Remove {
        /// Name of the marketplace to remove
        name: String,
    },
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// Install plugin from marketplace
    Install {
        /// Plugin name (use name@marketplace for specific marketplace)
        name: String,
    },
    /// Search plugins in marketplaces
    Search {
        /// Search query
        query: String,
    },
}

/// Execute a CLI command
pub fn execute_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install { target } => match target {
            InstallTarget::Skill { url } => {
                install_skill(&url)?;
            }
            InstallTarget::Plugin { url } => {
                install_plugin_from_url(&url)?;
            }
            InstallTarget::Agent { url } => {
                println!(
                    "{} Agent installation from GitHub is not yet implemented.",
                    style("!").yellow()
                );
                println!("URL: {}", url);
            }
            InstallTarget::Command { url } => {
                println!(
                    "{} Command installation from GitHub is not yet implemented.",
                    style("!").yellow()
                );
                println!("URL: {}", url);
            }
        },
        Commands::Marketplace { action } => match action {
            MarketplaceAction::Add { url } => {
                add_marketplace(&url)?;
            }
            MarketplaceAction::Update { name } => {
                update_marketplace(name.as_deref())?;
            }
            MarketplaceAction::List => {
                list_marketplaces()?;
            }
            MarketplaceAction::Remove { name } => {
                remove_marketplace(&name)?;
            }
        },
        Commands::Plugin { action } => match action {
            PluginAction::Install { name } => {
                install_plugin_from_marketplace(&name)?;
            }
            PluginAction::Search { query } => {
                search_plugins(&query)?;
            }
        },
        Commands::List { item_type } => match item_type.as_str() {
            "skills" | "skill" => {
                list_skills()?;
            }
            "plugins" | "plugin" => {
                list_plugins()?;
            }
            "marketplaces" | "marketplace" => {
                list_marketplaces()?;
            }
            "agents" | "agent" => {
                println!("Agent listing not yet implemented.");
            }
            "commands" | "command" => {
                println!("Command listing not yet implemented.");
            }
            _ => {
                println!("{} Unknown item type: {}", style("!").yellow(), item_type);
                println!("Valid types: skills, plugins, agents, commands, marketplaces");
            }
        },
        Commands::Remove { item_type, name } => match item_type.as_str() {
            "skill" | "skills" => {
                remove_skill(&name)?;
            }
            "plugin" | "plugins" => {
                remove_plugin(&name)?;
            }
            "marketplace" | "marketplaces" => {
                remove_marketplace(&name)?;
            }
            "agent" | "agents" => {
                println!("Agent removal not yet implemented.");
            }
            "command" | "commands" => {
                println!("Command removal not yet implemented.");
            }
            _ => {
                println!("{} Unknown item type: {}", style("!").yellow(), item_type);
                println!("Valid types: skill, plugin, agent, command, marketplace");
            }
        },
    }

    Ok(())
}
