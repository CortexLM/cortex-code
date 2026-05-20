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
fn agent_create_non_interactive_writes_requested_model() {
    let home = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "agent",
            "create",
            "--name",
            "demo-agent",
            "--non-interactive",
            "--model",
            "claude-sonnet-4-20250514",
        ])
        .env("HOME", home.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "agent create failed:\n{}",
        combined_output(&output)
    );

    let agent_path = home.path().join(".cortex/agents/demo-agent.md");
    let content = fs::read_to_string(&agent_path).unwrap();
    assert!(content.contains("model: claude-sonnet-4-20250514"));
}

#[test]
fn agent_create_non_interactive_without_model_omits_model_override() {
    let home = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "agent",
            "create",
            "--name",
            "default-agent",
            "--non-interactive",
        ])
        .env("HOME", home.path())
        .env_remove("CORTEX_HOME")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "agent create failed:\n{}",
        combined_output(&output)
    );

    let agent_path = home.path().join(".cortex/agents/default-agent.md");
    let content = fs::read_to_string(&agent_path).unwrap();
    assert!(!content.contains("model:"));
}
