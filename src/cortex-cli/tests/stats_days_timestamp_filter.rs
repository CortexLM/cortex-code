use serde_json::json;

#[path = "../src/stats_cmd.rs"]
mod stats_cmd;

fn write_session(
    sessions_dir: &std::path::Path,
    name: &str,
    timestamp: Option<&str>,
    input_tokens: u64,
) {
    let mut session = json!({
        "model": "gpt-4o",
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": 1
        }
    });

    if let Some(timestamp) = timestamp {
        session["timestamp"] = json!(timestamp);
    }

    std::fs::write(
        sessions_dir.join(name),
        serde_json::to_string_pretty(&session).expect("serialize session"),
    )
    .expect("write session");
}

#[tokio::test]
async fn stats_days_ignores_sessions_without_parseable_timestamps() {
    let temp = tempfile::tempdir().expect("temp dir");
    let sessions_dir = temp.path().join("sessions");
    std::fs::create_dir_all(&sessions_dir).expect("create sessions dir");

    let recent_timestamp = chrono::Utc::now().to_rfc3339();
    write_session(&sessions_dir, "recent.json", Some(&recent_timestamp), 10);
    write_session(&sessions_dir, "old.json", Some("2000-01-01T00:00:00Z"), 100);
    write_session(&sessions_dir, "invalid.json", Some("not-a-date"), 1_000);
    write_session(&sessions_dir, "missing.json", None, 10_000);

    let cli = stats_cmd::StatsCli {
        days: 1,
        provider: None,
        model: None,
        json: true,
        verbose: false,
    };
    let stats = stats_cmd::collect_stats(&sessions_dir, &cli)
        .await
        .expect("collect stats");

    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.input_tokens, 10);
    assert_eq!(stats.output_tokens, 1);
}
