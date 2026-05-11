use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn stats_aggregates_model_names_case_insensitively() {
    let sessions_dir = home_dir().join(".cortex").join("sessions");
    fs::create_dir_all(&sessions_dir).expect("sessions directory should be created");

    let unique = format!("codex-case-model-{}", std::process::id());
    let lower_model = unique.to_lowercase();
    let upper_model = lower_model.to_uppercase();
    let lower_path = sessions_dir.join(format!("{lower_model}-lower.json"));
    let upper_path = sessions_dir.join(format!("{lower_model}-upper.json"));

    fs::write(
        &lower_path,
        serde_json::json!({
            "model": lower_model,
            "messages": [{"role": "user", "content": "a"}],
            "usage": {"input_tokens": 100, "output_tokens": 100}
        })
        .to_string(),
    )
    .expect("lowercase session should be written");

    fs::write(
        &upper_path,
        serde_json::json!({
            "model": upper_model,
            "messages": [{"role": "user", "content": "b"}],
            "usage": {"input_tokens": 200, "output_tokens": 200}
        })
        .to_string(),
    )
    .expect("uppercase session should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["stats", "--days", "3650", "--json"])
        .output()
        .expect("cortex binary should run");

    let _ = fs::remove_file(&lower_path);
    let _ = fs::remove_file(&upper_path);

    assert!(
        output.status.success(),
        "expected zero exit; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stats output should be JSON");
    let by_model = json["by_model"]
        .as_object()
        .expect("by_model should be an object");

    assert!(
        by_model.contains_key(&lower_model),
        "expected lowercase model key in by_model: {by_model:?}"
    );
    assert!(
        !by_model.contains_key(&upper_model),
        "uppercase model key should be folded into lowercase key: {by_model:?}"
    );
    assert_eq!(by_model[&lower_model]["sessions"], 2);
    assert_eq!(by_model[&lower_model]["input_tokens"], 300);
    assert_eq!(by_model[&lower_model]["output_tokens"], 300);
}

fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .expect("home directory environment should be available")
}
