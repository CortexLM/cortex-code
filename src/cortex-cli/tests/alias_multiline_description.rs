use std::path::Path;
use std::process::Command;

fn cortex_command(home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_Cortex"));
    command.env("HOME", home);
    command.env("USERPROFILE", home);
    command.env("APPDATA", home.join("AppData").join("Roaming"));
    command
}

fn run_success(command: &mut Command) -> String {
    let output = command.output().expect("run Cortex");
    assert!(
        output.status.success(),
        "command failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn alias_descriptions_indent_every_multiline_line() {
    let temp = tempfile::tempdir().expect("temp dir");
    let description = "First line\nSecond line";

    let set_output = run_success(
        cortex_command(temp.path())
            .args(["alias", "set", "bounty_alias_demo", "version", "-d"])
            .arg(description)
            .arg("-f"),
    );
    assert!(
        set_output.contains("  Description: First line\n               Second line"),
        "{set_output}"
    );

    let list_output = run_success(cortex_command(temp.path()).args(["alias", "list"]));
    assert!(
        list_output.contains("      First line\n      Second line"),
        "{list_output}"
    );

    let show_output =
        run_success(cortex_command(temp.path()).args(["alias", "show", "bounty_alias_demo"]));
    assert!(
        show_output.contains("  Description: First line\n               Second line"),
        "{show_output}"
    );
}
