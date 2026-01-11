use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;

/// Validate a name doesn't contain path traversal characters
/// Returns an error if the name is empty, contains "..", or contains path separators
pub fn validate_safe_name(name: &str, field_name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Invalid {}: name is empty", field_name));
    }
    if name == "." || name == ".." {
        return Err(anyhow!(
            "Invalid {}: '{}' is not allowed (path traversal)",
            field_name,
            name
        ));
    }
    if name.contains("..") {
        return Err(anyhow!(
            "Invalid {}: contains '..' (path traversal not allowed)",
            field_name
        ));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(anyhow!(
            "Invalid {}: contains path separator",
            field_name
        ));
    }
    Ok(())
}

/// Parse a GitHub URL into owner and repo name
/// Only accepts GitHub URLs - returns error for other hosts (GitLab, Bitbucket, etc.)
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
            let owner = parts[0].to_string();
            let repo = parts[1].to_string();

            // Validate owner and repo for safety
            validate_safe_name(&owner, "GitHub owner")?;
            validate_safe_name(&repo, "GitHub repository")?;

            return Ok((owner, repo));
        }
        return Err(anyhow!("Invalid GitHub SSH URL format: {}", url));
    }

    // Handle HTTPS format - must contain github.com
    let url_without_scheme = url.strip_prefix("https://").unwrap_or(url);
    let url_without_scheme = url_without_scheme.strip_prefix("http://").unwrap_or(url_without_scheme);

    // Validate this is actually a GitHub URL
    if !url_without_scheme.starts_with("github.com/") {
        return Err(anyhow!(
            "Not a GitHub URL: {}. Only github.com URLs are supported.",
            url
        ));
    }

    let path = url_without_scheme.strip_prefix("github.com/").unwrap();
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 {
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Validate owner and repo for safety
        validate_safe_name(&owner, "GitHub owner")?;
        validate_safe_name(&repo, "GitHub repository")?;

        Ok((owner, repo))
    } else {
        Err(anyhow!("Invalid GitHub URL format: {}", url))
    }
}

/// Clone a GitHub repository to a destination directory using git
/// Uses atomic replacement: clones to temp dir first, validates, then replaces
/// destination only on success. This prevents data loss if clone or validation fails.
///
/// The optional `validate` callback is called on the temp directory after cloning
/// but before replacing the destination. If validation fails, the temp directory
/// is cleaned up and the original destination is preserved.
pub fn clone_repo_validated<F>(url: &str, dest: &Path, validate: Option<F>) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    // Ensure parent directory exists
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow!("Invalid destination path: no parent directory"))?;
    std::fs::create_dir_all(parent)?;

    // Clone to a temporary directory first to avoid data loss on failure
    let temp_dest = parent.join(format!(
        ".{}.tmp.{}",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("clone"),
        std::process::id()
    ));

    // Clean up any leftover temp directory from previous failed attempts
    if temp_dest.exists() {
        std::fs::remove_dir_all(&temp_dest)?;
    }

    let output = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(&temp_dest)
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        // Clean up temp directory on failure
        let _ = std::fs::remove_dir_all(&temp_dest);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git clone failed: {}", stderr));
    }

    // Run validation on temp directory BEFORE replacing destination
    // This ensures we don't lose the original if validation fails
    if let Some(validate_fn) = validate {
        if let Err(e) = validate_fn(&temp_dest) {
            // Validation failed - clean up temp and preserve original
            let _ = std::fs::remove_dir_all(&temp_dest);
            return Err(e);
        }
    }

    // Clone and validation succeeded - now safely replace the destination
    // Remove old destination if it exists
    if dest.exists() {
        std::fs::remove_dir_all(dest)?;
    }

    // Move temp to final destination
    // Note: Do NOT delete temp_dest on failure - preserve it for manual recovery
    // since the original destination was already removed
    std::fs::rename(&temp_dest, dest).with_context(|| {
        format!(
            "Failed to move cloned repository to {}. Data preserved at: {}",
            dest.display(),
            temp_dest.display()
        )
    })?;

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

    #[test]
    fn test_parse_github_url_empty_repo_ssh() {
        // SSH URL with empty repo name should fail
        let result = parse_github_url("git@github.com:owner/");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is empty"));
    }

    #[test]
    fn test_parse_github_url_empty_owner_ssh() {
        // SSH URL with empty owner should fail
        let result = parse_github_url("git@github.com:/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is empty"));
    }

    #[test]
    fn test_parse_github_url_empty_repo_https() {
        // HTTPS URL with empty repo name should fail
        let result = parse_github_url("https://github.com/owner/");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is empty"));
    }

    #[test]
    fn test_parse_github_url_empty_owner_https() {
        // HTTPS URL with empty owner should fail
        let result = parse_github_url("https://github.com//repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is empty"));
    }

    #[test]
    fn test_parse_github_url_path_traversal_dotdot() {
        // URL with ".." should fail
        let result = parse_github_url("https://github.com/owner/..");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path traversal"));
    }

    #[test]
    fn test_parse_github_url_path_traversal_dot() {
        // URL with single "." should fail
        let result = parse_github_url("https://github.com/./repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path traversal"));
    }

    #[test]
    fn test_parse_github_url_rejects_non_github() {
        // GitLab URLs should be rejected
        let result = parse_github_url("https://gitlab.com/owner/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a GitHub URL"));

        // Bitbucket URLs should be rejected
        let result = parse_github_url("https://bitbucket.org/owner/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a GitHub URL"));

        // Generic git URLs should be rejected
        let result = parse_github_url("https://example.com/owner/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a GitHub URL"));
    }

    #[test]
    fn test_validate_safe_name() {
        // Valid names should pass
        assert!(validate_safe_name("valid-name", "test").is_ok());
        assert!(validate_safe_name("valid_name", "test").is_ok());
        assert!(validate_safe_name("valid.name", "test").is_ok());

        // Empty should fail
        assert!(validate_safe_name("", "test").is_err());

        // Path traversal should fail
        assert!(validate_safe_name("..", "test").is_err());
        assert!(validate_safe_name(".", "test").is_err());
        assert!(validate_safe_name("foo/..", "test").is_err());
        assert!(validate_safe_name("../foo", "test").is_err());

        // Path separators should fail
        assert!(validate_safe_name("foo/bar", "test").is_err());
        assert!(validate_safe_name("foo\\bar", "test").is_err());
    }
}
