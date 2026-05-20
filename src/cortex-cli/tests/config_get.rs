use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn config_get_resolves_dotted_keys() {
    let home_dir = tempdir().unwrap();
    let cortex_home = home_dir.path().join(".cortex");
    fs::create_dir_all(&cortex_home).unwrap();
    fs::write(
        cortex_home.join("config.toml"),
        r#"[model]
default = "gpt-4o"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["config", "get", "model.default"])
        .env("HOME", home_dir.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "config get failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("gpt-4o"), "stdout was {stdout:?}");
}
