use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread;

fn read_request(stream: &mut TcpStream) -> String {
    let mut buf = [0_u8; 4096];
    let mut request = Vec::new();

    loop {
        let n = stream.read(&mut buf).expect("read request");
        if n == 0 {
            break;
        }
        request.extend_from_slice(&buf[..n]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    String::from_utf8_lossy(&request).into_owned()
}

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("write response");
}

fn serve_once(listener: &TcpListener, base_url: &str) {
    let (mut stream, _) = listener.accept().expect("accept request");
    let request = read_request(&mut stream);
    let first_line = request.lines().next().unwrap_or_default();

    if first_line.starts_with("GET /v1/releases/latest?channel=stable ") {
        let body = format!(
            r#"{{
  "version": "0.0.8",
  "channel": "stable",
  "released_at": "2026-05-11T00:00:00Z",
  "changelog_url": "{base_url}/github.com/CortexLM/cortex-cli/blob/main/CHANGELOG.md",
  "release_notes": "test release",
  "assets": {{
    "windows-x86_64": {{"url": "{base_url}/download/windows.zip", "sha256": "abc", "size": 1}},
    "linux-x86_64": {{"url": "{base_url}/download/linux.tar.gz", "sha256": "abc", "size": 1}},
    "darwin-aarch64": {{"url": "{base_url}/download/darwin.tar.gz", "sha256": "abc", "size": 1}}
  }}
}}"#
        );
        write_response(&mut stream, "200 OK", "application/json", &body);
        return;
    }

    if first_line.starts_with("GET /raw.githubusercontent.com/CortexLM/cortex/main/CHANGELOG.md ") {
        write_response(
            &mut stream,
            "200 OK",
            "text/markdown",
            "# Changelog\n\n- fixed changelog repo URL\n",
        );
        return;
    }

    write_response(&mut stream, "404 Not Found", "text/plain", "not found");
}

#[test]
fn stale_cortex_cli_changelog_url_uses_cortex_repo() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    listener
        .set_nonblocking(false)
        .expect("configure test server");
    let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
    let server_url = base_url.clone();

    let server = thread::spawn(move || {
        serve_once(&listener, &server_url);
        serve_once(&listener, &server_url);
    });

    let home = tempfile::tempdir().expect("temp home");
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "upgrade",
            "--changelog",
            "--check",
            "--url",
            base_url.as_str(),
        ])
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .output()
        .expect("run Cortex upgrade");

    server.join().expect("server thread finished");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "upgrade command failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("fixed changelog repo URL"),
        "expected changelog content from corrected CortexLM/cortex URL\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("Failed to fetch changelog"),
        "changelog fetch should not fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
