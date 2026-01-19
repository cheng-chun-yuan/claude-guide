use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    ClaudeCode,
    Cursor,
    Codex,
    OpenCode,
    Amp,
    Roo,
    Goose,
    Kilo,
    Antigravity,
    GitHubCopilot,
}

impl AgentType {
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::Cursor => "Cursor",
            AgentType::Codex => "Codex",
            AgentType::OpenCode => "OpenCode",
            AgentType::Amp => "Amp",
            AgentType::Roo => "Roo Code",
            AgentType::Goose => "Goose",
            AgentType::Kilo => "Kilo Code",
            AgentType::Antigravity => "Antigravity",
            AgentType::GitHubCopilot => "GitHub Copilot",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "claude",
            AgentType::Cursor => "cursor",
            AgentType::Codex => "codex",
            AgentType::OpenCode => "opencode",
            AgentType::Amp => "amp",
            AgentType::Roo => "roo",
            AgentType::Goose => "goose",
            AgentType::Kilo => "kilo",
            AgentType::Antigravity => "antigravity",
            AgentType::GitHubCopilot => "github-copilot",
        }
    }

    pub fn project_dir(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => ".claude/skills",
            AgentType::Cursor => ".cursor/skills",
            AgentType::Codex => ".codex/skills",
            AgentType::OpenCode => ".opencode/skill",
            AgentType::Amp => ".agents/skills",
            AgentType::Roo => ".roo/skills",
            AgentType::Goose => ".goose/skills",
            AgentType::Kilo => ".kilocode/skills",
            AgentType::Antigravity => ".agent/skills",
            AgentType::GitHubCopilot => ".github/skills",
        }
    }

    pub fn global_dir(&self) -> PathBuf {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("~"));
        match self {
            AgentType::ClaudeCode => home.join(".claude/skills"),
            AgentType::Cursor => home.join(".cursor/skills"),
            AgentType::Codex => home.join(".codex/skills"),
            AgentType::OpenCode => home.join(".config/opencode/skill"),
            AgentType::Amp => home.join(".config/agents/skills"),
            AgentType::Roo => home.join(".roo/skills"),
            AgentType::Goose => home.join(".config/goose/skills"),
            AgentType::Kilo => home.join(".kilocode/skills"),
            AgentType::Antigravity => home.join(".gemini/antigravity/skills"),
            AgentType::GitHubCopilot => home.join(".copilot/skills"),
        }
    }

    pub fn all() -> &'static [AgentType] {
        &[
            AgentType::ClaudeCode,
            AgentType::Cursor,
            AgentType::Codex,
            AgentType::OpenCode,
            AgentType::Amp,
            AgentType::Roo,
            AgentType::Goose,
            AgentType::Kilo,
            AgentType::Antigravity,
            AgentType::GitHubCopilot,
        ]
    }
}

pub fn parse_agent_type(input: &str) -> anyhow::Result<AgentType> {
    let normalized = input.to_lowercase();

    match normalized.as_str() {
        "claude" | "claudecode" | "claude-code" => Ok(AgentType::ClaudeCode),
        "cursor" => Ok(AgentType::Cursor),
        "codex" => Ok(AgentType::Codex),
        "opencode" | "open" => Ok(AgentType::OpenCode),
        "amp" => Ok(AgentType::Amp),
        "roo" | "roocode" | "roo-code" => Ok(AgentType::Roo),
        "goose" => Ok(AgentType::Goose),
        "kilo" | "kilocode" | "kilo-code" => Ok(AgentType::Kilo),
        "antigravity" => Ok(AgentType::Antigravity),
        "github" | "githubcopilot" | "github-copilot" | "copilot" => Ok(AgentType::GitHubCopilot),
        _ => anyhow::bail!(
            "Unknown agent type '{}'. Valid agents: claude, cursor, codex, opencode, amp, roo, goose, kilo, antigravity, github-copilot",
            input
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display_names() {
        assert_eq!(AgentType::ClaudeCode.display_name(), "Claude Code");
        assert_eq!(AgentType::Cursor.display_name(), "Cursor");
        assert_eq!(AgentType::OpenCode.display_name(), "OpenCode");
    }

    #[test]
    fn test_agent_type_paths() {
        let agent = AgentType::ClaudeCode;
        assert_eq!(agent.project_dir(), ".claude/skills");
        assert!(agent.global_dir().ends_with(".claude/skills"));
    }

    #[test]
    fn test_parse_agent_type() {
        assert_eq!(parse_agent_type("claude").unwrap(), AgentType::ClaudeCode);
        assert_eq!(parse_agent_type("CURSOR").unwrap(), AgentType::Cursor);
        assert_eq!(
            parse_agent_type("claude-code").unwrap(),
            AgentType::ClaudeCode
        );
        assert!(parse_agent_type("unknown").is_err());
    }
}
