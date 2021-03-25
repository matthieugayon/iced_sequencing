use iced_native::Rectangle;

pub fn get_step_dimension(bounds: Rectangle, x_div: usize, y_div: usize) -> Rectangle {
    let width = (bounds.width as u32 / x_div as u32) as f32;
    let x = (((bounds.width - x_div as f32 * width) as u32) / 2) as f32;

    let height = (bounds.height as u32 / y_div as u32) as f32;
    let y = (((bounds.height - y_div as f32 * height) as u32) / 2) as f32;

    Rectangle { x, y, width, height }
}