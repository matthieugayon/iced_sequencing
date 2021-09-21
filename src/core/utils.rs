use iced_native::{Rectangle, Size};

pub fn get_step_dimension(bounds: Rectangle, x_div: usize, y_div: usize) -> Size {
    let width = bounds.width / x_div as f32;
    let height = bounds.height / y_div as f32;

    Size { width, height }
}
