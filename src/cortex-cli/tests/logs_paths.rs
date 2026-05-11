use std::path::PathBuf;
use std::process::Command;

#[test]
fn logs_directory_matches_debug_paths_logs_dir() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let cortex_home = temp_dir.path().join("cortex-home");

    let debug_output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["debug", "paths", "--json"])
        .env("CORTEX_HOME", &cortex_home)
        .env("NO_COLOR", "1")
        .output()
        .expect("run Cortex debug paths");

    assert!(
        debug_output.status.success(),
        "debug paths failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&debug_output.stdout),
        String::from_utf8_lossy(&debug_output.stderr)
    );

    let paths: serde_json::Value =
        serde_json::from_slice(&debug_output.stdout).expect("debug paths should return JSON");
    let debug_logs_dir = paths["logs_dir"]["path"]
        .as_str()
        .expect("debug paths should include logs_dir.path");

    let logs_output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs"])
        .env("CORTEX_HOME", &cortex_home)
        .env("NO_COLOR", "1")
        .output()
        .expect("run Cortex logs");

    let logs_stdout = String::from_utf8_lossy(&logs_output.stdout);
    assert!(
        logs_output.status.success(),
        "logs failed\nstdout:{logs_stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&logs_output.stderr)
    );

    let logs_dir = logs_stdout
        .lines()
        .find_map(|line| line.strip_prefix("Logs directory: "))
        .expect("logs output should include its logs directory");

    assert_eq!(
        PathBuf::from(logs_dir),
        PathBuf::from(debug_logs_dir),
        "cortex logs should report the same logs directory as cortex debug paths"
    );
}
