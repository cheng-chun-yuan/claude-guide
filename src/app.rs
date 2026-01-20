use anyhow::Result;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};
use crate::keybindings::map_key_to_action;
use crate::actions::Action;
use crate::config::InstallScope;
use crate::config::{
    create_agent_template, create_command_template, create_skill_template, scan_agents,
    scan_commands, scan_plugins, scan_skills, validate_command, validate_server_name, Agent,
    AgentType, Command, Hook, HookAction, HookEvent, HookGroup, McpServer, Plugin, PresetMcpServer,
    Settings, Skill, ALLOWED_MCP_COMMANDS, PRESET_MCP_SERVERS,
};
use crate::services::ResourceManager;
use crate::version::{
    create_profile_from_current, delete_profile, export_profile, list_profiles, SkillProfile,
    VersionManager,
};

pub mod types;
pub use types::*;

pub struct App {
    pub running: bool,
    pub tab: Tab,
    pub input_mode: InputMode,
    pub focus: Focus,
    pub list_index: usize,
    pub modal: Option<ModalType>,
    pub modal_index: usize,
    pub message: Option<String>,
    pub unsaved_changes: bool,

    pub version_manager: VersionManager,

    pub settings: Settings,
    pub settings_path: PathBuf,
    pub current_platform: AgentType,
    pub current_scope: InstallScope,

    pub skills: Vec<Skill>,
    pub plugins: Vec<Plugin>,
    pub commands: Vec<Command>,
    pub agents: Vec<Agent>,
    pub profiles: Vec<SkillProfile>,
}

impl App {
    pub fn new() -> Result<Self> {
        let settings_path = crate::config::get_settings_path();
        let settings = Settings::load(&settings_path).unwrap_or_default();

        let mut app = Self {
            running: true,
            tab: Tab::Home,
            input_mode: InputMode::Normal,
            focus: Focus::List,
            list_index: 0,
            modal: None,
            modal_index: 0,
            message: None,
            unsaved_changes: false,
            version_manager: VersionManager::new(),

            settings,
            settings_path,
            current_platform: AgentType::ClaudeCode,
            current_scope: InstallScope::Global,
            skills: Vec::new(),
            plugins: Vec::new(),
            commands: Vec::new(),
            agents: Vec::new(),
            profiles: Vec::new(),
        };

        app.version_manager.load()?;
        app.reload_all()?;
        Ok(app)
    }

    pub fn reload_all(&mut self) -> Result<()> {
        self.settings_path = self.current_platform.settings_path();
        self.settings = Settings::load(&self.settings_path).unwrap_or_default();

        let (skills_dir, plugins_dir, commands_dir, agents_dir) = match self.current_scope {
            InstallScope::Global => (
                self.current_platform.skills_dir(),
                self.current_platform.plugins_dir(),
                self.current_platform.commands_dir(),
                self.current_platform.agents_dir(),
            ),
            InstallScope::Local => {
                let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let skills_rel = PathBuf::from(self.current_platform.project_dir());
                let parent = skills_rel.parent().unwrap_or_else(|| Path::new("."));

                (
                    base.join(&skills_rel),
                    base.join(parent).join("plugins"),
                    base.join(parent).join("commands"),
                    base.join(parent).join("agents"),
                )
            }
        };

        self.skills = scan_skills(&skills_dir).unwrap_or_default();
        self.plugins =
            scan_plugins(&plugins_dir, &self.settings.enabled_plugins).unwrap_or_default();
        self.commands = scan_commands(&commands_dir).unwrap_or_default();
        self.agents = scan_agents(&agents_dir).unwrap_or_default();
        self.profiles = list_profiles().unwrap_or_default();
        self.unsaved_changes = false;
        Ok(())
    }

    pub fn save_settings(&mut self) -> Result<()> {
        self.settings.save(&self.settings_path)?;
        self.unsaved_changes = false;
        self.message = Some("Settings saved".to_string());
        Ok(())
    }

    pub fn current_list(&self) -> Vec<ListItem> {
        match self.tab {
            Tab::Home => vec![],
            Tab::Hooks => self
                .settings
                .hooks
                .all_hooks()
                .into_iter()
                .enumerate()
                .map(|(idx, (event, hook))| ListItem::Hook {
                    event,
                    index: idx,
                    hook: hook.clone(),
                })
                .collect(),
            Tab::Skills => self.skills.iter().cloned().map(ListItem::Skill).collect(),
            Tab::Plugins => self.plugins.iter().cloned().map(ListItem::Plugin).collect(),
            Tab::Commands => self
                .commands
                .iter()
                .cloned()
                .map(ListItem::Command)
                .collect(),
            Tab::Agents => self.agents.iter().cloned().map(ListItem::Agent).collect(),
            Tab::Mcp => self
                .settings
                .mcp_servers
                .iter()
                .map(|(name, server)| ListItem::Mcp {
                    name: name.clone(),
                    server: server.clone(),
                })
                .collect(),
            Tab::Profiles => self
                .profiles
                .iter()
                .cloned()
                .map(ListItem::Profile)
                .collect(),
        }
    }

    pub fn selected_item(&self) -> Option<ListItem> {
        let list = self.current_list();
        list.get(self.list_index).cloned()
    }


    pub fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        // Clear any previous message
        self.message = None;

        // Handle special cases for insert mode
        if self.input_mode == InputMode::Insert {
            match key.code {
                KeyCode::Tab => {
                    self.cycle_modal_field(true);
                    return Ok(());
                }
                KeyCode::BackTab => {
                    self.cycle_modal_field(false);
                    return Ok(());
                }
                KeyCode::Char(' ') => {
                    // In add hook modal, space on type field toggles type
                    if let Some(ModalType::AddHook { .. }) = &self.modal {
                        if self.modal_index == 1 {
                            self.cycle_hook_type();
                            return Ok(());
                        }
                    }
                    // Otherwise, treat as character input
                    if self.modal_index == 2
                        || !matches!(&self.modal, Some(ModalType::AddHook { .. }))
                    {
                        self.update_modal_field(' ');
                        return Ok(());
                    }
                }
                KeyCode::Char(c) => {
                    // Only allow character input on the target field for hooks
                    if let Some(ModalType::AddHook { .. }) = &self.modal {
                        if self.modal_index == 2 {
                            self.update_modal_field(c);
                        }
                    } else {
                        self.update_modal_field(c);
                    }
                    return Ok(());
                }
                KeyCode::Backspace => {
                    self.backspace_modal_field();
                    return Ok(());
                }
                KeyCode::Up => {
                    // In add hook modal, up/down on event field cycles events
                    if let Some(ModalType::AddHook { .. }) = &self.modal {
                        if self.modal_index == 0 {
                            self.cycle_hook_event(false);
                            return Ok(());
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(ModalType::AddHook { .. }) = &self.modal {
                        if self.modal_index == 0 {
                            self.cycle_hook_event(true);
                            return Ok(());
                        }
                    }
                }
                _ => {}
            }
        }

        // Map key to action and handle
        if let Some(action) = map_key_to_action(key, &self.input_mode) {
            self.handle_action(action)?;
        }
        Ok(())
    }
    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::MoveLeft => self.focus = Focus::List,
            Action::MoveRight => self.focus = Focus::Detail,
            Action::NextTab => self.next_tab(),
            Action::PrevTab => self.prev_tab(),
            Action::GoToTab(idx) => self.go_to_tab(idx),
            Action::Select => self.select(),
            Action::Cancel => self.cancel(),
            Action::Add => self.start_add(),
            Action::Edit => self.start_edit(),
            Action::Delete => self.start_delete(),
            Action::Save => self.save_settings()?,
            Action::Toggle => self.toggle_item()?,
            Action::ShowHelp => self.show_help(),
            Action::Quit => self.try_quit(),
            Action::ForceQuit => self.running = false,
            Action::Refresh => self.reload_all()?,
            Action::ToggleScope => self.toggle_scope()?,
            Action::ChangePlatform => self.start_change_platform(),
            Action::ManageVersions => self.start_manage_versions(),
            Action::Confirm => self.confirm_modal()?,
            Action::Dismiss => self.dismiss_modal(),
            _ => {}
        }
        Ok(())
    }

    fn move_up(&mut self) {
        if self.input_mode == InputMode::Modal {
            if let Some(ModalType::SelectMcpPreset { selected_index }) = &mut self.modal {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            } else if let Some(ModalType::ChangePlatform { selected_index }) = &mut self.modal {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            } else if let Some(ModalType::ManageVersions { selected_index, .. }) = &mut self.modal {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            } else if self.modal_index > 0 {
                self.modal_index -= 1;
            }
        } else if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.input_mode == InputMode::Modal {
            if let Some(ModalType::SelectMcpPreset { selected_index }) = &mut self.modal {
                let max_index = PRESET_MCP_SERVERS.len();
                if *selected_index < max_index {
                    *selected_index += 1;
                }
            } else if let Some(ModalType::ChangePlatform { selected_index }) = &mut self.modal {
                let max_index = AgentType::all().len().saturating_sub(1);
                if *selected_index < max_index {
                    *selected_index += 1;
                }
            } else if let Some(ModalType::ManageVersions {
                versions,
                selected_index,
                ..
            }) = &mut self.modal
            {
                if !versions.is_empty() && *selected_index < versions.len().saturating_sub(1) {
                    *selected_index += 1;
                }
            } else {
                self.modal_index += 1;
            }
        } else {
            let list_len = self.current_list().len();
            if list_len > 0 && self.list_index < list_len - 1 {
                self.list_index += 1;
            }
        }
    }

    fn next_tab(&mut self) {
        let tabs = Tab::all();
        let current_idx = self.tab.index();
        let next_idx = (current_idx + 1) % tabs.len();
        self.go_to_tab(next_idx);
    }

    fn prev_tab(&mut self) {
        let len = Tab::all().len();
        let current_idx = self.tab.index();
        let prev_idx = current_idx.checked_sub(1).unwrap_or(len - 1);
        self.go_to_tab(prev_idx);
    }

    fn go_to_tab(&mut self, idx: usize) {
        if let Some(tab) = Tab::from_index(idx) {
            self.tab = tab;
            self.list_index = 0;
            self.focus = Focus::List;
        }
    }

    fn select(&mut self) {
        if self.focus == Focus::List && !self.current_list().is_empty() {
            if self.tab == Tab::Profiles {
                if let Some(ListItem::Profile(profile)) = self.selected_item() {
                    self.modal = Some(ModalType::Confirm {
                        title: "Apply Profile".to_string(),
                        message: format!(
                            "Apply profile '{}' to {} scope ({})?",
                            profile.name,
                            match self.current_scope {
                                InstallScope::Global => "Global",
                                InstallScope::Local => "Local",
                            },
                            self.current_platform.display_name()
                        ),
                        on_confirm: ConfirmAction::ApplyProfile {
                            name: profile.name.clone(),
                        },
                    });
                    self.input_mode = InputMode::Modal;
                }
            } else {
                self.focus = Focus::Detail;
            }
        }
    }

    fn cancel(&mut self) {
        match self.input_mode {
            InputMode::Insert => {
                self.input_mode = InputMode::Normal;
                self.modal = None;
            }
            InputMode::Modal => {
                self.dismiss_modal();
            }
            InputMode::Normal => {
                if self.focus == Focus::Detail {
                    self.focus = Focus::List;
                }
            }
        }
    }

    fn start_add(&mut self) {
        let modal = match self.tab {
            Tab::Home => return,
            Tab::Hooks => ModalType::AddHook {
                event: HookEvent::UserPromptSubmit,
                hook_type: HookType::Command,
                target: String::new(),
            },
            Tab::Skills => ModalType::AddSkill {
                name: String::new(),
            },
            Tab::Plugins => return,
            Tab::Commands => ModalType::AddCommand {
                name: String::new(),
            },
            Tab::Agents => ModalType::AddAgent {
                name: String::new(),
            },
            Tab::Mcp => ModalType::SelectMcpPreset { selected_index: 0 },
            Tab::Profiles => ModalType::AddProfile {
                name: String::new(),
            },
        };
        self.modal = Some(modal);
        self.input_mode = InputMode::Modal;
        self.modal_index = 0;
    }

    fn start_add_custom_mcp(&mut self) {
        self.modal = Some(ModalType::AddMcp {
            name: String::new(),
            command: String::new(),
            args: String::new(),
        });
        self.input_mode = InputMode::Insert;
        self.modal_index = 0;
    }

    fn install_preset_mcp(&mut self, preset: &PresetMcpServer) {
        if self.settings.mcp_servers.contains_key(preset.name) {
            self.message = Some(format!("MCP server '{}' already exists", preset.name));
            return;
        }

        let server = preset.to_mcp_server();
        self.settings
            .mcp_servers
            .insert(preset.name.to_string(), server);
        self.unsaved_changes = true;
        self.message = Some(format!("Added MCP server '{}'", preset.name));
    }

    fn start_edit(&mut self) {
        if let Some(item) = self.selected_item() {
            match item {
                ListItem::Plugin(_) => {
                    self.toggle_item().ok();
                }
                _ => {
                    self.message = Some("Edit: Open in external editor".to_string());
                }
            }
        }
    }

    fn start_delete(&mut self) {
        if let Some(ModalType::ManageVersions {
            skill_name,
            versions,
            selected_index,
        }) = &self.modal
        {
            if let Some(version) = versions.get(*selected_index) {
                self.modal = Some(ModalType::Confirm {
                    title: "Delete Version".to_string(),
                    message: format!(
                        "Are you sure you want to delete version {} of skill '{}'?",
                        version.version, skill_name
                    ),
                    on_confirm: ConfirmAction::DeleteVersion {
                        skill_name: skill_name.clone(),
                        version: version.version.clone(),
                    },
                });
                return;
            }
        }

        if let Some(item) = self.selected_item() {
            let (title, message, on_confirm) = match item {
                ListItem::Hook { event, index, .. } => (
                    "Delete Hook".to_string(),
                    format!("Are you sure you want to delete this {} hook?", event),
                    ConfirmAction::DeleteHook { event, index },
                ),
                ListItem::Skill(s) => (
                    "Delete Skill".to_string(),
                    format!("Are you sure you want to delete skill '{}'?", s.name),
                    ConfirmAction::DeleteSkill { name: s.name },
                ),
                ListItem::Plugin(_) => return,
                ListItem::Command(c) => (
                    "Delete Command".to_string(),
                    format!("Are you sure you want to delete command '{}'?", c.name),
                    ConfirmAction::DeleteCommand { name: c.name },
                ),
                ListItem::Agent(a) => (
                    "Delete Agent".to_string(),
                    format!("Are you sure you want to delete agent '{}'?", a.name),
                    ConfirmAction::DeleteAgent { name: a.name },
                ),
                ListItem::Mcp { name, .. } => (
                    "Delete MCP Server".to_string(),
                    format!("Are you sure you want to delete MCP server '{}'?", name),
                    ConfirmAction::DeleteMcp { name },
                ),
                ListItem::Profile(p) => (
                    "Delete Profile".to_string(),
                    format!("Are you sure you want to delete profile '{}'?", p.name),
                    ConfirmAction::DeleteProfile { name: p.name },
                ),
            };

            self.modal = Some(ModalType::Confirm {
                title,
                message,
                on_confirm,
            });
            self.input_mode = InputMode::Modal;
        }
    }

    fn toggle_item(&mut self) -> Result<()> {
        if let Tab::Plugins = self.tab {
            if let Some(ListItem::Plugin(p)) = self.selected_item() {
                let new_state = !p.enabled;
                self.settings
                    .enabled_plugins
                    .insert(p.id.clone(), new_state);
                if let Some(plugin) = self.plugins.iter_mut().find(|pl| pl.id == p.id) {
                    plugin.enabled = new_state;
                }
                self.unsaved_changes = true;
            }
        }
        Ok(())
    }

    fn show_help(&mut self) {
        self.modal = Some(ModalType::Help);
        self.input_mode = InputMode::Modal;
    }

    fn try_quit(&mut self) {
        if self.unsaved_changes {
            self.modal = Some(ModalType::Confirm {
                title: "Unsaved Changes".to_string(),
                message: "You have unsaved changes. Quit anyway?".to_string(),
                on_confirm: ConfirmAction::Quit,
            });
            self.input_mode = InputMode::Modal;
        } else {
            self.running = false;
        }
    }

    fn toggle_scope(&mut self) -> Result<()> {
        self.current_scope = match self.current_scope {
            InstallScope::Global => InstallScope::Local,
            InstallScope::Local => InstallScope::Global,
        };
        self.reload_all()?;
        self.message = Some(format!(
            "Switched to {} scope",
            match self.current_scope {
                InstallScope::Global => "Global",
                InstallScope::Local => "Local",
            }
        ));
        Ok(())
    }

    fn start_change_platform(&mut self) {
        let current_idx = AgentType::all()
            .iter()
            .position(|a| *a == self.current_platform)
            .unwrap_or(0);
        self.modal = Some(ModalType::ChangePlatform {
            selected_index: current_idx,
        });
        self.input_mode = InputMode::Modal;
    }

    fn start_manage_versions(&mut self) {
        if self.tab != Tab::Skills {
            self.message = Some("Version management is only available for Skills".to_string());
            return;
        }

        if let Some(ListItem::Skill(skill)) = self.selected_item() {
            if let Ok(versions) = self.version_manager.list_versions(&skill.name) {
                if versions.is_empty() {
                    self.message = Some(format!("No versions found for skill '{}'", skill.name));
                    return;
                }
                self.modal = Some(ModalType::ManageVersions {
                    skill_name: skill.name.clone(),
                    versions,
                    selected_index: 0,
                });
                self.input_mode = InputMode::Modal;
            } else {
                self.message = Some(format!(
                    "Failed to list versions for skill '{}'",
                    skill.name
                ));
            }
        }
    }

    fn confirm_modal(&mut self) -> Result<()> {
        if let Some(modal) = self.modal.take() {
            match modal {
                ModalType::Confirm { on_confirm, .. } => {
                    self.execute_confirm_action(on_confirm)?;
                }
                ModalType::ChangePlatform { selected_index } => {
                    if let Some(agent) = AgentType::all().get(selected_index) {
                        self.current_platform = *agent;
                        self.reload_all()?;
                        self.message = Some(format!("Switched to {}", agent.display_name()));
                    }
                }
                ModalType::ManageVersions {
                    skill_name,
                    versions,
                    selected_index,
                } => {
                    if let Some(version) = versions.get(selected_index) {
                        self.modal = Some(ModalType::Confirm {
                            title: "Switch Version".to_string(),
                            message: format!(
                                "Switch skill '{}' to version {}?",
                                skill_name, version.version
                            ),
                            on_confirm: ConfirmAction::SwitchVersion {
                                skill_name,
                                version: version.version.clone(),
                            },
                        });
                        self.input_mode = InputMode::Modal;
                        return Ok(());
                    }
                }
                ModalType::AddHook {
                    event,
                    hook_type,
                    target,
                } => {
                    if !target.is_empty() {
                        let action = match hook_type {
                            HookType::Command => HookAction::Command {
                                command: target,
                                timeout: None,
                            },
                            HookType::Url => HookAction::Url { url: target },
                        };
                        let hook = Hook {
                            matcher: None,
                            action,
                        };
                        let hook_group = HookGroup { hooks: vec![hook] };
                        self.settings
                            .hooks
                            .get_hook_groups_mut(&event)
                            .push(hook_group);
                        self.unsaved_changes = true;
                    }
                }
                ModalType::AddSkill { name } => {
                    if !name.is_empty() {
                        self.create_skill(&name)?;
                    }
                }
                ModalType::AddCommand { name } => {
                    if !name.is_empty() {
                        self.create_command(&name)?;
                    }
                }
                ModalType::AddAgent { name } => {
                    if !name.is_empty() {
                        self.create_agent(&name)?;
                    }
                }
                ModalType::AddProfile { name } => {
                    if !name.is_empty() {
                        self.create_profile(&name)?;
                    }
                }
                ModalType::AddMcp {
                    name,
                    command,
                    args,
                } => {
                    if !name.is_empty() && !command.is_empty() {
                        if !validate_server_name(&name) {
                            self.message = Some(
                                "Invalid name: use letters, numbers, dashes, underscores only"
                                    .to_string(),
                            );
                            return Ok(());
                        }
                        if !validate_command(&command) {
                            self.message = Some(format!(
                                "Invalid command. Allowed: {}",
                                ALLOWED_MCP_COMMANDS.join(", ")
                            ));
                            return Ok(());
                        }
                        let args: Vec<String> =
                            args.split_whitespace().map(|s| s.to_string()).collect();
                        let server = McpServer::new(command).with_args(args);
                        self.settings.mcp_servers.insert(name, server);
                        self.unsaved_changes = true;
                    }
                }
                ModalType::SelectMcpPreset { selected_index } => {
                    if selected_index >= PRESET_MCP_SERVERS.len() {
                        self.start_add_custom_mcp();
                        return Ok(());
                    } else {
                        let preset = &PRESET_MCP_SERVERS[selected_index];
                        self.install_preset_mcp(preset);
                    }
                }
                ModalType::Help => {}
            }
        }
        self.input_mode = InputMode::Normal;
        self.modal_index = 0;
        Ok(())
    }

    fn dismiss_modal(&mut self) {
        self.modal = None;
        self.input_mode = InputMode::Normal;
        self.modal_index = 0;
    }

    fn execute_confirm_action(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::DeleteHook { event, index } => {
                let mut count = 0;
                for ev in HookEvent::all() {
                    let groups = self.settings.hooks.get_hook_groups_mut(&ev);
                    for gi in 0..groups.len() {
                        for hi in 0..groups[gi].hooks.len() {
                            if count == index && ev == event {
                                groups[gi].hooks.remove(hi);
                                if groups[gi].hooks.is_empty() {
                                    groups.remove(gi);
                                }
                                self.unsaved_changes = true;
                                self.adjust_list_index();
                                return Ok(());
                            }
                            count += 1;
                        }
                    }
                }
            }
            ConfirmAction::DeleteSkill { name } => {
                let skill_dir = crate::config::get_skills_dir().join(&name);
                ResourceManager::delete_dir(skill_dir)?;
                self.skills.retain(|s| s.name != name);
                self.adjust_list_index();
            }
            ConfirmAction::DeleteCommand { name } => {
                let cmd_path = crate::config::get_commands_dir().join(format!("{}.md", name));
                ResourceManager::delete_file(cmd_path)?;
                self.commands.retain(|c| c.name != name);
                self.adjust_list_index();
            }
            ConfirmAction::DeleteAgent { name } => {
                let agent_path = crate::config::get_agents_dir().join(format!("{}.md", name));
                ResourceManager::delete_file(agent_path)?;
                self.agents.retain(|a| a.name != name);
                self.adjust_list_index();
            }
            ConfirmAction::DeleteMcp { name } => {
                self.settings.mcp_servers.remove(&name);
                self.unsaved_changes = true;
                self.adjust_list_index();
            }
            ConfirmAction::SwitchVersion {
                skill_name,
                version,
            } => {
                if let Err(e) = self.version_manager.switch_version(&skill_name, &version) {
                    self.message = Some(format!("Failed to switch version: {}", e));
                } else {
                    self.message = Some(format!(
                        "Switched skill '{}' to version {}",
                        skill_name, version
                    ));
                    self.reload_all()?;
                }
            }
            ConfirmAction::DeleteVersion {
                skill_name,
                version,
            } => {
                if let Err(e) = self.version_manager.remove_version(&skill_name, &version) {
                    self.message = Some(format!("Failed to delete version: {}", e));
                } else {
                    self.message = Some(format!("Deleted version {}", version));
                    // Try to reopen manage versions modal
                    if let Ok(versions) = self.version_manager.list_versions(&skill_name) {
                        if !versions.is_empty() {
                            self.modal = Some(ModalType::ManageVersions {
                                skill_name,
                                versions,
                                selected_index: 0,
                            });
                            self.input_mode = InputMode::Modal;
                            return Ok(());
                        }
                    }
                    self.modal = None;
                    self.input_mode = InputMode::Normal;
                }
            }
            ConfirmAction::ApplyProfile { name } => {
                let target_dir = match self.current_scope {
                    InstallScope::Global => dirs::home_dir().unwrap_or_else(|| PathBuf::from("~")),
                    InstallScope::Local => {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    }
                };
                if let Err(e) = export_profile(&name, &target_dir, self.current_platform, false) {
                    self.message = Some(format!("Failed to apply profile: {}", e));
                } else {
                    self.reload_all()?;
                    self.message = Some(format!("Applied profile '{}'", name));
                }
            }
            ConfirmAction::DeleteProfile { name } => {
                if let Err(e) = delete_profile(&name) {
                    self.message = Some(format!("Failed to delete profile: {}", e));
                } else {
                    self.profiles.retain(|p| p.name != name);
                    self.adjust_list_index();
                    self.message = Some(format!("Deleted profile '{}'", name));
                }
            }
            ConfirmAction::Quit => {
                self.running = false;
            }
        }
        Ok(())
    }

    fn adjust_list_index(&mut self) {
        let list_len = self.current_list().len();
        self.list_index = self.list_index.min(list_len.saturating_sub(1));
    }

    fn create_resource(&mut self, path: PathBuf, content: String, msg: String) -> Result<()> {
        ResourceManager::create(path, content)?;
        self.reload_all()?;
        self.message = Some(msg);
        Ok(())
    }

    fn create_skill(&mut self, name: &str) -> Result<()> {
        let path = crate::config::get_skills_dir().join(name).join("SKILL.md");
        let content = create_skill_template(name);
        self.create_resource(path, content, format!("Created skill '{}'", name))
    }

    fn create_command(&mut self, name: &str) -> Result<()> {
        let path = crate::config::get_commands_dir().join(format!("{}.md", name));
        let content = create_command_template(name);
        self.create_resource(path, content, format!("Created command '{}'", name))
    }

    fn create_agent(&mut self, name: &str) -> Result<()> {
        let path = crate::config::get_agents_dir().join(format!("{}.md", name));
        let content = create_agent_template(name);
        self.create_resource(path, content, format!("Created agent '{}'", name))
    }

    fn create_profile(&mut self, name: &str) -> Result<()> {
        create_profile_from_current(name, None, self.current_platform)?;
        self.reload_all()?;
        self.message = Some(format!("Created profile '{}'", name));
        Ok(())
    }

    pub fn update_modal_field(&mut self, c: char) {
        if let Some(ref mut modal) = self.modal {
            match modal {
                ModalType::AddHook { target, .. } => target.push(c),
                ModalType::AddSkill { name }
                | ModalType::AddCommand { name }
                | ModalType::AddAgent { name }
                | ModalType::AddProfile { name } => name.push(c),
                ModalType::AddMcp {
                    name,
                    command,
                    args,
                } => match self.modal_index {
                    0 => name.push(c),
                    1 => command.push(c),
                    2 => args.push(c),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn backspace_modal_field(&mut self) {
        if let Some(ref mut modal) = self.modal {
            match modal {
                ModalType::AddHook { target, .. } => {
                    target.pop();
                }
                ModalType::AddSkill { name }
                | ModalType::AddCommand { name }
                | ModalType::AddAgent { name }
                | ModalType::AddProfile { name } => {
                    name.pop();
                }
                ModalType::AddMcp {
                    name,
                    command,
                    args,
                } => match self.modal_index {
                    0 => {
                        name.pop();
                    }
                    1 => {
                        command.pop();
                    }
                    2 => {
                        args.pop();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn cycle_modal_field(&mut self, forward: bool) {
        if let Some(ref modal) = self.modal {
            let max = match modal {
                ModalType::AddHook { .. } | ModalType::AddMcp { .. } => 2,
                _ => 0,
            };
            self.modal_index = match forward {
                true => (self.modal_index + 1) % (max + 1),
                false if self.modal_index == 0 => max,
                false => self.modal_index - 1,
            };
        }
    }

    pub fn cycle_hook_event(&mut self, forward: bool) {
        if let Some(ModalType::AddHook { event, .. }) = &mut self.modal {
            let events = HookEvent::all();
            let current_idx = events.iter().position(|e| *e == *event).unwrap_or(0);
            let len = events.len();
            let new_idx = match forward {
                true => (current_idx + 1) % len,
                false if current_idx == 0 => len - 1,
                false => current_idx - 1,
            };
            *event = events[new_idx].clone();
        }
    }

    pub fn cycle_hook_type(&mut self) {
        if let Some(ModalType::AddHook { hook_type, .. }) = &mut self.modal {
            *hook_type = match hook_type {
                HookType::Command => HookType::Url,
                HookType::Url => HookType::Command,
            };
        }
    }
}
