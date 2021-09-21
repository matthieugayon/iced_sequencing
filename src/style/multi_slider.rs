use super::color_utils::{darken, lighten};
use iced_native::{Background, Color};

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub slider: Slider,
}

#[derive(Debug, Clone, Copy)]
pub struct Slider {
    pub color: Color,
    pub marker_height: f32,
    pub marker_color: Color,
}

pub trait StyleSheet {
    fn default(&self, primary_color: Color) -> Style;
    fn active(&self, primary_color: Color) -> Style;
    fn highlight(&self, primary_color: Color) -> Slider;
    fn hovered(&self, primary_color: Color) -> Slider;
}

struct Default;

impl StyleSheet for Default {
    fn default(&self, primary_color: Color) -> Style {
        Style {
            background: Some(Background::Color(lighten(Color::BLACK, 0.92))),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: lighten(Color::BLACK, 0.7),
            slider: Slider {
                color: primary_color,
                marker_height: 1.,
                marker_color: primary_color,
            },
        }
    }

    fn active(&self, primary_color: Color) -> Style {
        Style {
            background: Some(Background::Color(lighten(Color::BLACK, 0.92))),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: lighten(Color::BLACK, 0.7),
            slider: Slider {
                color: darken(primary_color, 0.1),
                marker_height: 1.,
                marker_color: primary_color,
            },
        }
    }

    fn highlight(&self, primary_color: Color) -> Slider {
        Slider {
            color: primary_color,
            marker_height: 1.,
            marker_color: primary_color,
        }
    }

    fn hovered(&self, primary_color: Color) -> Slider {
        Slider {
            color: lighten(primary_color, 0.1),
            marker_height: 1.,
            marker_color: primary_color,
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
