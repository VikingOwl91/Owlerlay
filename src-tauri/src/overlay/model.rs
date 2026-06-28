use serde::{Deserialize, Serialize};

/// Flex direction for arranging a group's countdowns in the OBS browser source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Row,
    #[default]
    Column,
}

impl Layout {
    /// CSS `flex-direction` value used by the overlay template.
    pub fn as_css(self) -> &'static str {
        match self {
            Layout::Row => "row",
            Layout::Column => "column",
        }
    }
}

/// A named group of countdowns rendered together as a single OBS browser source.
///
/// Styling lives per countdown ([`OverlayConfig`]); the group only owns the
/// layout and whether idle countdowns are hidden.
#[derive(Debug, Clone)]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub members: Vec<u64>,
    pub layout: Layout,
    pub hide_idle: bool,
}

/// Per-countdown overlay appearance. Every field the overlay templates reference
/// lives here so rendering never sees an undefined variable.
///
/// ponytail: only `icon` / `text_color` / `background` / `show_hh_mm` are set
/// from the UI today; the rest ride on [`Default`] until the styling UI lands
/// (roadmap #1 "polish countdown").
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    pub icon: String,
    pub show_timer: bool,
    pub show_progress: bool,
    /// Timer font size, in `rem`.
    pub font_size: f32,
    pub text_color: String,
    pub background: String,
    pub border: String,
    /// Corner radius, in `px`.
    pub border_radius: u32,
    pub backdrop_filter: bool,
    pub box_shadow: String,
    /// Icon size as a CSS length, e.g. `"2rem"`.
    pub icon_size: String,
    pub divider_color: String,
    pub bar_bg: String,
    pub bar_fg: String,
    pub show_hh_mm: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            icon: String::new(),
            show_timer: true,
            show_progress: false,
            font_size: 2.0,
            text_color: "white".to_string(),
            background: "transparent".to_string(),
            border: "none".to_string(),
            border_radius: 8,
            backdrop_filter: false,
            box_shadow: String::new(),
            icon_size: "2rem".to_string(),
            divider_color: "white".to_string(),
            bar_bg: "#333".to_string(),
            bar_fg: "#4ade80".to_string(),
            show_hh_mm: false,
        }
    }
}
