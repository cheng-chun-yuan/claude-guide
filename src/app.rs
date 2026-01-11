use anyhow::Result;
use std::path::PathBuf;

use crate::actions::Action;
use crate::config::{
    scan_agents, scan_commands, scan_plugins, scan_skills, Agent, Command, Hook, HookAction,
    HookEvent, HookGroup, McpServer, Plugin, Settings, Skill,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Hooks,
    Skills,
    Plugins,
    Commands,
    Agents,
    Mcp,
}

impl Tab {
    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Hooks,
            Tab::Skills,
            Tab::Plugins,
            Tab::Commands,
            Tab::Agents,
            Tab::Mcp,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Hooks => "Hooks",
            Tab::Skills => "Skills",
            Tab::Plugins => "Plugins",
            Tab::Commands => "Commands",
            Tab::Agents => "Agents",
            Tab::Mcp => "MCP",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Hooks => 0,
            Tab::Skills => 1,
            Tab::Plugins => 2,
            Tab::Commands => 3,
            Tab::Agents => 4,
            Tab::Mcp => 5,
        }
    }

    pub fn from_index(index: usize) -> Option<Tab> {
        match index {
            0 => Some(Tab::Hooks),
            1 => Some(Tab::Skills),
            2 => Some(Tab::Plugins),
            3 => Some(Tab::Commands),
            4 => Some(Tab::Agents),
            5 => Some(Tab::Mcp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

#[derive(Debug, Clone)]
pub enum ModalType {
    Confirm {
        title: String,
        message: String,
        on_confirm: ConfirmAction,
    },
    AddHook {
        event: HookEvent,
        hook_type: HookType,
        target: String,
    },
    AddSkill {
        name: String,
    },
    AddCommand {
        name: String,
    },
    AddAgent {
        name: String,
    },
    AddMcp {
        name: String,
        command: String,
        args: String,
    },
    Help,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteHook { event: HookEvent, index: usize },
    DeleteSkill { name: String },
    DeleteCommand { name: String },
    DeleteAgent { name: String },
    DeleteMcp { name: String },
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    Command,
    Url,
}

impl HookType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookType::Command => "command",
            HookType::Url => "url",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ListItem {
    Hook {
        event: HookEvent,
        index: usize,
        hook: Hook,
    },
    Skill(Skill),
    Plugin(Plugin),
    Command(Command),
    Agent(Agent),
    Mcp {
        name: String,
        server: McpServer,
    },
}

impl ListItem {
    pub fn display_name(&self) -> String {
        match self {
            ListItem::Hook { event, hook, .. } => {
                format!("[{}] {}", event, hook.get_type())
            }
            ListItem::Skill(s) => s.display_name().to_string(),
            ListItem::Plugin(p) => {
                let status = if p.enabled { "+" } else { "-" };
                format!("[{}] {}", status, p.display_name())
            }
            ListItem::Command(c) => c.name.clone(),
            ListItem::Agent(a) => a.display_name().to_string(),
            ListItem::Mcp { name, .. } => name.clone(),
        }
    }
}

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

    // Configuration
    pub settings: Settings,
    pub settings_path: PathBuf,
    #[allow(dead_code)]
    pub claude_dir: PathBuf,

    // Loaded data
    pub skills: Vec<Skill>,
    pub plugins: Vec<Plugin>,
    pub commands: Vec<Command>,
    pub agents: Vec<Agent>,
}

impl App {
    pub fn new() -> Result<Self> {
        let claude_dir = crate::config::get_claude_dir();
        let settings_path = crate::config::get_settings_path();

        let settings = Settings::load(&settings_path).unwrap_or_default();

        let mut app = Self {
            running: true,
            tab: Tab::Hooks,
            input_mode: InputMode::Normal,
            focus: Focus::List,
            list_index: 0,
            modal: None,
            modal_index: 0,
            message: None,
            unsaved_changes: false,
            settings,
            settings_path,
            claude_dir,
            skills: Vec::new(),
            plugins: Vec::new(),
            commands: Vec::new(),
            agents: Vec::new(),
        };

        app.reload_all()?;
        Ok(app)
    }

    pub fn reload_all(&mut self) -> Result<()> {
        self.settings = Settings::load(&self.settings_path).unwrap_or_default();
        self.skills = scan_skills(&crate::config::get_skills_dir()).unwrap_or_default();
        self.plugins = scan_plugins(
            &crate::config::get_plugins_dir(),
            &self.settings.enabled_plugins,
        )
        .unwrap_or_default();
        self.commands = scan_commands(&crate::config::get_commands_dir()).unwrap_or_default();
        self.agents = scan_agents(&crate::config::get_agents_dir()).unwrap_or_default();
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
        }
    }

    pub fn selected_item(&self) -> Option<ListItem> {
        let list = self.current_list();
        list.get(self.list_index).cloned()
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
            Action::Confirm => self.confirm_modal()?,
            Action::Dismiss => self.dismiss_modal(),
            _ => {}
        }
        Ok(())
    }

    fn move_up(&mut self) {
        if self.input_mode == InputMode::Modal {
            if self.modal_index > 0 {
                self.modal_index -= 1;
            }
        } else if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.input_mode == InputMode::Modal {
            self.modal_index += 1;
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
        let tabs = Tab::all();
        let current_idx = self.tab.index();
        let prev_idx = if current_idx == 0 {
            tabs.len() - 1
        } else {
            current_idx - 1
        };
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
            self.focus = Focus::Detail;
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
            Tab::Hooks => ModalType::AddHook {
                event: HookEvent::UserPromptSubmit,
                hook_type: HookType::Command,
                target: String::new(),
            },
            Tab::Skills => ModalType::AddSkill {
                name: String::new(),
            },
            Tab::Plugins => return, // Plugins can't be added, only toggled
            Tab::Commands => ModalType::AddCommand {
                name: String::new(),
            },
            Tab::Agents => ModalType::AddAgent {
                name: String::new(),
            },
            Tab::Mcp => ModalType::AddMcp {
                name: String::new(),
                command: String::new(),
                args: String::new(),
            },
        };
        self.modal = Some(modal);
        self.input_mode = InputMode::Insert;
        self.modal_index = 0;
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
                ListItem::Plugin(_) => return, // Plugins can't be deleted
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
                // Toggle in the HashMap
                let new_state = !p.enabled;
                self.settings
                    .enabled_plugins
                    .insert(p.id.clone(), new_state);
                // Update local plugins list
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

    fn confirm_modal(&mut self) -> Result<()> {
        if let Some(modal) = self.modal.take() {
            match modal {
                ModalType::Confirm { on_confirm, .. } => {
                    self.execute_confirm_action(on_confirm)?;
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
                ModalType::AddMcp {
                    name,
                    command,
                    args,
                } => {
                    if !name.is_empty() && !command.is_empty() {
                        let args: Vec<String> =
                            args.split_whitespace().map(|s| s.to_string()).collect();
                        let server = McpServer::new(command).with_args(args);
                        self.settings.mcp_servers.insert(name, server);
                        self.unsaved_changes = true;
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
                // Find the actual hook in the nested structure
                let mut count = 0;
                for ev in HookEvent::all() {
                    let groups = self.settings.hooks.get_hook_groups_mut(&ev);
                    for gi in 0..groups.len() {
                        for hi in 0..groups[gi].hooks.len() {
                            if count == index && ev == event {
                                groups[gi].hooks.remove(hi);
                                // Remove empty groups
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
                if skill_dir.exists() {
                    std::fs::remove_dir_all(&skill_dir)?;
                }
                self.skills.retain(|s| s.name != name);
                self.adjust_list_index();
            }
            ConfirmAction::DeleteCommand { name } => {
                let cmd_path = crate::config::get_commands_dir().join(format!("{}.md", name));
                if cmd_path.exists() {
                    std::fs::remove_file(&cmd_path)?;
                }
                self.commands.retain(|c| c.name != name);
                self.adjust_list_index();
            }
            ConfirmAction::DeleteAgent { name } => {
                let agent_path = crate::config::get_agents_dir().join(format!("{}.md", name));
                if agent_path.exists() {
                    std::fs::remove_file(&agent_path)?;
                }
                self.agents.retain(|a| a.name != name);
                self.adjust_list_index();
            }
            ConfirmAction::DeleteMcp { name } => {
                self.settings.mcp_servers.remove(&name);
                self.unsaved_changes = true;
                self.adjust_list_index();
            }
            ConfirmAction::Quit => {
                self.running = false;
            }
        }
        Ok(())
    }

    fn adjust_list_index(&mut self) {
        let list_len = self.current_list().len();
        if list_len == 0 {
            self.list_index = 0;
        } else if self.list_index >= list_len {
            self.list_index = list_len - 1;
        }
    }

    fn create_skill(&mut self, name: &str) -> Result<()> {
        let skills_dir = crate::config::get_skills_dir();
        let skill_dir = skills_dir.join(name);
        std::fs::create_dir_all(&skill_dir)?;

        let skill_file = skill_dir.join("SKILL.md");
        let content = crate::config::skills::create_skill_template(name);
        std::fs::write(&skill_file, content)?;

        self.reload_all()?;
        self.message = Some(format!("Created skill '{}'", name));
        Ok(())
    }

    fn create_command(&mut self, name: &str) -> Result<()> {
        let commands_dir = crate::config::get_commands_dir();
        std::fs::create_dir_all(&commands_dir)?;

        let cmd_path = commands_dir.join(format!("{}.md", name));
        let content = crate::config::commands::create_command_template(name);
        std::fs::write(&cmd_path, content)?;

        self.reload_all()?;
        self.message = Some(format!("Created command '{}'", name));
        Ok(())
    }

    fn create_agent(&mut self, name: &str) -> Result<()> {
        let agents_dir = crate::config::get_agents_dir();
        std::fs::create_dir_all(&agents_dir)?;

        let agent_path = agents_dir.join(format!("{}.md", name));
        let content = crate::config::agents::create_agent_template(name);
        std::fs::write(&agent_path, content)?;

        self.reload_all()?;
        self.message = Some(format!("Created agent '{}'", name));
        Ok(())
    }

    pub fn update_modal_field(&mut self, c: char) {
        if let Some(ref mut modal) = self.modal {
            match modal {
                ModalType::AddHook { target, .. } => target.push(c),
                ModalType::AddSkill { name } => name.push(c),
                ModalType::AddCommand { name } => name.push(c),
                ModalType::AddAgent { name } => name.push(c),
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
                ModalType::AddSkill { name } => {
                    name.pop();
                }
                ModalType::AddCommand { name } => {
                    name.pop();
                }
                ModalType::AddAgent { name } => {
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
                ModalType::AddHook { .. } => 2,
                ModalType::AddMcp { .. } => 2,
                _ => 0,
            };
            if forward {
                self.modal_index = (self.modal_index + 1) % (max + 1);
            } else {
                self.modal_index = if self.modal_index == 0 {
                    max
                } else {
                    self.modal_index - 1
                };
            }
        }
    }

    pub fn cycle_hook_event(&mut self, forward: bool) {
        if let Some(ModalType::AddHook { event, .. }) = &mut self.modal {
            let events = HookEvent::all();
            let current_idx = events.iter().position(|e| e == event).unwrap_or(0);
            let new_idx = if forward {
                (current_idx + 1) % events.len()
            } else if current_idx == 0 {
                events.len() - 1
            } else {
                current_idx - 1
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
