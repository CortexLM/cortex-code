use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

fn serve_once(status_line: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("read local addr");

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0; 1024];
            let _ = stream.read(&mut request);
            let body = b"test";
            let response = format!(
                "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(body);
        }
    });

    format!("http://{addr}")
}

#[test]
fn debug_wait_server_ready_rejects_404() {
    let server_url = serve_once("HTTP/1.1 404 Not Found");

    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args([
            "debug",
            "wait",
            "--server-ready",
            "--server-url",
            &server_url,
            "--timeout",
            "1",
            "--interval",
            "100",
            "--json",
        ])
        .output()
        .expect("run Cortex debug wait");

    assert!(
        !output.status.success(),
        "404 readiness response should fail; stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"success\": false"),
        "stdout should report failure JSON: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
