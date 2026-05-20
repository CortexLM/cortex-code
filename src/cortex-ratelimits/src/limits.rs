//! Rate limit types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Rate limit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Requests per minute limit.
    pub requests_per_minute: Option<u32>,
    /// Requests remaining this minute.
    pub requests_remaining: Option<u32>,
    /// Tokens per minute limit.
    pub tokens_per_minute: Option<u32>,
    /// Tokens remaining this minute.
    pub tokens_remaining: Option<u32>,
    /// Tokens per day limit.
    pub tokens_per_day: Option<u64>,
    /// Tokens used today.
    pub tokens_used_today: Option<u64>,
    /// When limits reset.
    pub reset_at: Option<DateTime<Utc>>,
    /// Current window.
    pub window: RateLimitWindow,
}

impl Default for RateLimitInfo {
    fn default() -> Self {
        Self {
            requests_per_minute: None,
            requests_remaining: None,
            tokens_per_minute: None,
            tokens_remaining: None,
            tokens_per_day: None,
            tokens_used_today: None,
            reset_at: None,
            window: RateLimitWindow::Minute,
        }
    }
}

impl RateLimitInfo {
    /// Check if approaching rate limit.
    pub fn is_approaching_limit(&self) -> bool {
        if let (Some(remaining), Some(limit)) = (self.requests_remaining, self.requests_per_minute)
        {
            if remaining < limit / 10 {
                return true;
            }
        }
        if let (Some(remaining), Some(limit)) = (self.tokens_remaining, self.tokens_per_minute) {
            if remaining < limit / 10 {
                return true;
            }
        }
        false
    }

    /// Check if rate limited.
    pub fn is_rate_limited(&self) -> bool {
        self.requests_remaining == Some(0) || self.tokens_remaining == Some(0)
    }

    /// Get usage percentage for requests.
    pub fn request_usage_percent(&self) -> Option<f32> {
        match (self.requests_remaining, self.requests_per_minute) {
            (Some(remaining), Some(limit)) if limit > 0 => {
                Some(100.0 * (1.0 - remaining as f32 / limit as f32))
            }
            _ => None,
        }
    }

    /// Get usage percentage for tokens.
    pub fn token_usage_percent(&self) -> Option<f32> {
        match (self.tokens_remaining, self.tokens_per_minute) {
            (Some(remaining), Some(limit)) if limit > 0 => {
                Some(100.0 * (1.0 - remaining as f32 / limit as f32))
            }
            _ => None,
        }
    }
}

/// Rate limit window type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitWindow {
    Second,
    Minute,
    Hour,
    Day,
}

impl std::fmt::Display for RateLimitWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Second => write!(f, "sec"),
            Self::Minute => write!(f, "min"),
            Self::Hour => write!(f, "hr"),
            Self::Day => write!(f, "day"),
        }
    }
}

/// Usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total requests made.
    pub total_requests: u64,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Total output tokens.
    pub total_output_tokens: u64,
    /// Total cached tokens.
    pub total_cached_tokens: u64,
    /// Session start time.
    pub session_start: Option<DateTime<Utc>>,
}

impl UsageStats {
    pub fn new() -> Self {
        Self {
            session_start: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Add usage from a request.
    pub fn add_request(&mut self, input_tokens: u64, output_tokens: u64, cached_tokens: u64) {
        self.total_requests += 1;
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
        self.total_cached_tokens += cached_tokens;
    }

    /// Get total tokens.
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }
}

/// Parse rate limit headers from HTTP response.
pub fn parse_rate_limit_headers(
    headers: &std::collections::HashMap<String, String>,
) -> RateLimitInfo {
    let mut info = RateLimitInfo::default();

    if let Some(limit) = headers
        .get("x-ratelimit-limit-requests")
        .and_then(|v| v.parse().ok())
    {
        info.requests_per_minute = Some(limit);
    }

    if let Some(remaining) = headers
        .get("x-ratelimit-remaining-requests")
        .and_then(|v| v.parse().ok())
    {
        info.requests_remaining = Some(remaining);
    }

    if let Some(limit) = headers
        .get("x-ratelimit-limit-tokens")
        .and_then(|v| v.parse().ok())
    {
        info.tokens_per_minute = Some(limit);
    }

    if let Some(remaining) = headers
        .get("x-ratelimit-remaining-tokens")
        .and_then(|v| v.parse().ok())
    {
        info.tokens_remaining = Some(remaining);
    }

    if let Some(reset) = headers
        .get("x-ratelimit-reset-requests")
        .or_else(|| headers.get("x-ratelimit-reset-tokens"))
    {
        info.reset_at = parse_reset_header(reset, Utc::now());
    }

    info
}

fn parse_reset_header(value: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    if let Ok(timestamp) = value.parse::<i64>() {
        return if timestamp > 1_000_000_000 {
            DateTime::from_timestamp(timestamp, 0)
        } else {
            Some(now + Duration::seconds(timestamp))
        };
    }

    if let Ok(timestamp) = value.parse::<DateTime<Utc>>() {
        return Some(timestamp);
    }

    parse_duration_seconds(value).map(|seconds| now + Duration::seconds(seconds))
}

fn parse_duration_seconds(value: &str) -> Option<i64> {
    let mut seconds = 0_i64;
    let mut digits = String::new();
    let mut saw_unit = false;

    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }

        let amount = digits.parse::<i64>().ok()?;
        digits.clear();
        saw_unit = true;
        match ch {
            'h' => seconds = seconds.checked_add(amount.checked_mul(60 * 60)?)?,
            'm' => seconds = seconds.checked_add(amount.checked_mul(60)?)?,
            's' => seconds = seconds.checked_add(amount)?,
            _ => return None,
        }
    }

    if saw_unit && digits.is_empty() {
        Some(seconds)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn fixed_now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }

    #[test]
    fn parses_openai_duration_reset_headers() {
        let reset = parse_reset_header("6m0s", fixed_now()).unwrap();

        assert_eq!(reset, fixed_now() + Duration::seconds(360));
    }

    #[test]
    fn parses_plain_integer_reset_as_seconds_from_now() {
        let reset = parse_reset_header("60", fixed_now()).unwrap();

        assert_eq!(reset, fixed_now() + Duration::seconds(60));
    }

    #[test]
    fn preserves_unix_timestamp_reset_headers() {
        let reset = parse_reset_header("1800000000", fixed_now()).unwrap();

        assert_eq!(reset, DateTime::from_timestamp(1_800_000_000, 0).unwrap());
    }

    #[test]
    fn parses_reset_headers_in_rate_limit_info() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-ratelimit-reset-requests".to_string(),
            "1m30s".to_string(),
        );

        let info = parse_rate_limit_headers(&headers);

        assert!(info.reset_at.is_some());
        assert!(info.reset_at.unwrap() > Utc::now());
    }

    #[test]
    fn ignores_invalid_reset_headers() {
        assert!(parse_reset_header("soon", fixed_now()).is_none());
    }
}
