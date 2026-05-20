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
fn logs_clear_level_removes_only_matching_level_files() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();

    let error_log = logs_dir.join("error.log");
    let debug_log = logs_dir.join("debug.log");
    let info_log = logs_dir.join("info.txt");

    fs::write(&error_log, "error\n").unwrap();
    fs::write(&debug_log, "debug\n").unwrap();
    fs::write(&info_log, "info\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--clear", "--level", "error", "--keep-days", "0"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "logs --clear --level failed:\n{}",
        combined_output(&output)
    );

    assert!(!error_log.exists(), "expected error log to be cleared");
    assert!(debug_log.exists(), "expected debug log to remain");
    assert!(info_log.exists(), "expected info log to remain");
}
