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
| [notebooklm-skill](https://github.com/PleasePrompto/notebooklm-skill) | Query Google NotebookLM notebooks directly from Claude Code for source-grounded, citation-backed answers from Gemini. Features browser automation, library management, and persistent auth. | `claude-guide install skill https://github.com/PleasePrompto/notebooklm-skill` |
| [ralph-loop](https://github.com/anthropics/claude-plugins-official/tree/main/plugins/ralph-loop) | Autonomous AI development loops using the Ralph Wiggum technique. Creates continuous feedback loops where Claude iteratively improves work until completion. Use `/ralph-loop "task" --max-iterations 20` | Install via marketplace (see below) |

### Official Anthropic Skills

The [anthropics/skills](https://github.com/anthropics/skills) repository contains official skills for document manipulation:

| Skill | Description |
|-------|-------------|
| **document-skills:pdf** | PDF manipulation - extract text/tables, create PDFs, merge/split documents, fill forms |
| **document-skills:docx** | Word documents - create, edit, track changes, comments, formatting preservation |
| **document-skills:pptx** | PowerPoint - create, edit presentations with layouts, templates, charts |
| **document-skills:xlsx** | Excel spreadsheets - formulas, formatting, data analysis, visualization |
| **document-skills:frontend-design** | Create distinctive, production-grade frontend interfaces with high design quality |
| **document-skills:brand-guidelines** | Apply official brand colors and typography to artifacts |

To use official skills, add the marketplace first:
```bash
claude-guide marketplace add https://github.com/anthropics/skills
```

### Official Anthropic Plugins

From [claude-plugins-official](https://github.com/anthropics/claude-plugins-official):

| Plugin | Description |
|--------|-------------|
| **ralph-wiggum** | Autonomous iteration loops (`/ralph-loop`, `/cancel-ralph`) |
| **commit-commands** | Git workflow automation (`/commit`, `/commit-push-pr`) |
| **code-review** | Automated PR review with 5 specialized agents |
| **feature-dev** | Structured 7-phase feature development |
| **plugin-dev** | Toolkit for developing Claude Code plugins |
| **security-guidance** | Security warnings for potential vulnerabilities |
| **hookify** | Create custom hooks to prevent unwanted behaviors |

### MCP Servers

Popular MCP (Model Context Protocol) servers to enhance Claude Code:

| MCP Server | Description | Link |
|------------|-------------|------|
| [Context7](https://github.com/upstash/context7) | Up-to-date documentation for any library | [GitHub](https://github.com/upstash/context7) |
| [Tavily](https://github.com/tavily-ai/tavily-mcp) | AI-powered web search | [GitHub](https://github.com/tavily-ai/tavily-mcp) |
| [Chrome DevTools](https://github.com/anthropics/anthropic-quickstarts/tree/main/mcp-chrome-devtools) | Browser automation and debugging | [GitHub](https://github.com/anthropics/anthropic-quickstarts/tree/main/mcp-chrome-devtools) |
| [Filesystem](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) | File system operations | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) |

### Community Resources

| Resource | Description | Link |
|----------|-------------|------|
| [awesome-claude-skills](https://github.com/travisvn/awesome-claude-skills) | Curated list of Claude Skills and resources | [GitHub](https://github.com/travisvn/awesome-claude-skills) |
| [claude-skills-collection](https://github.com/abubakarsiddik31/claude-skills-collection) | Collection of official and community-built skills | [GitHub](https://github.com/abubakarsiddik31/claude-skills-collection) |

### Marketplaces

| Marketplace | Description | Add |
|-------------|-------------|-----|
| [claude-plugins-official](https://github.com/anthropics/claude-plugins-official) | Official Anthropic plugins (ralph-loop, commit-commands, etc.) | `claude-guide marketplace add https://github.com/anthropics/claude-plugins-official` |
| [anthropics/skills](https://github.com/anthropics/skills) | Official document skills (pdf, docx, pptx, xlsx) | `claude-guide marketplace add https://github.com/anthropics/skills` |

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
