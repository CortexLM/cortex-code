use std::process::Command;

#[test]
fn root_help_documents_canonical_cortex_home_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .arg("--help")
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run Cortex --help");

    assert!(
        output.status.success(),
        "Cortex --help failed: stdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("default: ~/.cortex"),
        "CORTEX_HOME help should document the canonical ~/.cortex default:\n{stdout}"
    );
    assert!(
        stdout.contains("~/.cortex/config.toml"),
        "Config path should use canonical Cortex home:\n{stdout}"
    );
    assert!(
        stdout.contains("~/.cortex/sessions/"),
        "Sessions path should use canonical Cortex home:\n{stdout}"
    );
    assert!(
        stdout.contains("~/.cortex/logs/"),
        "Logs path should use canonical Cortex home:\n{stdout}"
    );
    assert!(
        !stdout.contains("~/.config/cortex"),
        "Help should not advertise lowercase XDG config paths:\n{stdout}"
    );
    assert!(
        !stdout.contains("~/.local/share/cortex"),
        "Help should not advertise lowercase XDG data paths:\n{stdout}"
    );
    assert!(
        !stdout.contains("~/.cache/cortex"),
        "Help should not advertise lowercase XDG cache paths:\n{stdout}"
    );
}
