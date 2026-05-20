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
fn logs_paths_accepts_default_lines() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    fs::write(logs_dir.join("cortex.log"), "log\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--paths"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "logs --paths should accept default --lines:\n{}",
        combined_output(&output)
    );
}

#[test]
fn logs_paths_rejects_viewing_flags() {
    for args in [
        ["logs", "--paths", "--follow"].as_slice(),
        ["logs", "--paths", "--level", "error"].as_slice(),
        ["logs", "--paths", "--lines", "5"].as_slice(),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
            .args(args)
            .output()
            .unwrap();

        assert!(!output.status.success(), "expected {:?} to fail", args);
        let output = combined_output(&output);
        assert!(
            output.contains("cannot be used with") || output.contains("conflict"),
            "expected conflict error for {:?}, got:\n{}",
            args,
            output
        );
    }
}
