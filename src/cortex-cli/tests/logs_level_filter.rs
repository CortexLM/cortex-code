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
fn logs_level_filter_matches_exact_log_level() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    fs::write(
        logs_dir.join("cortex.log"),
        [
            "2026-05-20T00:00:00Z INFO startup ok",
            "2026-05-20T00:00:01Z WARN contains ERROR in the message",
            "2026-05-20T00:00:02Z ERROR actual failure",
            "",
        ]
        .join("\n"),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--level", "error", "--lines", "10"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "logs --level failed:\n{}",
        combined_output(&output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ERROR actual failure"), "output:\n{stdout}");
    assert!(!stdout.contains("WARN contains ERROR"), "output:\n{stdout}");
    assert!(!stdout.contains("INFO startup"), "output:\n{stdout}");
}

#[test]
fn logs_level_filter_does_not_match_partial_level() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    fs::write(
        logs_dir.join("cortex.log"),
        "2026-05-20T00:00:00Z ERROR actual failure\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--level", "err", "--lines", "10"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "logs --level failed:\n{}",
        combined_output(&output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("ERROR actual failure"),
        "output:\n{stdout}"
    );
}
