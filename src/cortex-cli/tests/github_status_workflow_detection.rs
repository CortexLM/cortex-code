use std::fs;
use std::process::Command;

#[test]
fn github_status_rejects_unrelated_workflow_that_mentions_cortex() {
    let repo = tempfile::tempdir().expect("create temp repo");
    fs::create_dir(repo.path().join(".git")).expect("create .git dir");
    let workflows_dir = repo.path().join(".github").join("workflows");
    fs::create_dir_all(&workflows_dir).expect("create workflows dir");
    fs::write(
        workflows_dir.join("homebrew.yml"),
        r#"name: Homebrew Release

on:
  push:
    tags:
      - "v*"

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - name: Publish unrelated package
        with:
          formula-name: Cortex
"#,
    )
    .expect("write unrelated workflow");

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["github", "status", "--json", "--path"])
        .arg(repo.path())
        .output()
        .expect("run Cortex github status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "unrelated workflow should not be treated as installed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let status: serde_json::Value =
        serde_json::from_str(&stdout).expect("status output should be JSON");
    assert_eq!(
        status["workflow_installed"],
        serde_json::Value::Bool(false),
        "status should reject unrelated workflow content\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
