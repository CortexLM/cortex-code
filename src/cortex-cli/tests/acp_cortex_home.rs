use std::process::Command;

use tempfile::tempdir;

#[test]
fn acp_agent_lookup_uses_cortex_home() {
    let temp_dir = tempdir().unwrap();
    let agent_dir = temp_dir.path().join("agents").join("testacpagent");
    std::fs::create_dir_all(&agent_dir).unwrap();
    std::fs::write(
        agent_dir.join("AGENT.md"),
        r#"---
name: testacpagent
description: Test ACP agent
---

Test prompt
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "acp",
            "--agent",
            "testacpagent",
            "--port",
            "1",
            "--host",
            "not a socket address",
        ])
        .env("CORTEX_HOME", temp_dir.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        !stderr.contains("Agent not found"),
        "ACP ignored CORTEX_HOME and failed agent lookup: {stderr}"
    );
}
