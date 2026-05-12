use std::process::Command;

#[test]
fn whoami_cortex_api_key_names_environment_variable_source() {
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .arg("whoami")
        .env("CORTEX_API_KEY", "test-key-for-whoami")
        .env_remove("CORTEX_AUTH_TOKEN")
        .output()
        .expect("run Cortex whoami");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "whoami failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Authenticated via CORTEX_API_KEY environment variable:"),
        "expected environment variable source label\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("Authenticated via CORTEX_API_KEY:"),
        "stale auth-source wording should not be present\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
