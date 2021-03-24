use iced_native::Color;

use super::hex::from_hex;

fn hex(hex_str: &str) -> Color {
    let color_tuple = from_hex(hex_str);
    Color::from_rgb(color_tuple.0 / 255., color_tuple.1 / 255., color_tuple.2 / 255.)
}

pub struct Style {
    pub step_color: Color
}

pub trait StyleSheet {
    fn default(&self) -> Style;
    fn selected(&self) -> Style;
}

pub struct Default;

impl StyleSheet for Default {
    fn default(&self) -> Style {
        Style {
            step_color: Color::from_rgb(0.46, 0.46, 0.46)
        }
    }

    fn selected(&self) -> Style {
        Style {
            step_color: hex("00aeca")
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