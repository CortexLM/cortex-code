//! MCP (Model Context Protocol) management commands.
//!
//! This module provides CLI commands for managing MCP servers, including:
//! - Listing configured servers
//! - Adding/removing servers
//! - Enabling/disabling servers
//! - OAuth authentication
//! - Debug and connection testing

mod auth;
mod config;
mod debug;
mod handlers;
mod macros;
mod types;
mod validation;

use anyhow::Result;

// Re-export public types
pub use types::{McpCli, McpSubcommand};

pub(super) fn ellipsize_for_display(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }

    let ellipsis_chars = "...".chars().count();
    if max_chars <= ellipsis_chars {
        return ".".repeat(max_chars);
    }

    let prefix: String = text.chars().take(max_chars - ellipsis_chars).collect();
    format!("{prefix}...")
}

impl McpCli {
    /// Run the MCP command.
    pub async fn run(self) -> Result<()> {
        let McpCli {
            config_overrides: _,
            subcommand,
        } = self;

        match subcommand {
            McpSubcommand::List(args) | McpSubcommand::Ls(args) => handlers::run_list(args).await,
            McpSubcommand::Get(args) => handlers::run_get(args).await,
            McpSubcommand::Add(args) => handlers::run_add(args).await,
            McpSubcommand::Remove(args) => handlers::run_remove(args).await,
            McpSubcommand::Enable(args) => handlers::run_enable(args).await,
            McpSubcommand::Disable(args) => handlers::run_disable(args).await,
            McpSubcommand::Rename(args) => handlers::run_rename(args).await,
            McpSubcommand::Auth(cmd) => auth::run_auth_command(cmd).await,
            McpSubcommand::Logout(args) => auth::run_logout(args).await,
            McpSubcommand::Debug(args) => debug::run_debug(args).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ellipsize_for_display;

    #[test]
    fn test_ellipsize_for_display_preserves_short_text() {
        assert_eq!(
            ellipsize_for_display("https://example.com", 25),
            "https://example.com"
        );
    }

    #[test]
    fn test_ellipsize_for_display_matches_ascii_byte_slice_output() {
        let url = "https://example.com/some/very/long/path";
        assert_eq!(ellipsize_for_display(url, 25), "https://example.com/so...");
        assert_eq!(
            ellipsize_for_display(url, 38),
            "https://example.com/some/very/long/..."
        );
    }

    #[test]
    fn test_ellipsize_for_display_handles_utf8_boundaries() {
        let url = "http://例え.jp/path/to/endpoint";
        let display = ellipsize_for_display(url, 25);

        assert!(display.ends_with("..."));
        assert!(display.len() <= url.len());
        assert!(display.is_char_boundary(display.len()));
    }

    #[test]
    fn test_ellipsize_for_display_handles_small_limits() {
        assert_eq!(ellipsize_for_display("hello", 3), "...");
        assert_eq!(ellipsize_for_display("hello", 2), "..");
        assert_eq!(ellipsize_for_display("hello", 0), "");
    }
}
