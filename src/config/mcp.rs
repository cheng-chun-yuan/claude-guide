use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl McpServer {
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PresetMcpServer {
    pub name: &'static str,
    pub description: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
}

impl PresetMcpServer {
    pub fn to_mcp_server(&self) -> McpServer {
        McpServer::new(self.command.to_string())
            .with_args(self.args.iter().map(|s| s.to_string()).collect())
    }
}

pub const PRESET_MCP_SERVERS: &[PresetMcpServer] = &[
    PresetMcpServer {
        name: "chrome-devtools",
        description: "Browser automation and DevTools access",
        command: "npx",
        args: &["chrome-devtools-mcp@latest"],
    },
    PresetMcpServer {
        name: "filesystem",
        description: "File system operations (current directory)",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-filesystem", "."],
    },
    PresetMcpServer {
        name: "github",
        description: "GitHub repository operations",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-github"],
    },
    PresetMcpServer {
        name: "slack",
        description: "Slack workspace integration",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-slack"],
    },
    PresetMcpServer {
        name: "postgres",
        description: "PostgreSQL database operations",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-postgres"],
    },
    PresetMcpServer {
        name: "sqlite",
        description: "SQLite database operations",
        command: "uvx",
        args: &["mcp-server-sqlite"],
    },
    PresetMcpServer {
        name: "puppeteer",
        description: "Browser automation with Puppeteer",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-puppeteer"],
    },
    PresetMcpServer {
        name: "brave-search",
        description: "Web search via Brave Search API",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-brave-search"],
    },
    PresetMcpServer {
        name: "memory",
        description: "Knowledge graph memory for persistent context",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-memory"],
    },
    PresetMcpServer {
        name: "fetch",
        description: "HTTP request operations",
        command: "uvx",
        args: &["mcp-server-fetch"],
    },
    PresetMcpServer {
        name: "sequential-thinking",
        description: "Dynamic problem-solving through thought sequences",
        command: "npx",
        args: &["-y", "@modelcontextprotocol/server-sequential-thinking"],
    },
    PresetMcpServer {
        name: "context7",
        description: "Up-to-date documentation for libraries",
        command: "npx",
        args: &["-y", "@context7/mcp@latest"],
    },
    PresetMcpServer {
        name: "tavily",
        description: "AI-powered web search",
        command: "npx",
        args: &["-y", "tavily-mcp@latest"],
    },
];

pub const ALLOWED_MCP_COMMANDS: &[&str] =
    &["npx", "uvx", "node", "python", "python3", "deno", "bun"];

pub fn validate_server_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn validate_command(command: &str) -> bool {
    ALLOWED_MCP_COMMANDS.contains(&command)
}
