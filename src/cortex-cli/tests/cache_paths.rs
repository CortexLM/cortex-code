use std::process::Command;

use serde_json::Value;

fn cortex_json(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(args)
        .output()
        .expect("failed to run Cortex test binary");

    assert!(
        output.status.success(),
        "Cortex {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("Cortex command should print JSON")
}

#[test]
fn cache_show_json_uses_debug_paths_cache_dir() {
    let cache_show = cortex_json(&["cache", "show", "--json"]);
    let debug_paths = cortex_json(&["debug", "paths", "--json"]);

    assert_eq!(
        cache_show["cache_dir"].as_str(),
        debug_paths["cache_dir"]["path"].as_str()
    );
}
