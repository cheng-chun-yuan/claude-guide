# claude-guide

A TUI (Terminal User Interface) for managing Claude Code configurations.

## Features

- **Hooks**: View and manage hooks (PreToolUse, PostToolUse, Notification, Stop, SubagentStop, UserPromptSubmit)
- **Skills**: Browse and manage custom skills (`~/.claude/skills/`)
- **Plugins**: View and toggle plugins from marketplaces
- **Commands**: Manage custom slash commands (`~/.claude/commands/`)
- **Agents**: Configure custom agents (`~/.claude/agents/`)
- **MCP Servers**: Manage Model Context Protocol server configurations

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

```bash
claude-guide
```

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
