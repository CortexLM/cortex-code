use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use tempfile::tempdir;

#[test]
fn logs_follow_prints_requested_initial_lines() {
    let home = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let logs_dir = cache.path().join("cortex").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    fs::write(logs_dir.join("cortex.log"), "one\ntwo\nthree\nfour\n").unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["logs", "--follow", "--lines", "2"])
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("CORTEX_HOME")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(250));
    let _ = child.kill();

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    let _ = child.wait();

    assert!(!stdout.contains("\none\n"), "output:\n{stdout}");
    assert!(!stdout.contains("\ntwo\n"), "output:\n{stdout}");
    assert!(stdout.contains("\nthree\n"), "output:\n{stdout}");
    assert!(stdout.contains("\nfour\n"), "output:\n{stdout}");
}
