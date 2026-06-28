//! Guards the overlay template/context contract: every variable the templates
//! reference must be supplied, or strict-undefined rendering fails the test.
//! This is the drift that previously broke the overlay (stale route vs. new
//! templates), now caught at test time instead of in OBS.

use minijinja::{Environment, UndefinedBehavior, Value, context};
use serde::Serialize;

/// Mirrors the `CountdownView` (`c`) object the server builds in
/// `src/server/routes.rs`. Keep the field set in sync with the templates.
#[derive(Serialize)]
struct C {
    id: u64,
    name: String,
    state: String,
    icon: String,
    show_timer: bool,
    show_progress: bool,
    remaining: String,
    percent: f32,
    font_size: f32,
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

#[test]
fn overlay_templates_render_under_strict_undefined() {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.add_template(
        "html",
        include_str!("../templates/overlay/countdown/countdown.html.j2"),
    )
    .expect("html parses");
    env.add_template(
        "css",
        include_str!("../templates/overlay/countdown/countdown.css.j2"),
    )
    .expect("css parses");
    env.add_template(
        "js",
        include_str!("../templates/overlay/countdown/countdown.js.j2"),
    )
    .expect("js parses");
    env.add_template(
        "wrapper",
        include_str!("../templates/overlay/browsersource.html.j2"),
    )
    .expect("wrapper parses");

    let c = Value::from_serialize(C {
        id: 1,
        name: "Test".into(),
        state: "Running".into(),
        icon: "owl.svg".into(),
        show_timer: true,
        show_progress: true,
        remaining: "00:01:00".into(),
        percent: 50.0,
        font_size: 2.0,
        text_color: "white".into(),
        divider_color: "white".into(),
        bar_bg: "#333".into(),
        bar_fg: "#4ade80".into(),
        background: "transparent".into(),
        border: "none".into(),
        border_radius: 8,
        backdrop_filter: true,
        box_shadow: "0 0 4px black".into(),
        icon_size: "2rem".into(),
    });

    let content = env
        .get_template("html")
        .unwrap()
        .render(context! { c => c.clone(), hide_idle => true })
        .expect("html renders");
    let source_style = env
        .get_template("css")
        .unwrap()
        .render(context! { c })
        .expect("css renders");
    let script = env
        .get_template("js")
        .unwrap()
        .render(context! { hide_idle => true, id => 1u64 })
        .expect("js renders");
    let page = env
        .get_template("wrapper")
        .unwrap()
        .render(context! { content, source_style, script, layout => "column" })
        .expect("wrapper renders");

    assert!(
        page.contains("countdown-1"),
        "renders the member's element id"
    );
    assert!(
        page.contains("flex-direction: column"),
        "honours the group layout"
    );
}
