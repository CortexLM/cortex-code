use std::process::Command;

use tempfile::tempdir;

#[test]
fn import_accepts_yaml_session_export() {
    let home_dir = tempdir().unwrap();
    let export_file = home_dir.path().join("session.yaml");
    std::fs::write(
        &export_file,
        r#"version: 1
session:
  id: "550e8400-e29b-41d4-a716-446655440000"
  title: "YAML import"
  created_at: "2024-01-01T00:00:00Z"
  cwd: "/tmp"
  model: "test-model"
messages:
  - role: "user"
    content: "hello from yaml"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["import", export_file.to_str().unwrap()])
        .env("HOME", home_dir.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        output.status.success(),
        "import failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(combined.contains("Imported session as:"));
    assert!(combined.contains("Messages: 1"));
}
