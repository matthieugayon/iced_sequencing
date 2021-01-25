use crate::native::grid;
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{mouse, Background, Point, Rectangle, Color, Size};
use crate::core::grid::{
    get_step_dimensions,
    get_event_absolute_position,
    normalize_point,
    GridPattern,
    STEP_MARGIN_RIGHT, TRACK_MARGIN_BOTTOM, CONTAINER_PADDING
};

pub use crate::native::grid::State;
pub use crate::style::grid::{Style, StyleSheet};

use ganic_no_std::{NUM_PERCS, NUM_STEPS};

pub type Grid<'a, Message, Backend> =
    grid::Grid<'a, Message, Renderer<Backend>>;

impl<B: Backend> grid::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        _cursor_position: Point,
        grid_pattern: GridPattern,
        selection: Option<Rectangle>,
        _style_sheet: &Self::Style
    ) -> Self::Output {
        // let is_mouse_over = bounds.contains(cursor_position);
        let step_size = get_step_dimensions(bounds);

        // println!("draw bounds {:?}", bounds);


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

        let grid = draw_grid(bounds, step_size);
        let steps = draw_steps(grid_pattern, bounds, step_size);
        let mut primitives = vec![grid, steps];

        match selection {
            Some(normalized_area) => {
                let selection_area = Primitive::Quad {
                    bounds: Rectangle {
                        x: (normalized_area.x + bounds.x).ceil(),
                        y: (normalized_area.y + bounds.y).ceil(),
                        width: (normalized_area.width).ceil(),
                        height: (normalized_area.height).ceil()
                    },
                    background: Background::Color(Color::TRANSPARENT),
                    border_radius: 0.0,
                    border_width: 1.0,
                    border_color: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
                };

                primitives.push(selection_area);
            }
            None => {}
        }

        (
            Primitive::Group {
                primitives
            },
            mouse::Interaction::default(),
        )
    }
}

fn draw_grid(bounds: Rectangle, step_size: Size) -> Primitive {
    let background = Primitive::Quad {
        bounds,
        background: Background::Color(Color::from_rgba(0.8, 0.8, 0.8, 0.5)),
        border_radius: 0.0,
        border_width: 0.0,
        border_color: Color::from_rgba(0.9, 0.9, 0.9, 0.5),
    };

    let mut primitives = vec![background];

    // now render grid
    for step in 0..NUM_STEPS {
        for track in 0..NUM_PERCS {
            let step_offset_x = CONTAINER_PADDING + (step as f32 * step_size.width);
            let step_offset_y = CONTAINER_PADDING + (track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM));

            primitives.push(Primitive::Quad {
                bounds: Rectangle{
                    x: step_offset_x + bounds.x,
                    y: step_offset_y + bounds.y,
                    width: step_size.width - STEP_MARGIN_RIGHT,
                    height: step_size.height
                },
                background: Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            })
        }
    }

    Primitive::Group {
        primitives
    }
}

fn draw_steps(grid_pattern: GridPattern, bounds: Rectangle, step_size: Size) -> Primitive {
    let normalized_bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: bounds.width,
        height: bounds.height
    };

    Primitive::Group {
        primitives: grid_pattern.data
            .iter()
            .map(|((step, track), grid_event)| {
                let event_position = get_event_absolute_position(*step, *track, grid_event.offset, normalized_bounds);
                let event_offset_y = CONTAINER_PADDING + (*track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM));
                let border_width: f32 = {
                    if grid_event.selected {
                        4.0
                    } else {
                        1.0
                    }
                };

                let border_color: Color = {
                    if grid_event.selected {
                        Color::from_rgba(0.0, 1.0, 1.0, 0.2)
                    } else {
                        Color::BLACK
                    }
                };

                Primitive::Quad {
                    bounds: Rectangle{
                        x: event_position.x + bounds.x,
                        y: event_offset_y + bounds.y,
                        width: step_size.width - STEP_MARGIN_RIGHT,
                        height: step_size.height,
                    },
                    background: Background::Color(Color::from_rgba(0.0, 1.0, 0.0, 0.5)),
                    border_radius: 0.0,
                    border_width,
                    border_color
                }
            })
            .collect()
    }
}
