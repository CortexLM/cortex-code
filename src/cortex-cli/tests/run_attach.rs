use std::process::Command;

#[test]
fn run_attach_rejects_filesystem_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "run",
            "--dry-run",
            "--attach",
            "/nonexistent/file.txt",
            "test",
        ])
        .output()
        .expect("cortex binary should run");

    assert!(
        !output.status.success(),
        "expected non-zero exit; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--attach expects an HTTP(S) server URL"),
        "stderr did not explain the invalid --attach value: {stderr}"
    );
}

#[test]
fn run_attach_allows_http_server_urls() {
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "run",
            "--dry-run",
            "--attach",
            "http://localhost:3000",
            "test",
        ])
        .output()
        .expect("cortex binary should run");

    assert!(
        output.status.success(),
        "expected zero exit; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Server attachment not yet fully implemented"));
}
