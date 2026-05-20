use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn workspace_show_json_detects_config_json() {
    let workspace = tempdir().unwrap();
    let home_dir = tempdir().unwrap();
    let cortex_dir = workspace.path().join(".cortex");
    fs::create_dir_all(&cortex_dir).unwrap();
    fs::write(cortex_dir.join("config.json"), r#"{"model":"gpt-4o"}"#).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["workspace", "show", "--json"])
        .current_dir(workspace.path())
        .env("HOME", home_dir.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "workspace show failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["has_cortex_config"], true);
    assert!(
        json["config_path"]
            .as_str()
            .unwrap()
            .ends_with(".cortex/config.json"),
        "config_path was {:?}",
        json["config_path"]
    );
}
