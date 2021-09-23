use crate::{core::grid::GridEvent, native::grid};
use iced_graphics::canvas::{Cache, Frame, Geometry, LineCap, Path, Stroke};
use iced_graphics::{Backend, Primitive, Renderer};

use crate::core::grid::{get_event_bounds, get_step_dimensions, GridPattern, TRACK_MARGIN_BOTTOM};
use iced_native::{mouse, Background, Color, Point, Rectangle, Size, Vector};

pub use crate::native::grid::State;
pub use crate::style::color_utils::{darken, lighten};
pub use crate::style::grid::{Style, StyleSheet};

use ganic_no_std::{NUM_PERCS, NUM_STEPS};

pub type Grid<'a, Message, Backend> = grid::Grid<'a, Message, Renderer<Backend>>;

impl<B: Backend> grid::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        drawable_area: Rectangle,
        _cursor_position: Point,
        grid_pattern: &GridPattern,
        selection: Option<Rectangle>,
        mouse_interaction: mouse::Interaction,
        is_playing: bool,
        highlight: [usize; NUM_PERCS],
        style_sheet: &Self::Style,
        grid_cache: &Cache,
        event_cache: &Cache,
        highlight_cache: &Cache,
    ) -> Self::Output {
        let style = style_sheet.default();
        let step_size = get_step_dimensions(drawable_area.size());

        let grid = grid_cache.draw(drawable_area.size(), |frame| {
            draw_grid(
                frame,
                drawable_area.size(),
                step_size,
                is_playing,
                highlight,
                &style,
            )
        });
        let steps = event_cache.draw(drawable_area.size(), |frame| {
            draw_steps(
                frame,
                drawable_area.size(),
                grid_pattern,
                step_size,
                is_playing,
                highlight,
                &style,
            )
        });

        let mut primitives = vec![grid.into_primitive(), steps.into_primitive()];

        match selection {
            Some(selection) => {
                primitives.push(draw_selection(selection, drawable_area, &style));
            }
            None => {}
        }

        (
            Primitive::Translate {
                translation: Vector::new(drawable_area.x, drawable_area.y),
                content: Box::new(Primitive::Group { primitives }),
            },
            mouse_interaction,
        )
    }
}

fn draw_selection(selection: Rectangle, bounds: Rectangle, style: &Style) -> Primitive {
    let mut frame = Frame::new(bounds.size());

    let top_left = Point {
        x: selection.x,
        y:  selection.y
    };

    let area = Path::rectangle(top_left, selection.size());

    frame.stroke(
        &area,
        Stroke {
            width: 1.,
            color: style.selection_border_color,
            line_cap: LineCap::Square,
            ..Stroke::default()
        },
    );

    Geometry::into_primitive(frame.into_geometry())
}

fn draw_grid(
    frame: &mut Frame,
    size: Size,
    step_size: Size,
    is_playing: bool,
    highlight: [usize; NUM_PERCS],
    style: &Style,
) {
    for track in 0..NUM_PERCS {
        let track_origin = Point {
            x: 0.,
            y: track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM),
        };
        let track_size = Size {
            width: size.width,
            height: step_size.height,
        };

        let track_bg = Path::rectangle(track_origin, track_size);

        frame.fill(&track_bg, style.step_bg_color_2);

        let offset_origin = Point {
            x: 0.,
            y: track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM) + step_size.height as f32,
        };

        let offset_size = Size {
            width: size.width,
            height: TRACK_MARGIN_BOTTOM,
        };

        let offset_bg = Path::rectangle(offset_origin, offset_size);

        frame.fill(&offset_bg, style.step_bg_color);
    }

    for step in 0..NUM_STEPS + 1 {
        let step_offset_x = (step + 1) as f32 * step_size.width;

        let step_line = Path::line(
            Point {
                x: step_offset_x,
                y: 0.,
            },
            Point {
                x: step_offset_x,
                y: size.height,
            },
        );

        frame.stroke(
            &step_line,
            Stroke {
                width: 1.,
                color: style.step_border_left_color_2,
                line_cap: LineCap::Square,
                ..Stroke::default()
            },
        );
    }
}

fn draw_steps(
    frame: &mut Frame,
    size: Size,
    grid_pattern: &GridPattern,
    step_size: Size,
    is_playing: bool,
    highlight: [usize; NUM_PERCS],
    style: &Style,
) {
    let mut events: Vec<(usize, usize, GridEvent)> = grid_pattern
        .data
        .iter()
        .map(|((step, track), grid_event)| (*step, *track, *grid_event))
        .collect();

    events.sort_by(|x, y| {
        if x.1 == y.1 {
            return x.0.cmp(&y.0);
        }
        x.1.cmp(&y.1)
    });

    let selected_events: Vec<&(usize, usize, GridEvent)> =
        events.iter().filter(|(_, _, e)| e.selected).collect();

    let mut sorted_events: Vec<&(usize, usize, GridEvent)> =
        events.iter().filter(|(_, _, e)| !e.selected).collect();

    sorted_events.extend_from_slice(&selected_events);

    sorted_events.iter().for_each(|(step, track, grid_event)| {
        let event_bounds = get_event_bounds(*step, *track, grid_event.offset, size);
        let step_position = get_event_bounds(*step, *track, 0., size);

        let bg_color = {
            if highlight[*track] == *step && is_playing {
                *style.event_highlight_bg_color.get(track).unwrap()
            } else {
                *style.event_bg_color.get(track).unwrap()
            }
        };

        if grid_event.selected {
            let selected_countour = Path::rectangle(
                Point {
                    x: event_bounds.x,
                    y: event_bounds.y,
                },
                step_size,
            );
            frame.fill(&selected_countour, style.event_selected_border_color);

            let event = Path::rectangle(
                Point {
                    x: event_bounds.x + 2.,
                    y: event_bounds.y + 2.,
                },
                Size {
                    width: step_size.width - 4.,
                    height: step_size.height - 4.,
                },
            );
            frame.fill(&event, lighten(bg_color, 0.2));
            frame.stroke(
                &event,
                Stroke {
                    width: 1.,
                    color: Color::from_rgb(0.36, 0.36, 0.3),
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                },
            );

            let slider_inner_height = step_size.height - 6.;
            let velocity_height = (slider_inner_height * grid_event.velocity).ceil();
            let velocity_top_offset = slider_inner_height - velocity_height;

            let inner_slider = Path::rectangle(
                Point {
                    x: event_bounds.x + 3.,
                    y: event_bounds.y + 3. + velocity_top_offset,
                },
                Size {
                    width: step_size.width - 6.,
                    height: velocity_height,
                },
            );
            frame.fill(&inner_slider, darken(bg_color, 0.1));
        } else {
            let event = Path::rectangle(
                Point {
                    x: event_bounds.x,
                    y: event_bounds.y,
                },
                step_size,
            );
            frame.fill(&event, lighten(bg_color, 0.2));
            frame.stroke(
                &event,
                Stroke {
                    width: 1.,
                    color: style.event_border_color,
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                },
            );

            let slider_inner_height = step_size.height - 2.;
            let velocity_height = (slider_inner_height * grid_event.velocity).ceil();
            let velocity_top_offset = slider_inner_height - velocity_height;

            let inner_slider = Path::rectangle(
                Point {
                    x: event_bounds.x + 1.,
                    y: event_bounds.y + 1. + velocity_top_offset,
                },
                Size {
                    width: step_size.width - 2.,
                    height: velocity_height,
                },
            );
            frame.fill(&inner_slider, darken(bg_color, 0.1));
        }

        if grid_event.offset > 0. {
            let offset = Path::rectangle(
                Point {
                    x: step_position.x,
                    y: event_bounds.y + step_size.height,
                },
                Size {
                    width: event_bounds.x - step_position.x - 1.,
                    height: 2.,
                },
            );
            frame.fill(&offset, style.event_marker_color.0);
        } else if grid_event.offset < 0. {
            let offset = Path::rectangle(
                Point {
                    x: event_bounds.x,
                    y: event_bounds.y + step_size.height,
                },
                Size {
                    width: step_position.x - event_bounds.x,
                    height: 2.,
                },
            );
            frame.fill(&offset, style.event_marker_color.1);
        }
    });
}
