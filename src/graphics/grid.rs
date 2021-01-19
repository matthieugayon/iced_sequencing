//! Display a ramp control that controls a [`Param`]. It is usually used to
//! represent the easing of a parameter between two points in time.
//!
//! [`Param`]: ../core/param/trait.Param.html

use crate::native::grid;
use iced_graphics::canvas::{Frame, LineCap, Path, Stroke};
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::{mouse, Background, Point, Rectangle, Size, Vector};

pub use crate::native::grid::{State};
pub use crate::style::grid::{Style, StyleSheet};

use ganic_no_std::{pattern::Pattern};


/// A ramp GUI widget that controls a [`Param`]. It is usually used to
/// represent the easing of a parameter between two points in time.
///
/// [`Param`]: ../../core/param/trait.Param.html
/// [`Ramp`]: struct.Ramp.html
pub type Ramp<'a, Message, Backend> =
    grid::Grid<'a, Message, Renderer<Backend>>;

impl<B: Backend> grid::Renderer for Renderer<B> {
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        pattern: Pattern,
        is_dragging: bool,
        style_sheet: &Self::Style
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let style = if is_dragging {
            style_sheet.dragging()
        } else if is_mouse_over {
            style_sheet.hovered()
        } else {
            style_sheet.active()
        };

        let bounds_x = bounds.x.floor();
        let bounds_y = bounds.y.floor();

        let bounds_width = bounds.width.floor();
        let bounds_height = bounds.height.floor();

        let back = Primitive::Quad {
            bounds: Rectangle {
                x: bounds_x,
                y: bounds_y,
                width: bounds_width,
                height: bounds_height,
            },
            background: Background::Color(style.back_color),
            border_radius: 0.0,
            border_width: style.back_border_width,
            border_color: style.back_border_color,
        };

        let border_width = style.back_border_width as f32;
        let twice_border_width = border_width * 2.0;

        let range_width = bounds_width - twice_border_width;
        let range_height = bounds_height - twice_border_width;

        let line: Primitive = {
                let stroke = Stroke {
                    width: style.line_width as f32,
                    color: style.line_up_color,
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                };

                let control = Point::new(
                    range_width * 0.5,
                    -range_height,
                );
                let to = Point::new(range_width, -range_height);

                let path = Path::new(|p| {
                    p.move_to(to);
                    p.quadratic_curve_to(control, Point::ORIGIN)
                });

                let mut frame =
                    Frame::new(Size::new(range_width, range_height));

                frame.translate(Vector::new(0.0, range_height));

                frame.stroke(&path, stroke);

                Primitive::Translate {
                    translation: Vector::new(
                        bounds_x + border_width,
                        bounds_y + border_width,
                    ),
                    content: Box::new(
                        frame.into_geometry().into_primitive(),
                    ),
                }
            };

        (
            Primitive::Group {
                primitives: vec![back, line],
            },
            mouse::Interaction::default(),
        )
    }
}
