use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn uninstall_backup_dry_run_uses_cortex_home() {
    let home_dir = tempdir().unwrap();
    let cortex_home = home_dir.path().join("custom-cortex-home");
    fs::create_dir_all(&cortex_home).unwrap();
    fs::write(cortex_home.join("config.toml"), "model = \"test\"\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["uninstall", "--backup", "--dry-run"])
        .env("HOME", home_dir.path())
        .env("CORTEX_HOME", &cortex_home)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "uninstall dry-run failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains(&*cortex_home.join(".cortex-backup").to_string_lossy()),
        "stdout did not contain CORTEX_HOME backup path:\n{stdout}"
    );
}
