//! Trusted workspace management.

use anyhow::Result;
use std::path::PathBuf;

// ============================================================================
// Trusted Workspaces
// ============================================================================

fn trusted_workspaces_dir() -> Option<PathBuf> {
    cortex_common::get_cortex_home()
}

fn trusted_workspaces_file() -> Option<PathBuf> {
    trusted_workspaces_dir().map(|dir| dir.join("trusted_workspaces.json"))
}

/// Check if a workspace is already trusted.
pub fn is_workspace_trusted(workspace: &std::path::Path) -> bool {
    let trusted_file = match trusted_workspaces_file() {
        Some(path) => path,
        None => return false,
    };

    if !trusted_file.exists() {
        return false;
    }

    let content = match std::fs::read_to_string(&trusted_file) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let workspace_str = workspace.to_string_lossy();
    if let Some(trusted) = data.get("trusted").and_then(|t| t.as_array()) {
        return trusted
            .iter()
            .filter_map(|v| v.as_str())
            .any(|path| path == workspace_str);
    }

    false
}

/// Mark a workspace as trusted.
pub fn mark_workspace_trusted(workspace: &std::path::Path) -> Result<()> {
    let cortex_dir = trusted_workspaces_dir().unwrap_or_else(|| PathBuf::from(".cortex"));

    std::fs::create_dir_all(&cortex_dir)?;

    let trusted_file = cortex_dir.join("trusted_workspaces.json");
    let mut trusted: Vec<String> = Vec::new();

    if trusted_file.exists()
        && let Ok(content) = std::fs::read_to_string(&trusted_file)
        && let Ok(data) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(arr) = data.get("trusted").and_then(|t| t.as_array())
    {
        trusted = arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }

    let workspace_str = workspace.to_string_lossy().to_string();
    if !trusted.contains(&workspace_str) {
        trusted.push(workspace_str);
    }

    let data = serde_json::json!({
        "version": 1,
        "trusted": trusted
    });

    std::fs::write(&trusted_file, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{OsStr, OsString};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &OsStr) -> Self {
            let previous = std::env::var_os(key);
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(previous) = &self.previous {
                    std::env::set_var(self.key, previous);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    #[test]
    fn trusted_workspaces_file_uses_cortex_home_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let _guard = EnvVarGuard::set("CORTEX_HOME", temp_dir.path().as_os_str());

        assert_eq!(
            trusted_workspaces_file().unwrap(),
            temp_dir.path().join("trusted_workspaces.json")
        );
    }

    #[test]
    fn is_workspace_trusted_reads_trust_store_from_cortex_home() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let _guard = EnvVarGuard::set("CORTEX_HOME", temp_dir.path().as_os_str());
        let workspace = temp_dir.path().join("project");
        let trusted_file = temp_dir.path().join("trusted_workspaces.json");

        std::fs::write(
            &trusted_file,
            serde_json::json!({
                "version": 1,
                "trusted": [workspace.to_string_lossy()]
            })
            .to_string(),
        )
        .unwrap();

        assert!(is_workspace_trusted(&workspace));
    }

    #[test]
    fn mark_workspace_trusted_writes_trust_store_to_cortex_home() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let _guard = EnvVarGuard::set("CORTEX_HOME", temp_dir.path().as_os_str());
        let workspace = temp_dir.path().join("project");
        let trusted_file = temp_dir.path().join("trusted_workspaces.json");

        mark_workspace_trusted(&workspace).unwrap();

        assert!(trusted_file.exists());
        assert!(is_workspace_trusted(&workspace));
    }
}
