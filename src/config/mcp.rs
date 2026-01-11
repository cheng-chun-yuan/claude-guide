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

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpServersConfig {
    #[serde(flatten)]
    pub servers: HashMap<String, McpServer>,
}

impl McpServersConfig {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, server: McpServer) {
        self.servers.insert(name, server);
    }

    pub fn remove(&mut self, name: &str) -> Option<McpServer> {
        self.servers.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut McpServer> {
        self.servers.get_mut(name)
    }

    pub fn list(&self) -> Vec<(&String, &McpServer)> {
        self.servers.iter().collect()
    }

    pub fn count(&self) -> usize {
        self.servers.len()
    }
}
