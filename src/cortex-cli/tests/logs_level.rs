use clap::Parser;
use cortex_cli::logs_cmd::LogsCli;

#[test]
fn accepts_valid_log_level_filter() {
    let cli = LogsCli::try_parse_from(["logs", "--level", "debug"])
        .expect("valid log level should be accepted");

    assert_eq!(cli.level.as_deref(), Some("debug"));
}

#[test]
fn rejects_invalid_log_level_filter() {
    let err = LogsCli::try_parse_from(["logs", "--level", "rr"])
        .expect_err("invalid log level should be rejected");

    assert!(
        err.to_string().contains("invalid log level"),
        "error should explain that the log level is invalid"
    );
}
