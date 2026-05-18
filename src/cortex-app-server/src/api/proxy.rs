//! Port proxy endpoints for dev servers inside container.

use axum::{Json, extract::Path};

use crate::error::{AppError, AppResult};

const ALLOWED_PROXY_PORTS: &[u16] = &[
    3000, 3001, 3002, 3003, // React、Next.js
    4000, 4173, 4200, // Angular、Vite 预览
    5000, 5173, 5174, // Vite、Flask
    8000, 8080, 8081, // Django、通用开发服务
];

/// List open ports on localhost.
pub async fn list_open_ports() -> Json<Vec<u16>> {
    let mut open_ports = Vec::new();

    for port in ALLOWED_PROXY_PORTS {
        if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            open_ports.push(*port);
        }
    }

    Json(open_ports)
}

/// Proxy request to a local port (root path).
pub async fn proxy_to_port(Path(port): Path<u16>) -> AppResult<axum::response::Response> {
    proxy_to_port_path(Path((port, String::new()))).await
}

/// Proxy request to a local port with path.
pub async fn proxy_to_port_path(
    Path((port, path)): Path<(u16, String)>,
) -> AppResult<axum::response::Response> {
    use axum::body::Body;
    use axum::http::{StatusCode, header};

    if !is_proxy_port_allowed(port) {
        return Err(AppError::Authorization(format!(
            "Port {port} is not allowed for proxying"
        )));
    }

    let target_url = format!("http://127.0.0.1:{port}/{path}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {e}")))?;

    let response = client
        .get(&target_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Proxy request failed: {e}")))?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read response: {e}")))?;

    let mut builder = axum::response::Response::builder()
        .status(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK));

    // Forward content-type
    if let Some(ct) = headers.get(reqwest::header::CONTENT_TYPE)
        && let Ok(v) = ct.to_str()
    {
        builder = builder.header(header::CONTENT_TYPE, v);
    }

    // Add CORS headers
    builder = builder
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*");

    builder
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

fn is_proxy_port_allowed(port: u16) -> bool {
    ALLOWED_PROXY_PORTS.contains(&port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn proxy_rejects_non_dev_ports() {
        let result = proxy_to_port_path(Path((22, String::new()))).await;

        assert!(matches!(result, Err(AppError::Authorization(_))));
    }

    #[test]
    fn proxy_allows_only_known_dev_ports() {
        assert!(is_proxy_port_allowed(5173));
        assert!(is_proxy_port_allowed(8080));
        assert!(!is_proxy_port_allowed(22));
        assert!(!is_proxy_port_allowed(9200));
    }
}
