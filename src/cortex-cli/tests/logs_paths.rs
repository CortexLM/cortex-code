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
fn logs_paths_lists_only_log_files() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();

    fs::write(logs_dir.join("cortex.log"), "log\n").unwrap();
    fs::write(logs_dir.join("debug.txt"), "debug\n").unwrap();
    fs::write(logs_dir.join("state.json"), "{}\n").unwrap();
    fs::write(logs_dir.join("session.lock"), "locked\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--paths"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "logs --paths failed:\n{}",
        combined_output(&output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cortex.log"), "output:\n{stdout}");
    assert!(stdout.contains("debug.txt"), "output:\n{stdout}");
    assert!(!stdout.contains("state.json"), "output:\n{stdout}");
    assert!(!stdout.contains("session.lock"), "output:\n{stdout}");
}
