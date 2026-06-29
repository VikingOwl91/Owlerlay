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

/// How the overlay renders a countdown's remaining time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    /// Smallest grouping that fits — strips leading all-zero units: ≥1d shows
    /// days, ≥1h shows H:M:S, ≥1m shows M:S, else just seconds.
    #[default]
    Auto,
    /// `DD:HH:MM:SS`
    Dhms,
    /// `HH:MM:SS` (hours may exceed 24)
    Hms,
    /// `MM:SS` (minutes may exceed 59)
    Ms,
    /// Total remaining seconds.
    S,
}

impl TimeFormat {
    /// Render `ms` of remaining time per this format. Fixed-grouping modes let
    /// the leading unit overflow (e.g. `MM:SS` of 2h is `120:00`); `Auto` strips
    /// leading all-zero groups, padding the rest.
    pub fn format(self, ms: u64) -> String {
        let total = ms / 1_000;
        let s = total % 60;
        let m = (total / 60) % 60;
        let h = (total / 3_600) % 24;
        let d = total / 86_400;
        match self {
            TimeFormat::S => total.to_string(),
            TimeFormat::Ms => format!("{:02}:{s:02}", total / 60),
            TimeFormat::Hms => format!("{:02}:{m:02}:{s:02}", total / 3_600),
            TimeFormat::Dhms => format!("{d:02}:{h:02}:{m:02}:{s:02}"),
            TimeFormat::Auto if d > 0 => format!("{d:02}:{h:02}:{m:02}:{s:02}"),
            TimeFormat::Auto if h > 0 => format!("{h:02}:{m:02}:{s:02}"),
            TimeFormat::Auto if m > 0 => format!("{m:02}:{s:02}"),
            TimeFormat::Auto => format!("{s:02}"),
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
/// Deserialized straight from the `set_overlay_config` payload; `#[serde(default)]`
/// lets a partial payload fall back to [`Default`] field by field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayConfig {
    pub icon: String,
    pub show_timer: bool,
    pub show_progress: bool,
    /// Timer font size, in `rem`.
    pub font_size: f32,
    /// CSS `font-family` stack for the timer. `"inherit"` keeps the page's
    /// default mono face; the control UI sends a curated stack per widget.
    pub font_family: String,
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
    pub time_format: TimeFormat,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            icon: String::new(),
            show_timer: true,
            show_progress: false,
            font_size: 2.0,
            font_family: "inherit".to_string(),
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
            time_format: TimeFormat::Auto,
        }
    }
}
