use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

impl Command {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            name,
            path: path.to_path_buf(),
            content,
        })
    }
}

pub fn scan_commands(commands_dir: &Path) -> Result<Vec<Command>> {
    let mut commands = Vec::new();

    if !commands_dir.exists() {
        return Ok(commands);
    }

    for entry in WalkDir::new(commands_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            match Command::load(path) {
                Ok(cmd) => commands.push(cmd),
                Err(e) => eprintln!(
                    "Warning: Failed to load command at {}: {}",
                    path.display(),
                    e
                ),
            }
        }
    }

    commands.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(commands)
}

pub fn create_command_template(name: &str) -> String {
    format!(
        r#"# {}

Description of what this command does.

## Usage

`/{}`

## Instructions

Add your command instructions here.
"#,
        name, name
    )
}
