# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-01-19

### Added
- Version control system for skills
- Profile management (create, export, delete profiles)
- Export skills to local project folders for different agents (Claude, OpenCode, Cursor, etc.)
- `version` CLI subcommand
- GitHub Actions CI/CD workflow
- TUI: Platform switcher (press `p`) to view/manage configs for different agents
- TUI: Skill version management (press `v`) to switch between installed versions

### Changed
- Refactored codebase to remove unused modules and dependencies
- Improved error handling and CLI output
- Updated Cargo.toml metadata for production release

### Removed
- Unused catalog, security, and store modules
- Unused installer functionality

## [0.1.0] - 2024-01-10

### Added
- Initial release
- TUI for managing Claude Code configurations
- Skill installation from GitHub
- Plugin management
- Marketplace support
- Hook configuration
- MCP server management
