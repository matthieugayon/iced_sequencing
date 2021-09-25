use colors_transform::Color as ColorTransform;
use colors_transform::Rgb;
use iced_native::Color;
use palette::{LinSrgb, Shade};

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

pub fn from_hex_unsafe(hex: &str) -> (f32, f32, f32) {
    let rgb: Rgb = Rgb::from_hex_str(hex).unwrap();
    rgb.as_tuple()
}

pub fn from_hex(hex: &str) -> Option<(f32, f32, f32)> {
    match Rgb::from_hex_str(hex) {
        Ok(rgb) => Some(rgb.as_tuple()),
        Err(_) => None,
    }
}

pub fn hex(hex_str: &str) -> Color {
    let color_tuple = from_hex_unsafe(hex_str);
    Color::from_rgb(
        color_tuple.0 / 255.,
        color_tuple.1 / 255.,
        color_tuple.2 / 255.,
    )
}
