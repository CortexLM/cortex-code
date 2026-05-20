use std::process::Command;

use tempfile::tempdir;

#[test]
fn plugin_remove_rejects_absolute_path() {
    let home_dir = tempdir().unwrap();
    let outside_dir = tempdir().unwrap();
    let target = outside_dir.path().join("srcplugin");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("plugin.toml"), "name = \"demo\"\n").unwrap();
    std::fs::write(target.join("marker.txt"), "keep\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["plugin", "remove", "-y", target.to_str().unwrap()])
        .env("HOME", home_dir.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("Invalid plugin name"),
        "unexpected stderr: {stderr}"
    );
    assert!(target.exists(), "absolute plugin path was deleted");
    assert!(target.join("marker.txt").exists());
}
