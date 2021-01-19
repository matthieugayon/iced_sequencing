use crate::native::grid;
use iced_graphics::canvas::{Frame, LineCap, Path, Stroke};
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{mouse, Background, Point, Rectangle, Size, Vector};

pub use crate::native::grid::{State, DrawMode};
pub use crate::style::grid::{Style, StyleSheet};

pub type Grid<'a, Message, Backend> =
    grid::Grid<'a, Message, Renderer<Backend>>;

impl<B: Backend> grid::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        state: State,
        style_sheet: &Self::Style
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        // let style = if is_dragging {
        //     style_sheet.dragging()
        // } else if is_mouse_over {
        //     style_sheet.hovered()
        // } else {
        //     style_sheet.active()
        // };

        let bounds_x = bounds.x.floor();
        let bounds_y = bounds.y.floor();

        let bounds_width = bounds.width.floor();
        let bounds_height = bounds.height.floor();

      

        (
            Primitive::Group {
                primitives: vec![],
            },
            mouse::Interaction::default(),
        )
    }
}

