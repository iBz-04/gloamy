//! Static file handlers for the legacy embedded dashboard surface.
//!
//! The gateway does not serve UI assets. The web app in `web/` is built and
//! served separately and calls this gateway's REST/SSE API, so these handlers
//! intentionally return a clear error instead of embedded dashboard assets.

use axum::{http::StatusCode, response::IntoResponse};

/// Serve static files from `/_app/*` path
pub async fn handle_static() -> impl IntoResponse {
    no_embedded_ui_response()
}

/// SPA fallback for the embedded dashboard that this gateway no longer serves.
pub async fn handle_spa_fallback() -> impl IntoResponse {
    no_embedded_ui_response()
}

fn no_embedded_ui_response() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "This gateway serves the API only and does not host the UI. Build and serve the web app in web/, which calls this gateway's API.",
    )
}
