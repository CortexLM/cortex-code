//! Utility functions for the cortex-login module.

use anyhow::{Context, Result};
use std::path::Path;

/// Mask an API key for safe display.
pub fn safe_format_key(key: &str) -> String {
    let char_count = key.chars().count();
    if char_count <= 13 {
        return "***".to_string();
    }

    let prefix: String = key.chars().take(8).collect();
    let suffix: String = key.chars().skip(char_count - 5).collect();
    format!("{prefix}***{suffix}")
}

/// Set restrictive file permissions (0600 on Unix).
pub fn set_file_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to set permissions on {}", path.display()))?;
    }

    #[cfg(not(unix))]
    {
        let _ = path; // Suppress unused warning on Windows
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_format_key_long() {
        let key = "sk-proj-1234567890ABCDE";
        assert_eq!(safe_format_key(key), "sk-proj-***ABCDE");
    }

    #[test]
    fn test_safe_format_key_short() {
        let key = "sk-proj-12345";
        assert_eq!(safe_format_key(key), "***");
    }

    #[test]
    fn test_safe_format_key_non_ascii_boundary() {
        let key = "aaaaaaaéaaaaa";
        assert_eq!(safe_format_key(key), "***");
    }

    #[test]
    fn test_safe_format_key_long_non_ascii() {
        let key = "sk-proj-ééééééé-token";
        assert_eq!(safe_format_key(key), "sk-proj-***token");
    }
}
