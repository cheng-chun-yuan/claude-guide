use anyhow::Result;
use std::path::PathBuf;

/// Common CRUD operations for file-based resources (Skills, Agents, Commands)
pub struct ResourceManager;

impl ResourceManager {
    pub fn create(path: PathBuf, content: String) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn delete_file(path: PathBuf) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn delete_dir(path: PathBuf) -> Result<()> {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}
