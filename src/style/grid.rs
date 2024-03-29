use ganic_no_std::NUM_PERCS;
use iced_native::Color;
use super::color_utils::{hex, lighten, darken};

#[derive(Debug, Clone, Copy)]
pub struct WidgetBackground {
    pub bg_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    pub border_color: Color
}

#[derive(Debug, Clone, Copy)]
pub enum GridColor {
    Simple(Color),
    Multitrack([Color; NUM_PERCS])
}

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub line_width: f32
}

#[derive(Debug, Clone)]
pub struct Style {
    pub event: Event,
    pub grid: Grid,
    pub background: Option<WidgetBackground>,

    pub selection_stroke: Stroke,
    pub selected_track_bg_color: Color,
    pub current_step_bg_color: Color
}

#[derive(Debug, Clone)]
pub struct Grid {
    // BACKGROUNDS
    pub even_beat_bg_color: Color,
    pub odd_beat_bg_color: Color,
    pub edge_step_bg_color: Color,

    // LINES
    pub even_beat_line: Stroke,
    pub odd_beat_line: Stroke,
    pub edge_step_line: Stroke,
    pub track_margin_color: Color
}

// shades => dark to light
// #1A2122
// #1f2829
// #252f30
// #2a3637
// #303d3e
// #354345

impl std::default::Default for Grid {
    fn default() -> Self {
        Grid {
            even_beat_bg_color: hex("2a3637"),
            odd_beat_bg_color: hex("252f30"),
            edge_step_bg_color: hex("1f2829"),

            even_beat_line: Stroke { color: hex("1A2122"), line_width: 1.0 },
            odd_beat_line: Stroke { color: hex("1A2122"), line_width: 1.0 },
            edge_step_line: Stroke { color: darken(hex("2a3637"), 0.14), line_width: 1. },
            track_margin_color: hex("1a2122")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    // BACKGROUNDS
    pub contour_bg_color: Color,
    pub contour_width: f32,
    pub bg_color: GridColor,
    pub stroke: Stroke,
    pub slider_bg_color: GridColor,
    pub slider_highlighted_bg_color: GridColor,
    pub negative_offset_marker_bg_color: Color,
    pub positive_offset_marker_bg_color: Color,
}


impl std::default::Default for Event {
    fn default() -> Self {
        Event {
            contour_bg_color: hex("fafafa"),
            contour_width: 3.,
            bg_color: GridColor::Multitrack([
                darken(hex("ff7e53"), 0.12),
                darken(hex("eb8c63"), 0.13),
                darken(hex("d69a73"), 0.14),
                darken(hex("c2a883"), 0.15),
                darken(hex("aeb693"), 0.16),
                darken(hex("99c4a4"), 0.17),
                darken(hex("85d2b4"), 0.18),
                darken(hex("71e0c4"), 0.19),
                darken(hex("5ceed4"), 0.20),
                darken(hex("48fce4"), 0.21),
            ]),
            stroke: Stroke { color: hex("2a3637"), line_width: 1. },
            slider_bg_color: GridColor::Multitrack([
                lighten(hex("ff7e53"), 0.17),
                lighten(hex("eb8c63"), 0.18),
                lighten(hex("d69a73"), 0.19),
                lighten(hex("c2a883"), 0.20),
                lighten(hex("aeb693"), 0.21),
                lighten(hex("99c4a4"), 0.27),
                lighten(hex("85d2b4"), 0.28),
                lighten(hex("71e0c4"), 0.29),
                lighten(hex("5ceed4"), 0.30),
                lighten(hex("48fce4"), 0.31),
            ]),
            slider_highlighted_bg_color: GridColor::Multitrack([
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff"),
                hex("ffffff")
            ]),
            negative_offset_marker_bg_color: hex("fc4860"),
            positive_offset_marker_bg_color: hex("48bafc")
        }
    }
}


pub trait StyleSheet {
    fn default(&self) -> Style;
    fn dragging_selection(&self) -> Style;
}

pub struct MyDefault;

impl StyleSheet for MyDefault {
    fn default(&self) -> Style {
        Style {
            event: Event::default(),
            grid: Grid::default(),
            background: None,

            selection_stroke: Stroke { color: hex("8ea5a8"), line_width: 1.0 },
            selected_track_bg_color: lighten(Color::BLACK, 0.7),
            current_step_bg_color: hex("303d3e")
        }
    }

    fn dragging_selection(&self) -> Style {
        Style {
            event: Event {
                stroke: Stroke { color: lighten(Color::BLACK, 0.1), line_width: 1. },
                ..Event::default()
            },
            ..self.default()
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(MyDefault)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: StyleSheet + 'a,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
