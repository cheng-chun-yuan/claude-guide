use crate::config::{Agent, Command, Hook, HookEvent, McpServer, Plugin, Skill};
use crate::version::{SkillProfile, SkillVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Hooks,
    Skills,
    Plugins,
    Commands,
    Agents,
    Mcp,
    Profiles,
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
            Tab::Profiles,
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
            Tab::Profiles => "Profiles",
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
            Tab::Profiles => 6,
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
            6 => Some(Tab::Profiles),
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
pub enum ConfirmAction {
    DeleteHook { event: HookEvent, index: usize },
    DeleteSkill { name: String },
    DeleteCommand { name: String },
    DeleteAgent { name: String },
    DeleteMcp { name: String },
    DeleteVersion { skill_name: String, version: String },
    DeleteProfile { name: String },
    SwitchVersion { skill_name: String, version: String },
    ApplyProfile { name: String },
    Quit,
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
    AddProfile {
        name: String,
    },
    SelectMcpPreset {
        selected_index: usize,
    },
    ChangePlatform {
        selected_index: usize,
    },
    ManageVersions {
        skill_name: String,
        versions: Vec<SkillVersion>,
        selected_index: usize,
    },
    Help,
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
    Profile(SkillProfile),
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
            ListItem::Profile(p) => p.name.clone(),
        }
    }
}
