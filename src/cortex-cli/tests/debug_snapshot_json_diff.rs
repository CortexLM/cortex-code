use std::process::Command;

#[test]
fn debug_snapshot_json_includes_diff_when_requested() {
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["debug", "snapshot", "--json", "--diff"])
        .env("NO_COLOR", "1")
        .output()
        .expect("run Cortex debug snapshot");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "debug snapshot failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("debug snapshot output should be JSON");
    assert_eq!(json["diff"]["status"], "not_implemented");
    assert!(
        json["diff"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("not yet implemented")),
        "diff message should explain snapshot diff status\nstdout:\n{stdout}"
    );
}
