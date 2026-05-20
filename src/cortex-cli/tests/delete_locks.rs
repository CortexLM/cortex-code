use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn cortex(home_dir: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_Cortex"));
    command.env("HOME", home_dir).env_remove("CORTEX_HOME");
    command
}

fn imported_session_id(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    let combined = format!("{stdout}{stderr}");
    let marker = "Imported session as:";
    let marker_index = combined
        .find(marker)
        .unwrap_or_else(|| panic!("import output should include session id:\n{combined}"));
    combined[marker_index + marker.len()..]
        .split_whitespace()
        .next()
        .expect("import output should include session id after marker")
        .to_string()
}

#[test]
fn delete_force_removes_matching_lock_entry() {
    let home_dir = tempdir().unwrap();
    let export_file = home_dir.path().join("session.json");
    fs::write(
        &export_file,
        r#"{
  "version": 1,
  "session": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Locked delete",
    "created_at": "2024-01-01T00:00:00Z",
    "cwd": "/tmp",
    "model": "test-model"
  },
  "messages": [
    { "role": "user", "content": "hello" }
  ]
}
"#,
    )
    .unwrap();

    let import_output = cortex(home_dir.path())
        .args(["import", export_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        import_output.status.success(),
        "import failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&import_output.stdout),
        String::from_utf8_lossy(&import_output.stderr)
    );
    let session_id = imported_session_id(&import_output.stdout, &import_output.stderr);

    let lock_output = cortex(home_dir.path())
        .args(["lock", "add", &session_id])
        .output()
        .unwrap();
    assert!(
        lock_output.status.success(),
        "lock add failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&lock_output.stdout),
        String::from_utf8_lossy(&lock_output.stderr)
    );

    let delete_output = cortex(home_dir.path())
        .args(["delete", &session_id, "--yes", "--force"])
        .output()
        .unwrap();
    assert!(
        delete_output.status.success(),
        "delete failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&delete_output.stdout),
        String::from_utf8_lossy(&delete_output.stderr)
    );

    let lock_list_output = cortex(home_dir.path())
        .args(["lock", "list", "--json"])
        .output()
        .unwrap();
    assert!(
        lock_list_output.status.success(),
        "lock list failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&lock_list_output.stdout),
        String::from_utf8_lossy(&lock_list_output.stderr)
    );
    let locks: Value = serde_json::from_slice(&lock_list_output.stdout).unwrap();
    assert_eq!(locks.as_array().unwrap().len(), 0);
}
