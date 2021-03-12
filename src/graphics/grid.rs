use crate::{core::grid::GridEvent, native::grid};
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{mouse, Background, Point, Rectangle, Color, Size};
use crate::core::grid::{
    get_step_dimensions,
    get_event_absolute_position,
    GridPattern,
    TRACK_MARGIN_BOTTOM,
    CONTAINER_PADDING_LEFT,
    CONTAINER_PADDING_TOP
};

pub use crate::native::grid::State;
pub use crate::style::grid::{Style, StyleSheet};
pub use crate::style::color::{lighten,darken};

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
        mouse_interaction: mouse::Interaction,
        style_sheet: &Self::Style
    ) -> Self::Output {

        let style = style_sheet.default();

        let step_size = get_step_dimensions(bounds);

        let ceiled_bounds = Rectangle {
            x: bounds.x.ceil(),
            y: bounds.y.ceil(),
            width: bounds.width.ceil(),
            height: bounds.height.ceil()
        };

        let grid = draw_grid(ceiled_bounds, step_size, &style);
        let steps = draw_steps(grid_pattern, ceiled_bounds, step_size, &style);
        let mut primitives = vec![grid, steps];

        match selection {
            Some(selection) => {
                primitives.push(draw_selection(ceiled_bounds, selection, &style));
            }
            None => {}
        }

        (
            Primitive::Group {
                primitives
            },
            mouse_interaction,
        )
    }
}

fn draw_selection(bounds: Rectangle, area: Rectangle, style: &Style) -> Primitive {
    let bounds = Rectangle {
        x: area.x + bounds.x,
        y: area.y + bounds.y,
        width: area.width,
        height: area.height
    };

    Primitive::Quad {
        bounds,
        background: Background::Color(Color::TRANSPARENT),
        border_radius: 0.0,
        border_width: 1.0,
        border_color: style.selection_border_color
    }
}

fn draw_grid(bounds: Rectangle, step_size: Size, style: &Style) -> Primitive {
    let mut primitives:Vec<Primitive> = vec![];

    // now render grid
    for step in 0..NUM_STEPS {
        for track in 0..NUM_PERCS {
            let step_offset_x = CONTAINER_PADDING_LEFT + (step as f32 * step_size.width);
            let step_offset_y = CONTAINER_PADDING_TOP + (track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM));

            let step_group = step / 8;
            let first_group = step_group == 0 || step_group == 2;

            primitives.push(
                Primitive::Group {
                    primitives: vec![
                        Primitive::Quad {
                            bounds: Rectangle{
                                x: step_offset_x + bounds.x,
                                y: step_offset_y + bounds.y,
                                width: step_size.width,
                                height: step_size.height
                            },
                            background: Background::Color(if first_group { style.step_bg_color } else { style.step_bg_color_2 }),
                            border_radius: 0.0,
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                        // border left
                        Primitive::Quad {
                            bounds: Rectangle{
                                x: step_offset_x + bounds.x,
                                y: step_offset_y + bounds.y,
                                width: 1.,
                                height: step_size.height
                            },
                            background: Background::Color(if step % 2 == 0 { style.step_border_left_color_2 } else { style.step_border_left_color }),
                            border_radius: 0.0,
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        }
                    ]
                }
            )
        }
    }

    Primitive::Group {
        primitives
    }
}

fn draw_steps(grid_pattern: GridPattern, bounds: Rectangle, step_size: Size, style: &Style) -> Primitive {
    let normalized_bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: bounds.width,
        height: bounds.height
    };

    let mut events: Vec<(usize, usize, GridEvent)> = grid_pattern.data
        .iter()
        .map(|((step, track), grid_event)| {
            (*step, *track, *grid_event)
        })
        .collect();

    events.sort_by(|x,y| {
        if x.1 == y.1 {
            return x.0.cmp(&y.0)
        }
        x.1.cmp(&y.1)
    });

    let selected_events: Vec<&(usize, usize, GridEvent)> = events
        .iter()
        .filter(|(_, _, e)| e.selected)
        .collect();

    let mut sorted_events: Vec<&(usize, usize, GridEvent)> = events
        .iter()
        .filter(|(_, _, e)| !e.selected)
        .collect();

    sorted_events.extend_from_slice(&selected_events);

    Primitive::Group {
        primitives: sorted_events.iter()
            .map(|(step, track, grid_event)| {
                let event_position = get_event_absolute_position(*step, *track, grid_event.offset, normalized_bounds);
                let step_position = get_event_absolute_position(*step, *track, 0., normalized_bounds);
                let event_offset_y = CONTAINER_PADDING_TOP + (*track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM));

                let mut primitives: Vec<Primitive> = vec![];
                
                if grid_event.selected {
                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: event_position.x + bounds.x,
                            y: event_offset_y + bounds.y,
                            width: step_size.width,
                            height: step_size.height,
                        },
                        background: Background::Color(style.event_selected_border_color),
                        border_radius: 0.,
                        border_width: 0.,
                        border_color: Color::TRANSPARENT
                    });

                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: event_position.x + bounds.x + 3.,
                            y: event_offset_y + bounds.y + 3.,
                            width: step_size.width - 6.,
                            height: step_size.height - 6.,
                        },
                        background: Background::Color(lighten(*style.event_bg_color.get(track).unwrap(), 0.2)),
                        border_radius: 0.,
                        border_width: 1.,
                        border_color: Color::WHITE
                    });

                    let slider_inner_height = step_size.height - 8.;
                    let velocity_height = (slider_inner_height * grid_event.velocity).ceil();
                    let velocity_top_offset = slider_inner_height - velocity_height;

                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: event_position.x + bounds.x + 4.,
                            y: event_offset_y + bounds.y + 4. + velocity_top_offset,
                            width: step_size.width - 8.,
                            height: velocity_height,
                        },
                        background: Background::Color(*style.event_bg_color.get(track).unwrap()),
                        border_radius: 0.,
                        border_width: 0.,
                        border_color: Color::TRANSPARENT
                    })

                } else {
                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: event_position.x + bounds.x,
                            y: event_offset_y + bounds.y,
                            width: step_size.width,
                            height: step_size.height,
                        },
                        background: Background::Color(lighten(*style.event_bg_color.get(track).unwrap(), 0.2)),
                        border_radius: 0.,
                        border_width: 1.,
                        border_color: style.event_border_color
                    });

                    let slider_inner_height = step_size.height - 2.;
                    let velocity_height = (slider_inner_height * grid_event.velocity).ceil();
                    let velocity_top_offset = slider_inner_height - velocity_height;

                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: event_position.x + bounds.x + 1.,
                            y: event_offset_y + bounds.y + 1. + velocity_top_offset,
                            width: step_size.width - 2.,
                            height: velocity_height,
                        },
                        background: Background::Color(*style.event_bg_color.get(track).unwrap()),
                        border_radius: 0.,
                        border_width: 0.,
                        border_color: Color::TRANSPARENT
                    })
                }

                if grid_event.offset > 0. {
                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: 1. + step_position.x + bounds.x,
                            y: event_offset_y + bounds.y + step_size.height,
                            width: event_position.x - step_position.x - 1.,
                            height: 2.,
                        },
                        background:  Background::Color(style.event_marker_color.0),
                        border_radius: 0.0,
                        border_width: 0.,
                        border_color: Color::TRANSPARENT
                    })
                } else if grid_event.offset < 0. {
                    primitives.push(Primitive::Quad {
                        bounds: Rectangle{
                            x: event_position.x + bounds.x,
                            y: event_offset_y + bounds.y + step_size.height,
                            width: step_position.x - event_position.x,
                            height: 2.,
                        },
                        background: Background::Color(style.event_marker_color.1),
                        border_radius: 0.0,
                        border_width: 0.,
                        border_color: Color::TRANSPARENT
                    })
                }

                Primitive::Group { primitives }
            })
            .collect()
    }
}
