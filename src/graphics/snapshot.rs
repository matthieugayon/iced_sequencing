use iced_native::{ 
    mouse, Rectangle, Background, Color, 
    Element, Layout, Point, Size
};
use iced_graphics::{Backend, Primitive, Renderer, defaults};

use crate::native::snapshot;
pub use crate::style::snapshot::{Style, StyleSheet, Default};

pub type Snapshot<'a, Message, Backend> =
    snapshot::Snapshot<'a, Message, Renderer<Backend>>;

use ganic_no_std::{NUM_PERCS, NUM_STEPS};
use crate::core::grid::{GridPattern, GridEvent}; 
use crate::core::utils::get_step_dimension; 

impl<B> snapshot::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    fn draw<Message>(
        &mut self,
        bounds: Rectangle,
        pattern: GridPattern,
        selected: bool,
        style: &Self::Style,
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output {    
        let mut events: Vec<(usize, usize, GridEvent)> = pattern.data
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

        let step_bounds = Rectangle { height: bounds.height - 3., y: bounds.y + 2., ..bounds };
        let step_dim: Size = get_step_dimension(step_bounds, NUM_STEPS + 2, NUM_PERCS);
        let step_width = 0.85 * step_dim.width;
        let step_height = (step_dim.height - 1.).floor();
        let style: Style = if selected { style.selected() } else { style.default() };
        let division: usize = {
            if step_dim.width <= 2. {
                8
            } else if step_dim.width <= 3. {
                4
            } else {
                2
            }
        };
        let grid = NUM_STEPS / division;

        let mut primitives: Vec<Primitive> = (0..=grid).into_iter().map(|step| {
            let color = {
                if step == 0 || step == grid {
                    style.line_edge_color
                } else {
                    style.line_division_color
                }
            };

            Primitive::Quad {
                bounds: Rectangle{
                    x: (bounds.x + ((division * step + 1) as f32 * step_dim.width)).round(),
                    y: bounds.y + 1.,
                    width: 1.,
                    height: bounds.height - 2.
                },
                background: Background::Color(color),
                border_radius: 0.,
                border_width: 0.,
                border_color: Color::TRANSPARENT
            }
        }).collect();

        events.iter().for_each(|(step, track, grid_event)| {
            primitives.push(Primitive::Quad {
                bounds: Rectangle{
                    x: (step_bounds.x + (((*step + 1) as f32 + grid_event.offset) * step_dim.width)).round(),
                    y: step_bounds.y + (*track as f32 * step_dim.height),
                    width: step_width,
                    height: step_height 
                },
                background: Background::Color(style.step_color),
                border_radius: 0.,
                border_width: 0.,
                border_color: Color::TRANSPARENT
            });
        });

        if style.background.is_some() || style.border_width > 0.0 {
            primitives.insert(0, Primitive::Quad {
                bounds,
                background: style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
                border_radius: style.border_radius,
                border_width: style.border_width,
                border_color: style.border_color,
            });
        }

        if let Some((controls, controls_layout)) = controls {
            let defaults = Self::Defaults {
                text: defaults::Text {
                    color: Color::BLACK,
                },
            };
            let (controls_primitive, controls_interaction) = controls.draw(
                self,
                &defaults,
                controls_layout,
                cursor_position,
                viewport,
            );

            primitives.push(controls_primitive);

            return (
                Primitive::Group { primitives },
                controls_interaction,
            )
        }

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }
}
