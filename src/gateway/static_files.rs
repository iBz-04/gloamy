//! Static file handlers for the deprecated web dashboard surface.
//!
//! Gloamy now prefers the desktop application. These handlers intentionally
//! return a clear error instead of serving embedded dashboard assets.

use axum::{
    http::StatusCode,
    response::IntoResponse,
};

/// Serve static files from `/_app/*` path
pub async fn handle_static() -> impl IntoResponse {
    desktop_only_response()
}

/// SPA fallback for the removed browser dashboard.
pub async fn handle_spa_fallback() -> impl IntoResponse {
    desktop_only_response()
}

fn desktop_only_response() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The browser dashboard has been removed. Use the Gloamy desktop app instead.",
    )
}
