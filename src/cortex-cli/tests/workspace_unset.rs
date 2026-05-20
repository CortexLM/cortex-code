use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn combined_output(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn workspace_unset_removes_keys_from_workspace_config() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    let cortex_dir = repo.path().join(".cortex");
    fs::create_dir_all(&cortex_dir).unwrap();
    let config_path = cortex_dir.join("config.toml");
    fs::write(
        &config_path,
        r#"remove_me = "gone"
keep_me = "stays"

[model]
default = "gpt-4o"

[sandbox]
mode = "workspace-write"
"#,
    )
    .unwrap();

    let model_output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["workspace", "unset", "model"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(
        model_output.status.success(),
        "workspace unset model failed:\n{}",
        combined_output(&model_output)
    );

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(!content.contains("[model]"));
    assert!(!content.contains("default ="));
    assert!(content.contains("keep_me = \"stays\""));
    assert!(content.contains("[sandbox]"));

    let top_level_output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["workspace", "unset", "remove_me"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(
        top_level_output.status.success(),
        "workspace unset remove_me failed:\n{}",
        combined_output(&top_level_output)
    );

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(!content.contains("remove_me"));
    assert!(content.contains("keep_me = \"stays\""));
}

#[test]
fn workspace_unset_reports_missing_key() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    let cortex_dir = repo.path().join(".cortex");
    fs::create_dir_all(&cortex_dir).unwrap();
    fs::write(cortex_dir.join("config.toml"), "keep_me = \"stays\"\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["workspace", "unset", "missing"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "workspace unset missing unexpectedly succeeded:\n{}",
        combined_output(&output)
    );
    assert!(combined_output(&output).contains("Key 'missing' not found"));
}
