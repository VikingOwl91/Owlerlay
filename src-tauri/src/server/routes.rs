use crate::app_state::AppState;
use crate::countdown::commands::build_snapshot_dtos;
use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::events::AppEvent;
use crate::countdown::model::CountdownState;
use crate::overlay::model::{OverlayConfig, TimeFormat};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use minijinja::{Environment, Value, context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Deserialize)]
pub struct OverlayQuery {
    group: u64,
}

/// The `c` object the overlay templates render: per-countdown live state merged
/// with its [`OverlayConfig`] styling.
#[derive(Serialize)]
struct CountdownView {
    id: u64,
    name: String,
    state: CountdownState,
    icon: String,
    show_timer: bool,
    show_progress: bool,
    remaining: String,
    percent: f32,
    font_size: f32,
    font_family: String,
    text_color: String,
    divider_color: String,
    bar_bg: String,
    bar_fg: String,
    background: String,
    border: String,
    border_radius: u32,
    backdrop_filter: bool,
    box_shadow: String,
    icon_size: String,
}

impl CountdownView {
    fn build(snap: &CountdownSnapshotDto, config: &OverlayConfig) -> Self {
        Self {
            id: snap.id,
            name: snap.label.clone(),
            state: snap.state,
            icon: config.icon.clone(),
            show_timer: config.show_timer,
            show_progress: config.show_progress,
            remaining: config.time_format.format(snap.duration as u64),
            percent: percent_of(snap.duration, snap.initial_duration),
            font_size: config.font_size,
            font_family: config.font_family.clone(),
            text_color: config.text_color.clone(),
            divider_color: config.divider_color.clone(),
            bar_bg: config.bar_bg.clone(),
            bar_fg: config.bar_fg.clone(),
            background: config.background.clone(),
            border: config.border.clone(),
            border_radius: config.border_radius,
            backdrop_filter: config.backdrop_filter,
            box_shadow: config.box_shadow.clone(),
            icon_size: config.icon_size.clone(),
        }
    }
}

/// Renders a group as a full OBS browser-source page (wrapper + per-countdown
/// HTML/CSS + the SSE client script).
pub async fn overlay_group(
    State(state): State<Arc<AppState>>,
    Query(q): Query<OverlayQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let group = state.overlay_service.get_group(q.group).await.ok_or((
        StatusCode::NOT_FOUND,
        format!("overlay group {} not found", q.group),
    ))?;

    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let by_id: HashMap<u64, &CountdownSnapshotDto> = snapshots.iter().map(|s| (s.id, s)).collect();

    let env = overlay_env();
    let item_html = env.get_template("item_html").map_err(internal)?;
    let item_css = env.get_template("item_css").map_err(internal)?;

    let mut content = String::new();
    let mut source_style = String::new();
    for member in &group.members {
        let Some(snap) = by_id.get(member) else {
            continue;
        };
        let config = state.overlay_service.get_config(*member).await;
        let c = Value::from_serialize(CountdownView::build(snap, &config));
        content.push_str(
            &item_html
                .render(context! { c => c.clone(), hide_idle => group.hide_idle })
                .map_err(internal)?,
        );
        source_style.push_str(&item_css.render(context! { c }).map_err(internal)?);
    }

    let script = env
        .get_template("item_js")
        .map_err(internal)?
        .render(context! { hide_idle => group.hide_idle, id => group.id })
        .map_err(internal)?;

    let html = env
        .get_template("browsersource")
        .map_err(internal)?
        .render(context! {
            content,
            source_style,
            script,
            layout => group.layout.as_css(),
        })
        .map_err(internal)?;

    // No caching: a config/membership change fires a `reload`, and the browser
    // must re-fetch the freshly rendered page instead of serving a stale copy.
    Ok(([(header::CACHE_CONTROL, "no-store")], Html(html)))
}

/// Live updates for a group's countdowns. Emits `countdown-tick` (JSON
/// `{id, remaining, percent}`) on every tick and `reload` when the group's
/// rendered page goes stale.
pub async fn sse_group(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<u64>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, axum::Error>>>, StatusCode> {
    let group = state
        .overlay_service
        .get_group(group_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let members: HashSet<u64> = group.members.iter().copied().collect();

    // Cache each member's initial duration + time format at connect. Membership
    // and config changes emit a reload, so the client reconnects with fresh data.
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let initials: HashMap<u64, u128> = snapshots
        .iter()
        .filter(|s| members.contains(&s.id))
        .map(|s| (s.id, s.initial_duration))
        .collect();
    let mut formats: HashMap<u64, TimeFormat> = HashMap::new();
    for member in &members {
        formats.insert(
            *member,
            state.overlay_service.get_config(*member).await.time_format,
        );
    }

    let rx = state.event_bus.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |event| match event {
        Ok(AppEvent::Tick(p)) if members.contains(&p.id) => {
            let initial = initials.get(&p.id).copied().unwrap_or(0);
            let fmt = formats.get(&p.id).copied().unwrap_or_default();
            let data = serde_json::json!({
                "id": p.id,
                "remaining": fmt.format(p.remaining_ms),
                "percent": percent_of(p.remaining_ms as u128, initial),
            });
            Some(Ok(Event::default()
                .event("countdown-tick")
                .data(data.to_string())))
        }
        Ok(AppEvent::State(p)) if members.contains(&p.id) => {
            let data = serde_json::json!({ "id": p.id, "state": p.state });
            Some(Ok(Event::default()
                .event("countdown-state")
                .data(data.to_string())))
        }
        Ok(AppEvent::Changed(snaps)) => snaps
            .iter()
            .any(|s| members.contains(&s.id))
            .then(|| Ok(Event::default().event("reload").data(""))),
        Ok(AppEvent::Reload) => Some(Ok(Event::default().event("reload").data(""))),
        _ => None,
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}

pub async fn list_icons() -> Json<Vec<String>> {
    let mut names = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir("public/icons").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("svg" | "png")
            ) && let Some(filename) = path.file_name().and_then(|s| s.to_str())
            {
                names.push(filename.to_string());
            }
        }
    }
    names.sort();
    Json(names)
}

/// The overlay template environment. The templates are compiled into the
/// binary, so a parse failure here is a build-time bug, not a runtime condition.
/// Built once and cached — the templates never change between requests.
fn overlay_env() -> &'static Environment<'static> {
    static ENV: std::sync::OnceLock<Environment<'static>> = std::sync::OnceLock::new();
    ENV.get_or_init(|| {
        let mut env = Environment::new();
        // Surface missing context fields as render errors instead of silently
        // emitting blanks — this is the drift that broke the overlay before.
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        env.add_template(
            "browsersource",
            include_str!("../../templates/overlay/browsersource.html.j2"),
        )
        .expect("browsersource template should parse");
        env.add_template(
            "item_html",
            include_str!("../../templates/overlay/countdown/countdown.html.j2"),
        )
        .expect("countdown html template should parse");
        env.add_template(
            "item_css",
            include_str!("../../templates/overlay/countdown/countdown.css.j2"),
        )
        .expect("countdown css template should parse");
        env.add_template(
            "item_js",
            include_str!("../../templates/overlay/countdown/countdown.js.j2"),
        )
        .expect("countdown js template should parse");
        env
    })
}

fn internal(e: minijinja::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// Progress as the *remaining* fraction, so a countdown bar depletes toward 0.
// ponytail: flip to elapsed (`100 - this`) if you'd rather the bar fill up.
fn percent_of(remaining_ms: u128, initial_ms: u128) -> f32 {
    if initial_ms == 0 {
        0.0
    } else {
        (remaining_ms as f32 / initial_ms as f32) * 100.0
    }
}

