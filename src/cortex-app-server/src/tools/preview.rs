//! Helpers for building tool output previews.

/// Truncate a string to at most `max_bytes` bytes without splitting a UTF-8 character.
pub(super) fn truncate_at_char_boundary(input: &str, max_bytes: usize) -> &str {
    if input.len() <= max_bytes {
        return input;
    }

    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }

    &input[..end]
}

#[cfg(test)]
mod tests {
    use super::truncate_at_char_boundary;

    #[test]
    fn keeps_ascii_at_requested_byte_limit() {
        assert_eq!(truncate_at_char_boundary("abcdef", 3), "abc");
    }

    #[test]
    fn backs_up_to_utf8_boundary() {
        let input = format!("{}{}", "A".repeat(499), "\u{4E2D}");

        assert_eq!(input.len(), 502);
        assert_eq!(truncate_at_char_boundary(&input, 500), "A".repeat(499));
    }

    #[test]
    fn returns_full_input_when_under_limit() {
        assert_eq!(truncate_at_char_boundary("hello", 500), "hello");
    }
}
