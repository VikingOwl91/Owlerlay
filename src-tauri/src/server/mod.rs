use crate::app_state::AppState;
use axum::http::Method;
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

mod remote;
mod routes;

/// The single port the overlay + remote server listens on.
pub const PORT: u16 = 7420;

pub async fn start(state: Arc<AppState>) {
    let app = Router::new()
        .route("/api/icons", get(routes::list_icons))
        .route("/overlay", get(routes::overlay_group))
        .route("/sse/group/{id}", get(routes::sse_group))
        .nest_service("/static", ServeDir::new("public"))
        .merge(remote::router(state.clone()))
        .layer(
            // Public overlay routes only need cross-origin GETs (OBS). The
            // remote page is same-origin, so its POSTs aren't subject to CORS.
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        )
        .with_state(state.clone());

    // Opt-in LAN exposure: bind all interfaces only when the owner enabled the
    // phone remote, otherwise stay loopback-only (the historical default).
    let host = if state.remote_enabled {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let addr = format!("{host}:{PORT}");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            // Port already in use (e.g. a stale instance) shouldn't crash the
            // app — log and leave the server down rather than panic the task.
            eprintln!("overlay server: failed to bind {addr}: {e}");
            return;
        }
    };

    axum::serve(listener, app)
        .await
        .expect("overlay server crashed");
}
