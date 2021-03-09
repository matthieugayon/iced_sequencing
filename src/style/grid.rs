use std::collections::HashMap;
use iced_native::Color;

use super::hex::from_hex;

fn hex(hex_str: &str) -> Color {
    let color_tuple = from_hex(hex_str);
    Color::from_rgb(color_tuple.0 / 255., color_tuple.1 / 255., color_tuple.2 / 255.)
}

const STEP_BG_COLOR: Color = Color::from_rgba(0.5, 0.5, 0.5, 0.25);
const STEP_BG_COLOR_2: Color = Color::from_rgba(0.5, 0.5, 0.5, 0.35);
const STEP_BORDER_LEFT_COLOR: Color = Color::from_rgb(
    0x25 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x2A as f32 / 255.0, //180b28
);
const STEP_BORDER_LEFT_COLOR_2: Color = Color::from_rgba(0.7, 0.7, 0.7, 0.55);
const STEP_LINE_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const STEP_LINE_COLOR_2: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_HIGHLIGHT_BG_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_BORDER_COLOR: Color = STEP_BORDER_LEFT_COLOR_2;
const EVENT_HIGHLIGHT_BORDER_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_MARKER_COLOR: (Color, Color) = (
    Color::from_rgb(0.976, 0.973, 0.027), // yellow
    Color::from_rgb(0., 0.753, 0.039), // green
);
const EVENT_HIGHLIGHT_MARKER_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const EVENT_SELECTED_COLOR: Color = Color::from_rgb(0.894, 0.953, 0.059);
const EVENT_SELECTED_BORDER_COLOR: Color = Color::from_rgba(0.1, 0.1, 0.1, 1.);
const EVENT_SELECTED_MARKER_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);
const SELECTION_BORDER_COLOR: Color = Color::from_rgb(0., 0., 0.);

// const EVENT_BG_COLOR: Color = Color::from_rgb(0.315, 0.315, 0.315);

// let event_bg_color: HashMap<_, _> = vec![
//     (0, Color::from_rgb(0.396, 0.153, 0.043)), // purple
//     (1, Color::from_rgb(0.8, 0., 0.035))
// ].into_iter().collect();

#[derive(Debug, Clone)]
pub struct Style {
    pub step_bg_color: Color,
    pub step_bg_color_2: Color,
    pub step_border_left_color: Color,
    pub step_border_left_color_2: Color,
    pub step_line_color: Color,
    pub step_line_color_2: Color,
    pub event_bg_color: HashMap<usize, Color>,
    pub event_highlight_bg_color: Color,
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
    event_bg_color.insert(9, hex("6527b5"));
    event_bg_color.insert(8, hex("005ce1"));

    let red = hex("0098e7");
    event_bg_color.insert(7, red);
    event_bg_color.insert(6, red);
    event_bg_color.insert(5, red);

    let orange = hex("00aeca");
    event_bg_color.insert(4, orange);
    event_bg_color.insert(3, orange);

    let orange_light = hex("00c0a4");
    event_bg_color.insert(2, orange_light);
    event_bg_color.insert(1, orange_light);
    event_bg_color.insert(0, orange_light);

    event_bg_color
}

struct Default;

impl StyleSheet for Default {
    fn default(&self) -> Style {
        Style {
            step_bg_color: STEP_BG_COLOR,
            step_bg_color_2: STEP_BG_COLOR_2,
            step_border_left_color: STEP_BORDER_LEFT_COLOR,
            step_border_left_color_2: STEP_BORDER_LEFT_COLOR_2,
            step_line_color: STEP_LINE_COLOR,
            step_line_color_2: STEP_LINE_COLOR_2,
            event_bg_color: get_event_bg_color(),
            event_highlight_bg_color: EVENT_HIGHLIGHT_BG_COLOR,
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
            step_bg_color: STEP_BG_COLOR,
            step_bg_color_2: STEP_BG_COLOR_2,
            step_border_left_color: STEP_BORDER_LEFT_COLOR,
            step_border_left_color_2: STEP_BORDER_LEFT_COLOR_2,
            step_line_color: STEP_LINE_COLOR,
            step_line_color_2: STEP_LINE_COLOR_2,
            event_bg_color: get_event_bg_color(),
            event_highlight_bg_color: EVENT_HIGHLIGHT_BG_COLOR,
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
