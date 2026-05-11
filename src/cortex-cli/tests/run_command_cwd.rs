use std::path::PathBuf;

use cortex_cli::run_cmd::command_template_cwd;

#[test]
fn command_template_cwd_prefers_run_cwd_override() {
    let requested_cwd = PathBuf::from("requested-project");

    assert_eq!(
        command_template_cwd(Some(requested_cwd.as_path())),
        requested_cwd.to_string_lossy()
    );
}

#[test]
fn command_template_cwd_falls_back_to_process_cwd() {
    let current_dir = std::env::current_dir().expect("current dir should be available");

    assert_eq!(command_template_cwd(None), current_dir.to_string_lossy());
}
