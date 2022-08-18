use crate::{core::grid::GridEvent, native::grid};
use iced_core::mouse;
use iced_graphics::canvas::{Cache, Frame, Geometry, LineCap, Path, Stroke};
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::Background;

use crate::core::grid::{
    get_event_bounds, get_step_dimensions,
    GridPattern, TRACK_MARGIN_BOTTOM
};
use iced_native::{Point, Rectangle, Size, Vector};

pub use crate::native::grid::State;
pub use crate::style::color_utils::{darken, lighten};
pub use crate::style::grid::{Style, StyleSheet, GridColor};

use ganic_no_std::{NUM_PERCS, NUM_STEPS};

pub type Grid<'a, Message, Backend> = grid::Grid<'a, Message, Renderer<Backend>>;

const BEATS: usize = 4;
const BEAT_STEP_COUNT: usize = NUM_STEPS / BEATS;

impl<B: Backend> grid::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        drawable_area: Rectangle,
        _cursor_position: Point,
        grid_pattern: &GridPattern,
        selection: Option<Rectangle>,
        _mouse_interaction: mouse::Interaction,
        is_playing: bool,
        highlight: [usize; NUM_PERCS],
        style_sheet: &Self::Style,
        grid_cache: &Cache,
        event_cache: &Cache,
        _highlight_cache: &Cache,
    ) {
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

        // 1. grid
        let mut canvas_primitives = vec![grid.into_primitive()];

        // 2. highlighted steps
        if is_playing {
            canvas_primitives.push(draw_highlight(drawable_area.size(), highlight, &style));
        }

        // 3. events
        canvas_primitives.push(steps.into_primitive());

        // 4. selection
        match selection {
            Some(selection) => {
                canvas_primitives.push(draw_selection(selection, drawable_area, &style));
            }
            None => {}
        }

        let mut primitives = vec![
            Primitive::Translate {
                translation: Vector::new(drawable_area.x, drawable_area.y),
                content: Box::new(Primitive::Group { primitives: canvas_primitives }),
            }
        ];

        if style.background.is_some() {
            let background = style.background.unwrap();
            primitives.push(Primitive::Quad {
                bounds,
                background: Background::Color(background.bg_color),
                border_radius: background.border_radius,
                border_width: background.border_width,
                border_color: background.border_color,
            });
        }

        // (
        //     Primitive::Group { primitives },
        //     mouse_interaction,
        // )

        self.draw_primitive(Primitive::Group { primitives })
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
            width: style.selection_stroke.line_width,
            color: style.selection_stroke.color,
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
    _is_playing: bool,
    _highlight: [usize; NUM_PERCS],
    style: &Style,
) {
    // backgrounds
    for beat in 0..BEATS {
        // bg color definition
        let bg_color = match beat {
            0 | 2 => style.grid.even_beat_bg_color,
            _ => style.grid.odd_beat_bg_color
        };

        let beat_width = step_size.width * BEAT_STEP_COUNT as f32;
        let beat_origin = Point {
            x: beat as f32 * beat_width + step_size.width,
            y: 0.,
        };
        let beat_size = Size {
            width: beat_width,
            height: size.height,
        };

        let beat_bg = Path::rectangle(beat_origin, beat_size);

        frame.fill(&beat_bg, bg_color);
    }

    // edges bg
    let start_edge = Point { x: 0., y: 0. };
    let start_edge_size = Size { width: step_size.width, height: size.height };
    let start_bg = Path::rectangle(start_edge, start_edge_size);
    frame.fill(&start_bg, style.grid.edge_step_bg_color);

    let end_edge = Point { x: (NUM_STEPS + 1) as f32 * step_size.width, y: 0. };
    let end_edge_size = Size { width: step_size.width, height: size.height };
    let end_bg = Path::rectangle(end_edge, end_edge_size);
    frame.fill(&end_bg, style.grid.edge_step_bg_color);

    // track margins
    for track in 0..NUM_PERCS {
        let track_margin_origin = Point {
            x: 0.,
            y: track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM) + step_size.height as f32,
        };

        let track_margin_size = Size {
            width: size.width,
            height: TRACK_MARGIN_BOTTOM,
        };

        let track_margin_bg = Path::rectangle(track_margin_origin, track_margin_size);

        frame.fill(&track_margin_bg, style.grid.track_margin_color);
    }

    for step in 0..=NUM_STEPS {
        // stroke style definition
        let stroke = match step {
            0 | NUM_STEPS => style.grid.edge_step_line,
            step_index if (step_index / BEAT_STEP_COUNT) % 2 == 1 => style.grid.odd_beat_line,
            _ => style.grid.even_beat_line
        };

        let step_offset_x = (step + 1) as f32 * step_size.width;

        let step_line = Path::line(
            Point {
                x: step_offset_x,
                y: 0.,
            },
            Point {
                x: step_offset_x,
                y: size.height - TRACK_MARGIN_BOTTOM,
            },
        );

        frame.stroke(
            &step_line,
            Stroke {
                width: stroke.line_width,
                color: stroke.color,
                line_cap: LineCap::Square,
                ..Stroke::default()
            },
        );
    }
}

fn draw_highlight(
    size: Size,
    highlight: [usize; NUM_PERCS],
    style: &Style,
) -> Primitive {
    let mut frame = Frame::new(size);

    let highlighted_steps = Path::new(|path| {
        for (track , highlighted_step) in highlight.iter().enumerate() {
            let event_bounds = get_event_bounds(*highlighted_step, track, 0., size);
            path.rectangle(event_bounds.position(), event_bounds.size());
        }

        path.close();
    });

    frame.fill(&highlighted_steps, style.current_step_bg_color);

    Geometry::into_primitive(frame.into_geometry())
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

        // Color definitions

        let event_bg_color = match style.event.bg_color {
            GridColor::Simple(color) => color,
            GridColor::Multitrack(color_array) => color_array[*track],
        };

        let slider_bg_color = {
            if highlight[*track] == *step && is_playing {
                style.event.slider_highlighted_bg_color
            } else {
                style.event.slider_bg_color
            }
        };

        let slider_fill_color = match slider_bg_color {
            GridColor::Simple(color) => color,
            GridColor::Multitrack(color_array) => color_array[*track],
        };

        if grid_event.selected {
            // selected event contour
            let selected_countour = Path::rectangle(
                Point {
                    x: event_bounds.x,
                    y: event_bounds.y,
                },
                step_size,
            );
            frame.fill(&selected_countour, style.event.contour_bg_color);

            // event with fill & stroke
            let event = Path::rectangle(
                Point {
                    x: event_bounds.x + style.event.contour_width,
                    y: event_bounds.y + style.event.contour_width,
                },
                Size {
                    width: step_size.width - (style.event.contour_width * 2.),
                    height: step_size.height - (style.event.contour_width * 2.),
                },
            );
            frame.fill(&event, event_bg_color);

            let slider_inner_height = step_size.height - (style.event.contour_width * 2.);
            let velocity_height = (slider_inner_height * grid_event.velocity).ceil();
            let velocity_top_offset = slider_inner_height - velocity_height;

            // inner slider
            let inner_slider = Path::rectangle(
                Point {
                    x: event_bounds.x + style.event.contour_width,
                    y: event_bounds.y + style.event.contour_width + velocity_top_offset,
                },
                Size {
                    width: step_size.width - (style.event.contour_width * 2.),
                    height: velocity_height,
                },
            );
            frame.fill(&inner_slider, slider_fill_color);
            frame.stroke(
                &event,
                Stroke {
                    width: style.event.stroke.line_width,
                    color: style.event.stroke.color,
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                },
            );
        } else {
            // event with fill & stroke
            let event = Path::rectangle(
                Point {
                    x: event_bounds.x,
                    y: event_bounds.y,
                },
                step_size,
            );
            frame.fill(&event, event_bg_color);

            let slider_inner_height = step_size.height;
            let velocity_height = (slider_inner_height * grid_event.velocity).ceil();
            let velocity_top_offset = slider_inner_height - velocity_height;

            // inner slider
            let inner_slider = Path::rectangle(
                Point {
                    x: event_bounds.x,
                    y: event_bounds.y+ velocity_top_offset,
                },
                Size {
                    width: step_size.width,
                    height: velocity_height,
                },
            );
            frame.fill(&inner_slider, slider_fill_color);
            frame.stroke(
                &event,
                Stroke {
                    width: style.event.stroke.line_width,
                    color: style.event.stroke.color,
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                },
            );
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
            frame.fill(&offset, style.event.positive_offset_marker_bg_color);
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
            frame.fill(&offset, style.event.negative_offset_marker_bg_color);
        }
    });
}