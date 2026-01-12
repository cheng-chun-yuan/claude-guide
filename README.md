# claude-guide

A TUI (Terminal User Interface) and CLI for managing Claude Code configurations, skills, plugins, and marketplaces.

## Features

- **Hooks**: View and manage hooks (PreToolUse, PostToolUse, Notification, Stop, SubagentStop, UserPromptSubmit)
- **Skills**: Browse, install, and manage custom skills (`~/.claude/skills/`)
- **Plugins**: View, install, and toggle plugins from marketplaces
- **Commands**: Manage custom slash commands (`~/.claude/commands/`)
- **Agents**: Configure custom agents (`~/.claude/agents/`)
- **MCP Servers**: Manage Model Context Protocol server configurations
- **Marketplaces**: Add and manage plugin marketplaces

## Installation

```bash
cargo install --git https://github.com/cheng-chun-yuan/claude-guide
```

Or build from source:

```bash
git clone https://github.com/cheng-chun-yuan/claude-guide
cd claude-guide
cargo install --path .
```

## Usage

### TUI Mode

Launch the interactive terminal UI:

```bash
claude-guide
```

### CLI Commands

#### Install Skills from GitHub

```bash
# Install a skill from GitHub URL
claude-guide install skill https://github.com/user/skill-repo

# List installed skills
claude-guide list skills

# Remove a skill
claude-guide remove skill <name>
```

#### Install Plugins

```bash
# Install plugin from GitHub URL
claude-guide install plugin https://github.com/user/plugin-repo

# Install plugin from marketplace
claude-guide plugin install <plugin-name>

# Search plugins in marketplaces
claude-guide plugin search <query>

# List installed plugins
claude-guide list plugins
```

#### Manage Marketplaces

```bash
# Add a marketplace
claude-guide marketplace add https://github.com/user/marketplace-repo

# List registered marketplaces
claude-guide marketplace list

# Update all marketplaces
claude-guide marketplace update

# Update specific marketplace
claude-guide marketplace update <name>

# Remove a marketplace
claude-guide marketplace remove <name>
```

## Popular Skills & Resources

### Skills

| Skill | Description | Install |
|-------|-------------|---------|
| [notebooklm-skill](https://github.com/PleasePrompto/notebooklm-skill) | Query Google NotebookLM notebooks directly from Claude Code for source-grounded, citation-backed answers | `claude-guide install skill https://github.com/PleasePrompto/notebooklm-skill` |

### MCP Servers

Popular MCP (Model Context Protocol) servers to enhance Claude Code:

| MCP Server | Description | Link |
|------------|-------------|------|
| [Context7](https://github.com/upstash/context7) | Up-to-date documentation for any library | [GitHub](https://github.com/upstash/context7) |
| [Tavily](https://github.com/tavily-ai/tavily-mcp) | AI-powered web search | [GitHub](https://github.com/tavily-ai/tavily-mcp) |
| [Chrome DevTools](https://github.com/anthropics/anthropic-quickstarts/tree/main/mcp-chrome-devtools) | Browser automation and debugging | [GitHub](https://github.com/anthropics/anthropic-quickstarts/tree/main/mcp-chrome-devtools) |
| [Filesystem](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) | File system operations | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) |

### Marketplaces

| Marketplace | Description | Add |
|-------------|-------------|-----|
| [claude-plugins-official](https://github.com/anthropics/claude-plugins-official) | Official Anthropic plugins | `claude-guide marketplace add https://github.com/anthropics/claude-plugins-official` |

## Keybindings

| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `h/l` | Switch focus left/right |
| `Tab` | Next tab |
| `1-6` | Jump to tab |
| `a` | Add new item |
| `e` | Edit selected item |
| `d` | Delete selected item |
| `Space` | Toggle (plugins) |
| `r` | Refresh |
| `Ctrl+S` | Save settings |
| `?` | Show help |
| `q` | Quit |

## Configuration Files

| Feature | Location | Format |
|---------|----------|--------|
| Hooks | `~/.claude/settings.json` | JSON |
| Plugins | `~/.claude/settings.json` + `~/.claude/plugins/` | JSON |
| MCP Servers | `~/.claude/settings.json` | JSON |
| Skills | `~/.claude/skills/*/SKILL.md` | YAML frontmatter + Markdown |
| Commands | `~/.claude/commands/*.md` | Markdown |
| Agents | `~/.claude/agents/*.md` | YAML frontmatter + Markdown |

## License

MIT
