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
fn logs_clear_removes_only_log_files() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();

    let log_file = logs_dir.join("cortex.log");
    let txt_file = logs_dir.join("debug.txt");
    let json_file = logs_dir.join("notes.json");
    let bin_file = logs_dir.join("snapshot.bin");

    fs::write(&log_file, "log\n").unwrap();
    fs::write(&txt_file, "debug\n").unwrap();
    fs::write(&json_file, "{}\n").unwrap();
    fs::write(&bin_file, "binary\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--clear", "--keep-days", "0"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "logs --clear failed:\n{}",
        combined_output(&output)
    );

    assert!(!log_file.exists(), "expected .log file to be cleared");
    assert!(!txt_file.exists(), "expected .txt file to be cleared");
    assert!(json_file.exists(), "expected non-log .json file to remain");
    assert!(bin_file.exists(), "expected non-log .bin file to remain");
}
