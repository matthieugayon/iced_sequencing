use iced_native::{Rectangle, mouse, Background, Color};
use iced_graphics::{Backend, Renderer, Primitive};

use crate::native::multi_slider;
use crate::native::multi_slider::Slider;
pub use crate::native::multi_slider::State;

/// A vertical Multislider GUI widget.
///
/// A [`Multislider`] will try to fill the horizontal and vertical dimensions of its container.
///
/// [`MultiSlider`]: struct.MultiSlider.html
pub type MultiSlider<'a, Message> =
multi_slider::MultiSlider<'a, Message>;

impl<B: Backend> multi_slider::Renderer for Renderer<B> {
    fn draw(
        &mut self,
        bounds: Rectangle,
        background_color: Color,
        sliders: &Vec<Slider>,
        focused: bool
    ) -> Self::Output {
        let background = draw_background(bounds, background_color, focused);
        let sliders = draw_sliders(bounds, sliders, focused);
        let primitives = vec![background, sliders];

        (Primitive::Group { primitives }, mouse::Interaction::default())
    }
}

fn draw_background(bounds: Rectangle, background_color: Color, focused: bool) -> Primitive {
    let alpha = if focused { 1.0 } else { 0.66 };

    Primitive::Quad {
        bounds,
        background: Background::Color(background_color),
        border_radius: 0.0,
        border_width: 1.0,
        border_color: Color::from_rgba(0.0, 0.0, 0.0, alpha),
    }
}

fn draw_sliders(bounds: Rectangle, sliders: &Vec<Slider>, focused: bool) -> Primitive {
    Primitive::Group {
        primitives: sliders
            .iter()
            .map(|slider| {
                let alpha = if slider.hovered && focused { 1.0 } else { 0.66 };

                Primitive::Quad {
                    bounds: Rectangle{
                        x: slider.origin.x + bounds.x,
                        y: slider.origin.y + bounds.y,
                        width: slider.size.width,
                        height: slider.size.height,
                    },
                    background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, alpha)),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::from_rgba(0.9, 0.0, 0.0, 1.0)
                }
            })
            .collect()
    }
}
