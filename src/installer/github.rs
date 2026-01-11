use anyhow::{anyhow, Context, Result};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// Parse a GitHub URL into owner and repo name
pub fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Handle various GitHub URL formats:
    // https://github.com/owner/repo
    // https://github.com/owner/repo.git
    // github.com/owner/repo
    // git@github.com:owner/repo.git

    let url = url.trim();
    let url = url.strip_suffix(".git").unwrap_or(url);

    // Handle SSH format
    if url.starts_with("git@github.com:") {
        let path = url.strip_prefix("git@github.com:").unwrap();
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Handle HTTPS format
    let url = url.strip_prefix("https://").unwrap_or(url);
    let url = url.strip_prefix("http://").unwrap_or(url);
    let url = url.strip_prefix("github.com/").unwrap_or(url);

    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 2 {
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Validate owner and repo are not empty to prevent directory traversal issues
        if owner.is_empty() {
            return Err(anyhow!("Invalid GitHub URL: owner is empty"));
        }
        if repo.is_empty() {
            return Err(anyhow!("Invalid GitHub URL: repository name is empty"));
        }

        Ok((owner, repo))
    } else {
        Err(anyhow!("Invalid GitHub URL format: {}", url))
    }
}

/// Clone a GitHub repository to a destination directory using git
pub fn clone_repo(url: &str, dest: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove destination if it exists
    if dest.exists() {
        std::fs::remove_dir_all(dest)?;
    }

    let output = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(dest)
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git clone failed: {}", stderr));
    }

    Ok(())
}

/// Download a GitHub repository as a zip file and extract it
pub fn download_and_extract(owner: &str, repo: &str, dest: &Path) -> Result<()> {
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/main.zip",
        owner, repo
    );

    // Try main branch first, then master
    let response = reqwest::blocking::get(&url).or_else(|_| {
        let master_url = format!(
            "https://github.com/{}/{}/archive/refs/heads/master.zip",
            owner, repo
        );
        reqwest::blocking::get(&master_url)
    })?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download repository: HTTP {}",
            response.status()
        ));
    }

    // Create temp file
    let temp_dir = tempfile::tempdir()?;
    let zip_path = temp_dir.path().join("repo.zip");

    // Write zip to temp file
    let bytes = response.bytes()?;
    let mut file = std::fs::File::create(&zip_path)?;
    file.write_all(&bytes)?;

    // Extract zip
    let file = std::fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Find the root directory name in the zip (usually repo-main or repo-master)
    let root_dir = archive
        .file_names()
        .next()
        .and_then(|n| n.split('/').next())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Empty zip archive"))?;

    // Ensure destination exists
    std::fs::create_dir_all(dest)?;

    // Extract files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => {
                // Strip the root directory from the path
                let path_str = path.to_string_lossy();
                if let Some(stripped) = path_str.strip_prefix(&format!("{}/", root_dir)) {
                    if stripped.is_empty() {
                        continue;
                    }
                    dest.join(stripped)
                } else {
                    continue;
                }
            }
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Pull latest changes for an existing git repository
pub fn pull_repo(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git pull failed: {}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let cases = vec![
            ("https://github.com/owner/repo", ("owner", "repo")),
            ("https://github.com/owner/repo.git", ("owner", "repo")),
            ("github.com/owner/repo", ("owner", "repo")),
            ("git@github.com:owner/repo.git", ("owner", "repo")),
        ];

        for (url, (expected_owner, expected_repo)) in cases {
            let (owner, repo) = parse_github_url(url).unwrap();
            assert_eq!(owner, expected_owner);
            assert_eq!(repo, expected_repo);
        }
    }
}
