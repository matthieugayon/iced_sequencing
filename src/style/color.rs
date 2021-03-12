use palette::{LinSrgb, Shade};
use iced_native::Color;

pub fn lighten(color: Color, amount: f32) -> Color {
    let mut srgb_color = LinSrgb::new(color.r, color.g, color.b);
    srgb_color = srgb_color.lighten(amount);
    Color::from_rgb(srgb_color.red, srgb_color.green, srgb_color.blue)
}

pub fn darken(color: Color, amount: f32) -> Color {
    let mut srgb_color = LinSrgb::new(color.r, color.g, color.b);
    srgb_color = srgb_color.darken(amount);
    Color::from_rgb(srgb_color.red, srgb_color.green, srgb_color.blue)
}

