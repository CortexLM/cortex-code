//! Cortex App Server - HTTP API server for Cortex Agent.
//!
//! This crate provides:
//! - REST API for managing sessions and conversations
//! - WebSocket API for real-time streaming using real cortex-core Sessions
//! - Authentication and authorization
//! - Rate limiting and request validation
//! - Health checks and metrics
//! - Static file serving for web UI
//! - mDNS/Bonjour service discovery for automatic server discovery
//!
//! The server uses cortex-core's Session system to provide the same
//! tool execution capabilities as the CLI (Execute, Read, Create, Edit, etc.)

#![deny(clippy::print_stdout, clippy::print_stderr)]

pub mod admin;
pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod file_watcher;
pub mod handlers;
pub mod mdns;
pub mod middleware;
pub mod session_manager;
pub mod share;
pub mod state;
pub mod storage;
pub mod streaming;
pub mod tasks;
pub mod terminal_streaming;
pub mod tools;
pub mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

pub use config::ServerConfig;
pub use error::{AppError, AppResult};
pub use mdns::{DiscoveredServer, MdnsDiscovery, MdnsPublisher};
pub use state::AppState;

/// Run the server with the given configuration.
pub async fn run(config: ServerConfig) -> anyhow::Result<()> {
    run_with_shutdown(config, std::future::pending()).await
}

/// Run the server with graceful shutdown support.
pub async fn run_with_shutdown<F>(config: ServerConfig, shutdown: F) -> anyhow::Result<()>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let addr: SocketAddr = config.listen_addr.parse()?;

    // Warn if authentication is disabled
    if !config.auth.enabled {
        warn_auth_disabled(addr);
    }

    let state = Arc::new(AppState::new(config.clone()).await?);
    let state_for_cleanup = Arc::clone(&state);
    let app = create_router_with_state(state);

    info!("Starting Cortex server on {}", addr);

    // Start mDNS publisher if enabled
    let mdns_publisher = if config.mdns.enabled {
        match MdnsPublisher::new(addr.port(), config.mdns.service_name.clone()) {
            Ok(publisher) => {
                if let Err(e) = publisher.publish(addr.port()).await {
                    warn!(
                        "Failed to publish mDNS service: {}. Server will continue without mDNS.",
                        e
                    );
                    None
                } else {
                    info!("mDNS service published successfully");
                    Some(publisher)
                }
            }
            Err(e) => {
                warn!(
                    "Failed to create mDNS publisher: {}. Server will continue without mDNS.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    // Graceful shutdown: close all active sessions first
    // This ensures WebSocket clients receive proper close frames
    info!("Server shutting down, cleaning up active sessions...");
    state_for_cleanup.cli_session_manager.shutdown_all().await;

    // Cleanup mDNS on shutdown
    if let Some(publisher) = mdns_publisher {
        if let Err(e) = publisher.unpublish().await {
            warn!("Failed to unpublish mDNS service: {}", e);
        }
        if let Err(e) = publisher.shutdown() {
            warn!("Failed to shutdown mDNS daemon: {}", e);
        }
    }

    Ok(())
}

fn warn_auth_disabled(addr: SocketAddr) {
    warn!("Server running without authentication!");
    warn!("{}", auth_disabled_exposure_warning(addr));
    warn!("Use --auth to enable authentication.");
}

fn auth_disabled_exposure_warning(addr: SocketAddr) -> &'static str {
    if addr.ip().is_loopback() {
        "Only local processes can access this server."
    } else {
        "Anyone on the network can access this server."
    }
}

/// Create the application router.
pub fn create_router(state: AppState) -> Router {
    create_router_with_state(Arc::new(state))
}

/// Create the application router with an Arc-wrapped state.
///
/// This variant is useful when you need to keep a reference to the state
/// for cleanup purposes (e.g., during graceful shutdown).
pub fn create_router_with_state(state: Arc<AppState>) -> Router {
    let api_routes = api::routes()
        .merge(websocket::routes())
        .merge(streaming::routes())
        .merge(share::routes())
        .merge(admin::routes());

    Router::new()
        .nest("/api/v1", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::auth_disabled_exposure_warning;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    #[test]
    fn test_auth_disabled_exposure_warning_for_loopback_and_network_binds() {
        let ipv4_loopback = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        let ipv6_loopback = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3000);
        let wildcard = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 3000);

        assert_eq!(
            auth_disabled_exposure_warning(ipv4_loopback),
            "Only local processes can access this server."
        );
        assert_eq!(
            auth_disabled_exposure_warning(ipv6_loopback),
            "Only local processes can access this server."
        );
        assert_eq!(
            auth_disabled_exposure_warning(wildcard),
            "Anyone on the network can access this server."
        );
    }
}
