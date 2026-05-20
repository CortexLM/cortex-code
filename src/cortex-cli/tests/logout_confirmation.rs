use std::io::{self, BufRead, Cursor, Read};

use cortex_cli::login::read_logout_confirmation;

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("stdin unavailable"))
    }
}

impl BufRead for FailingReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Err(io::Error::other("stdin unavailable"))
    }

    fn consume(&mut self, _amt: usize) {}
}

#[test]
fn logout_confirmation_accepts_yes_values() {
    let mut lowercase = Cursor::new(b"yes\n");
    assert!(read_logout_confirmation(&mut lowercase).unwrap());

    let mut uppercase = Cursor::new(b"Y\n");
    assert!(read_logout_confirmation(&mut uppercase).unwrap());
}

#[test]
fn logout_confirmation_rejects_empty_or_negative_values() {
    let mut empty = Cursor::new(b"\n");
    assert!(!read_logout_confirmation(&mut empty).unwrap());

    let mut no = Cursor::new(b"no\n");
    assert!(!read_logout_confirmation(&mut no).unwrap());
}

#[test]
fn logout_confirmation_propagates_read_errors() {
    let mut reader = FailingReader;
    let error = read_logout_confirmation(&mut reader).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
}
