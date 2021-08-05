use iced_native::{ 
    mouse, Rectangle, Background, Color, 
    Element, Layout, Point
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

        let step_dim: Rectangle = get_step_dimension(bounds, NUM_STEPS + 1, NUM_PERCS);
        let offset_x = step_dim.x;
        let offset_y = step_dim.y;

        let style: Style = if selected { style.default() } else { style.selected() };

        let mut primitives: Vec<Primitive> = events.iter().map(|(step, track, grid_event)| {
            Primitive::Quad {
                bounds: Rectangle{
                    x: bounds.x + offset_x + ((*step as f32 + grid_event.offset) * step_dim.width).floor(),
                    y: bounds.y + offset_y + *track as f32 * step_dim.height,
                    width: step_dim.width,
                    height: step_dim.height
                },
                background: Background::Color(style.step_color),
                border_radius: 0.,
                border_width: 0.,
                border_color: Color::TRANSPARENT
            }
        }).collect();

        primitives.insert(0, 
            Primitive::Quad {
                bounds,
                background: Background::Color(Color::BLACK),
                border_radius: 0.,
                border_width: 1.,
                border_color: Color::WHITE
            }
        );

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
