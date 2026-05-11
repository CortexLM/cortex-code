use std::process::Command;

#[test]
fn agent_list_single_wildcard_filter_does_not_panic() {
    let home = tempfile::tempdir().expect("temp home");
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["agent", "list", "--filter", "*", "--json"])
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .output()
        .expect("run Cortex agent list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "agent list should accept a single wildcard filter\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("agent list --json should emit valid JSON");
    assert!(
        !stderr.contains("panicked"),
        "single wildcard filter should not panic\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
