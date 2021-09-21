use super::color_utils::lighten;
use iced_native::{Background, Color};

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

pub trait StyleSheet {
    fn default(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn default(&self) -> Style {
        Style {
            background: Some(Background::Color(lighten(Color::BLACK, 0.92))),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: lighten(Color::BLACK, 0.7),
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
