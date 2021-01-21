use crate::native::grid;
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{mouse, Background, Point, Rectangle, Color, Size, Vector};

pub use crate::native::grid::{
    State, DrawMode, STEP_HEIGHT, STEP_WIDTH, STEP_MARGIN_RIGHT, TRACK_MARGIN_BOTTOM, CONTAINER_PADDING
};
pub use crate::style::grid::{Style, StyleSheet};

use ganic_no_std::{NUM_PERCS, NUM_STEPS};

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

        // let bounds_x = bounds.x.floor();
        // let bounds_y = bounds.y.floor();

        // let bounds_width = bounds.width.floor();
        // let bounds_height = bounds.height.floor();

        let grid = draw_grid(bounds);
      

        (
            Primitive::Group {
                primitives: grid,
            },
            mouse::Interaction::default(),
        )
    }
}

fn draw_grid(bounds: Rectangle) -> Vec<Primitive> {
    let background = Primitive::Quad {
        bounds,
        background: Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
        border_radius: 0.0,
        border_width: 0.0,
        border_color: Color::from_rgba(1.0, 1.0, 1.0, 0.5),
    };

    let mut primitives = vec![background];

    // now render grid
    for step in 0..NUM_STEPS {
        for track in 0..NUM_PERCS {
            let step_offset_x = CONTAINER_PADDING + (step as f32 * (STEP_WIDTH + STEP_MARGIN_RIGHT));
            let step_offset_y = CONTAINER_PADDING + (track as f32 * (STEP_HEIGHT + TRACK_MARGIN_BOTTOM));

            primitives.push(Primitive::Quad {
                bounds: Rectangle{
                    x: step_offset_x,
                    y: step_offset_y,
                    width: STEP_WIDTH,
                    height: STEP_HEIGHT
                },
                background: Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            })
        }
    }

    primitives
}
