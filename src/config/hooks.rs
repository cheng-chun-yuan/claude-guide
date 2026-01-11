use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
    SubagentStop,
    UserPromptSubmit,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::Notification => "Notification",
            HookEvent::Stop => "Stop",
            HookEvent::SubagentStop => "SubagentStop",
            HookEvent::UserPromptSubmit => "UserPromptSubmit",
        }
    }

    pub fn all() -> Vec<HookEvent> {
        vec![
            HookEvent::PreToolUse,
            HookEvent::PostToolUse,
            HookEvent::Notification,
            HookEvent::Stop,
            HookEvent::SubagentStop,
            HookEvent::UserPromptSubmit,
        ]
    }
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HookMatcher {
    #[serde(rename = "toolName")]
    ToolName { pattern: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HookAction {
    #[serde(rename = "command")]
    Command {
        command: String,
        #[serde(default)]
        timeout: Option<u64>,
    },
    #[serde(rename = "url")]
    Url { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matcher: Option<HookMatcher>,
    #[serde(flatten)]
    pub action: HookAction,
}

impl Hook {
    pub fn get_type(&self) -> &'static str {
        match &self.action {
            HookAction::Command { .. } => "command",
            HookAction::Url { .. } => "url",
        }
    }

    pub fn get_target(&self) -> String {
        match &self.action {
            HookAction::Command { command, .. } => command.clone(),
            HookAction::Url { url } => url.clone(),
        }
    }

    pub fn get_timeout(&self) -> Option<u64> {
        match &self.action {
            HookAction::Command { timeout, .. } => *timeout,
            HookAction::Url { .. } => None,
        }
    }
}

/// A group of hooks (the actual format in settings.json has nested "hooks" array)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookGroup {
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default, rename = "PreToolUse")]
    pub pre_tool_use: Vec<HookGroup>,
    #[serde(default, rename = "PostToolUse")]
    pub post_tool_use: Vec<HookGroup>,
    #[serde(default, rename = "Notification")]
    pub notification: Vec<HookGroup>,
    #[serde(default, rename = "Stop")]
    pub stop: Vec<HookGroup>,
    #[serde(default, rename = "SubagentStop")]
    pub subagent_stop: Vec<HookGroup>,
    #[serde(default, rename = "UserPromptSubmit")]
    pub user_prompt_submit: Vec<HookGroup>,
}

impl HooksConfig {
    pub fn get_hook_groups(&self, event: &HookEvent) -> &Vec<HookGroup> {
        match event {
            HookEvent::PreToolUse => &self.pre_tool_use,
            HookEvent::PostToolUse => &self.post_tool_use,
            HookEvent::Notification => &self.notification,
            HookEvent::Stop => &self.stop,
            HookEvent::SubagentStop => &self.subagent_stop,
            HookEvent::UserPromptSubmit => &self.user_prompt_submit,
        }
    }

    pub fn get_hook_groups_mut(&mut self, event: &HookEvent) -> &mut Vec<HookGroup> {
        match event {
            HookEvent::PreToolUse => &mut self.pre_tool_use,
            HookEvent::PostToolUse => &mut self.post_tool_use,
            HookEvent::Notification => &mut self.notification,
            HookEvent::Stop => &mut self.stop,
            HookEvent::SubagentStop => &mut self.subagent_stop,
            HookEvent::UserPromptSubmit => &mut self.user_prompt_submit,
        }
    }

    /// Get all hooks flattened from all groups
    pub fn all_hooks(&self) -> Vec<(HookEvent, &Hook)> {
        let mut result = Vec::new();
        for event in HookEvent::all() {
            for group in self.get_hook_groups(&event) {
                for hook in &group.hooks {
                    result.push((event.clone(), hook));
                }
            }
        }
        result
    }
}
