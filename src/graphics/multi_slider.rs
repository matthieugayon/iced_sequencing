use iced_native::{Rectangle, mouse, Background, Color, Point, Padding};
use iced_graphics::{Backend, Renderer, Primitive};

use crate::native::multi_slider;
pub use crate::native::multi_slider::State;

pub use crate::style::multi_slider::{Style, StyleSheet, Slider};

pub type MultiSlider<'a, T, Message, Backend> =
    multi_slider::MultiSlider<'a, T, Message, Renderer<Backend>>;

impl<B> multi_slider::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        cursor_position: Point,
        range: std::ops::RangeInclusive<f32>,
        values: Vec<f32>,
        active: Option<usize>,
        is_dragging: bool,
        spacing: u16,
        base_color: Color,
        style_sheet: &Self::Style
    ) -> Self::Output {
        let style = style_sheet.default(base_color);
        let highlight_slider_style = style_sheet.highlight(base_color);
        let hovered_slider_style = style_sheet.hovered(base_color);
        let slider_width = (content_bounds.width / values.len() as f32).floor();
        
        let mut primitives = vec![];

        if style.background.is_some() || style.border_width > 0.0 {
            primitives.push(Primitive::Quad {
                bounds,
                background: style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
                border_radius: style.border_radius,
                border_width: style.border_width,
                border_color: style.border_color,
            });
        }

        primitives.push(
            Primitive::Group {
                primitives: values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        let slider_height = content_bounds.height * *value;
                        let slider_range_bounds = Rectangle {
                            x: content_bounds.x + index as f32 * slider_width + spacing as f32,
                            y: content_bounds.y,
                            width: slider_width - 2. * spacing as f32,
                            height: content_bounds.height,
                        };
                        let slider_bounds = Rectangle {
                            x: content_bounds.x + index as f32 * slider_width + spacing as f32,
                            y: content_bounds.y + content_bounds.height - slider_height,
                            width: slider_width - 2. * spacing as f32,
                            height: slider_height,
                        };

                        let slider_style = match active {
                            Some(active_slider) => {
                                if slider_range_bounds.contains(cursor_position) {
                                    hovered_slider_style
                                } else if active_slider == index {
                                    highlight_slider_style
                                } else {
                                    style.slider
                                }
                            },
                            None => {
                                if slider_range_bounds.contains(cursor_position) {
                                    hovered_slider_style
                                } else {
                                    style.slider
                                }
                            },
                        };

                        let slider_bar = Primitive::Quad {
                            bounds: slider_bounds,
                            background: Background::Color(slider_style.color),
                            border_radius: 0.0,
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT
                        };

                        let slider_primitives = vec![slider_bar];

                        // if slider_style.marker_height > 0. {

                        // }
        
                        Primitive::Group { primitives: slider_primitives }
                    })
                    .collect()
            }
        );

        (Primitive::Group { primitives }, mouse::Interaction::default())
    }
}