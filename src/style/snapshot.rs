use super::color_utils::{hex, lighten};
use iced_native::{Background, Color};

pub struct Style {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub step_color: Color,
    pub line_edge_color: Color,
    pub line_division_color: Color,
}

pub trait StyleSheet {
    fn default(&self) -> Style;
    fn selected(&self) -> Style;
    // dirty is when snapshot is not up to date with live snapshot
    fn dirty(&self) -> Style;
}

pub struct Default;

impl StyleSheet for Default {
    fn default(&self) -> Style {
        Style {
            background: Some(Background::Color(lighten(Color::BLACK, 0.2))),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: lighten(Color::BLACK, 0.7),
            step_color: Color::from_rgb(0.46, 0.46, 0.46),
            line_edge_color: lighten(Color::BLACK, 0.25),
            line_division_color: lighten(Color::BLACK, 0.3),
        }
    }

    fn selected(&self) -> Style {
        Style {
            background: Some(Background::Color(lighten(Color::BLACK, 0.2))),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: lighten(Color::BLACK, 0.7),
            step_color: hex("ff7d00"),
            line_edge_color: lighten(Color::BLACK, 0.25),
            line_division_color: lighten(Color::BLACK, 0.3),
        }
    }

    fn dirty(&self) -> Style {
        Style {
            background: Some(Background::Color(lighten(Color::BLACK, 0.2))),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: lighten(Color::BLACK, 0.7),
            step_color: hex("ff7d00"),
            line_edge_color: lighten(Color::BLACK, 0.25),
            line_division_color: lighten(Color::BLACK, 0.3),
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: StyleSheet + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
