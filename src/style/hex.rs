use colors_transform::{ Rgb, Color };

pub fn from_hex(hex: &str) -> (f32, f32, f32) {
    let rgb: Rgb = Rgb::from_hex_str(hex).unwrap();
    rgb.as_tuple()
}