use std::collections::HashMap;
use iced_native::Color;

use super::hex::from_hex;
use super::color::{lighten};


fn hex(hex_str: &str) -> Color {
    let color_tuple = from_hex(hex_str);
    Color::from_rgb(color_tuple.0 / 255., color_tuple.1 / 255., color_tuple.2 / 255.)
}

const STEP_BORDER_LEFT_COLOR: Color = Color::from_rgb(
    0x25 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x2A as f32 / 255.0, //180b28
);

// const STEP_BG_COLOR: Color = Color::from_rgba(0.5, 0.5, 0.5, 0.20);
// const STEP_BG_COLOR_2: Color = Color::from_rgba(0.5, 0.5, 0.5, 0.40);

const STEP_BORDER_LEFT_COLOR_2: Color = Color::from_rgb(0.46, 0.46, 0.46);
const STEP_LINE_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const STEP_LINE_COLOR_2: Color = Color::from_rgb(0.315, 0.315, 0.315);
// const EVENT_HIGHLIGHT_BG_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_BORDER_COLOR: Color = Color::WHITE;
const EVENT_HIGHLIGHT_BORDER_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_MARKER_COLOR: (Color, Color) = (
    Color::from_rgb(0., 0.7, 0.04), // green
    Color::from_rgb(0.7, 0., 0.04), // yellow
);
const EVENT_HIGHLIGHT_MARKER_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_SELECTED_COLOR: Color = Color::from_rgb(0.894, 0.953, 0.059);
const EVENT_SELECTED_BORDER_COLOR: Color = Color::from_rgb(0.87, 0.87, 0.87);
const EVENT_SELECTED_MARKER_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const SELECTION_BORDER_COLOR: Color = Color::from_rgb(0.8, 0.8, 0.8);


#[derive(Debug, Clone)]
pub struct Style {
    pub step_bg_color: Color,
    pub step_bg_color_2: Color,
    pub step_highlight_bg_color: Color,
    pub step_border_left_color: Color,
    pub step_border_left_color_2: Color,
    pub step_line_color: Color,
    pub step_line_color_2: Color,
    pub event_bg_color: HashMap<usize, Color>,
    pub event_highlight_bg_color: HashMap<usize, Color>,
    pub event_border_color: Color,
    pub event_highlight_border_color: Color,
    pub event_marker_color: (Color, Color),
    pub event_highlight_marker_color: Color,
    pub event_selected_color: Color,
    pub event_selected_border_color: Color,
    pub event_selected_marker_color: Color,
    pub selection_border_color: Color
}
pub trait StyleSheet {
    fn default(&self) -> Style;
    fn dragging_selection(&self) -> Style;
}

fn get_event_bg_color() -> HashMap<usize, Color> {
    let mut event_bg_color: HashMap<usize, Color> = HashMap::new();
    let purple = hex("6527b5");
    event_bg_color.insert(9, purple);

    let blue1 = hex("005ce1");
    event_bg_color.insert(8, blue1);

    let blue2 = hex("0098e7");
    event_bg_color.insert(7, blue2);
    event_bg_color.insert(6, blue2);
    event_bg_color.insert(5, blue2);

    let green1 = hex("00aeca");
    event_bg_color.insert(4, green1);
    event_bg_color.insert(3, green1);

    let green2 = hex("00c0a4");
    event_bg_color.insert(2, green2);
    event_bg_color.insert(1, green2);
    event_bg_color.insert(0, green2);

    event_bg_color
}

fn get_highlighted_event_bg_color() -> HashMap<usize, Color> {
    let mut event_bg_color: HashMap<usize, Color> = HashMap::new();
    let highlight_ratio = 0.5;
    let purple = lighten(hex("6527b5"), highlight_ratio);
    event_bg_color.insert(9, purple);

    let blue1 = lighten(hex("005ce1"), highlight_ratio);
    event_bg_color.insert(8, blue1);

    let blue2 = lighten(hex("0098e7"), highlight_ratio);
    event_bg_color.insert(7, blue2);
    event_bg_color.insert(6, blue2);
    event_bg_color.insert(5, blue2);

    let green1 = lighten(hex("00aeca"), highlight_ratio);
    event_bg_color.insert(4, green1);
    event_bg_color.insert(3, green1);

    let green2 = lighten(hex("00c0a4"), highlight_ratio);
    event_bg_color.insert(2, green2);
    event_bg_color.insert(1, green2);
    event_bg_color.insert(0, green2);

    event_bg_color
}

struct Default;

impl StyleSheet for Default {
    fn default(&self) -> Style {
        Style {
            step_bg_color: lighten(STEP_BORDER_LEFT_COLOR, 0.05),
            step_bg_color_2: lighten(STEP_BORDER_LEFT_COLOR, 0.12),
            step_highlight_bg_color: lighten(STEP_BORDER_LEFT_COLOR, 0.3),
            step_border_left_color: STEP_BORDER_LEFT_COLOR,
            step_border_left_color_2: STEP_BORDER_LEFT_COLOR_2,
            step_line_color: STEP_LINE_COLOR,
            step_line_color_2: STEP_LINE_COLOR_2,
            event_bg_color: get_event_bg_color(),
            event_highlight_bg_color: get_highlighted_event_bg_color(),
            event_border_color: EVENT_BORDER_COLOR,
            event_highlight_border_color: EVENT_HIGHLIGHT_BORDER_COLOR,
            event_marker_color: EVENT_MARKER_COLOR,
            event_highlight_marker_color: EVENT_HIGHLIGHT_MARKER_COLOR,
            event_selected_color: EVENT_SELECTED_COLOR,
            event_selected_border_color: EVENT_SELECTED_BORDER_COLOR,
            event_selected_marker_color: EVENT_SELECTED_MARKER_COLOR,
            selection_border_color: SELECTION_BORDER_COLOR
        }
    }

    fn dragging_selection(&self) -> Style {
        Style {
            step_bg_color: lighten(STEP_BORDER_LEFT_COLOR, 0.05),
            step_bg_color_2: lighten(STEP_BORDER_LEFT_COLOR, 0.12),
            step_highlight_bg_color: lighten(STEP_BORDER_LEFT_COLOR, 0.3),
            step_border_left_color: STEP_BORDER_LEFT_COLOR,
            step_border_left_color_2: STEP_BORDER_LEFT_COLOR_2,
            step_line_color: STEP_LINE_COLOR,
            step_line_color_2: STEP_LINE_COLOR_2,
            event_bg_color: get_event_bg_color(),
            event_highlight_bg_color: get_highlighted_event_bg_color(),
            event_border_color: EVENT_BORDER_COLOR,
            event_highlight_border_color: EVENT_HIGHLIGHT_BORDER_COLOR,
            event_marker_color: EVENT_MARKER_COLOR,
            event_highlight_marker_color: EVENT_HIGHLIGHT_MARKER_COLOR,
            event_selected_color: EVENT_SELECTED_COLOR,
            event_selected_border_color: EVENT_SELECTED_BORDER_COLOR,
            event_selected_marker_color: EVENT_SELECTED_MARKER_COLOR,
            selection_border_color: SELECTION_BORDER_COLOR
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
