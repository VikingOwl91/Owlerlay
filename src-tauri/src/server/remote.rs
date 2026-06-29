//! LAN remote-control routes: a mobile page plus the token-gated control API.
//!
//! All routes here sit behind [`require_token`]. Reads/overlay routes elsewhere
//! stay open (OBS can't authenticate); only these *write* + remote-page routes
//! require the capability token. Same-origin from the phone, so no CORS.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::app_state::AppState;
use crate::countdown::commands::{build_snapshot_dtos, do_pause, do_reset, do_resume, do_start};
use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::events::AppEvent;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    // The control API requires the token. The page itself is only gated on
    // "remote enabled" (not the token) so it can boot from a saved token on a
    // bookmarked/reloaded URL that has no `?t=`; its API calls still need it.
    let api = Router::new()
        .route("/api/remote/countdowns", get(list_countdowns))
        .route("/api/remote/countdowns/{id}/{action}", post(control))
        .route("/api/remote/sse", get(remote_sse))
        .layer(middleware::from_fn_with_state(state.clone(), require_token));

    let page = Router::new()
        .route("/remote", get(remote_page))
        .layer(middleware::from_fn_with_state(state, require_enabled));

    api.merge(page)
}

/// 404 when the remote is disabled — pretend the routes don't exist.
async fn require_enabled(State(state): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    if state.remote_enabled {
        next.run(req).await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// Gate: 404 when disabled, 401 on a missing/wrong token, else pass through.
async fn require_token(State(state): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    if !state.remote_enabled {
        return StatusCode::NOT_FOUND.into_response();
    }
    let provided = extract_token(&req);
    let expected = state.remote_token.read().await;
    match provided {
        Some(t) if constant_time_eq(t.as_bytes(), expected.as_bytes()) => next.run(req).await,
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

/// Length-independent byte comparison — avoids leaking the token via the
/// short-circuit timing of `==`. (The token also rides in the URL by design of
/// the capability link; this just removes the cheapest side channel.)
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Token from `?t=<hex>` (used by the page, fetch, and EventSource) or an
/// `Authorization: Bearer <hex>` header. The token is hex, so no URL-decoding.
fn extract_token(req: &Request) -> Option<String> {
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            if let Some(v) = pair.strip_prefix("t=") {
                return Some(v.to_string());
            }
        }
    }
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(str::to_string)
}

/// The mobile control page. Fully static — it reads the token from its own URL
/// and talks to the API client-side, so no server-side templating is needed.
async fn remote_page() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Html(include_str!("../../templates/remote/remote.html")),
    )
}

async fn list_countdowns(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CountdownSnapshotDto>>, StatusCode> {
    build_snapshot_dtos(&state)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn control(
    State(state): State<Arc<AppState>>,
    Path((id, action)): Path<(u64, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let app = &state.app_handle;
    let result = match action.as_str() {
        "start" => do_start(app, &state, id).await,
        "pause" => do_pause(app, &state, id).await,
        "resume" => do_resume(app, &state, id).await,
        "reset" => do_reset(app, &state, id).await,
        other => return Err((StatusCode::BAD_REQUEST, format!("unknown action: {other}"))),
    };
    result
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

/// Live updates across *all* countdowns (the remote isn't group-scoped). Sends
/// raw `remaining_ms`/state and lets the page format + a `changed` nudge to
/// refetch the list on create/delete.
async fn remote_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.event_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|event| match event {
        Ok(AppEvent::Tick(p)) => Some(Ok(Event::default()
            .event("tick")
            .data(json!({ "id": p.id, "remaining_ms": p.remaining_ms }).to_string()))),
        Ok(AppEvent::State(p)) => Some(Ok(Event::default()
            .event("state")
            .data(json!({ "id": p.id, "state": p.state }).to_string()))),
        Ok(AppEvent::Changed(_)) | Ok(AppEvent::Reload) => {
            Some(Ok(Event::default().event("changed").data("")))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}
