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

impl std::default::Default for Grid {
    fn default() -> Self {
        Grid {
            even_beat_bg_color: hex("374140"),
            odd_beat_bg_color: darken(hex("374140"), 0.2),
            edge_step_bg_color: hex("BDC3C7"),

            even_beat_line: Stroke { color: hex("2A2C2B"), line_width: 1. },
            odd_beat_line: Stroke { color: darken(hex("2A2C2B"), 0.2), line_width: 1. },
            edge_step_line: Stroke { color: lighten(hex("2A2C2B"), 0.1), line_width: 1. },
            track_margin_color: hex("2A2C2B")
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
            contour_bg_color: hex("BDC3C7"),
            contour_width: 2.,
            bg_color: GridColor::Simple(hex("D9CB9E")),
            stroke: Stroke { color: hex("24272a"), line_width: 1. },
            slider_bg_color: GridColor::Simple(hex("ff7d00")),
            slider_highlighted_bg_color: GridColor::Simple(darken(hex("ff7d00"), 0.1)),
            negative_offset_marker_bg_color: hex("7d00ff"),
            positive_offset_marker_bg_color: hex("00ff7d")
        }
    }
}


pub trait StyleSheet {
    fn default(&self) -> Style;
    fn dragging_selection(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn default(&self) -> Style {
        Style {
            event: Event::default(),
            grid: Grid::default(),
            background: None,

            selection_stroke: Stroke { color: Color::WHITE, line_width: 1. },
            selected_track_bg_color: lighten(Color::BLACK, 0.7),
            current_step_bg_color: lighten(hex("374140"), 0.1)
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

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
