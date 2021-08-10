use iced_graphics::{Backend, Primitive, Renderer, Color, defaults};
use iced_native::{container, mouse};
use iced_native::{Element, Layout, Point, Rectangle, Vector, Background};

use crate::native::h_list;

pub use crate::native::h_list::{
  State, Content, TitleBar, DragEvent
};
pub use crate::style::h_list::{Style, StyleSheet};

pub type HList<'a, Message, Backend> =
    h_list::HList<'a, Message, Renderer<Backend>>;

impl<B> h_list::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[Content<'_, Message, Self>],
        dragging: Option<(usize, Point)>,
        layout: Layout<'_>,
        style_sheet: &<Self as h_list::Renderer>::Style,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let style = style_sheet.default();

        let pane_cursor_position = if dragging.is_some() {
            // TODO: Remove once cursor availability is encoded in the type
            // system
            Point::new(-1.0, -1.0)
        } else {
            cursor_position
        };

        let mut mouse_interaction = mouse::Interaction::default();
        let mut dragged_pane = None;

        let mut panes: Vec<_> = content
            .iter()
            .zip(content_layout.children())
            .enumerate()
            .map(|(id, (pane, layout))| {
                let (primitive, new_mouse_interaction) = pane.draw(
                    self,
                    defaults,
                    layout,
                    pane_cursor_position,
                    viewport,
                );

                if new_mouse_interaction > mouse_interaction {
                    mouse_interaction = new_mouse_interaction;
                }

                if let Some((dragging, origin)) = dragging {
                    if id == dragging {
                        dragged_pane = Some((id, layout, origin));
                    }
                }

                primitive
            })
            .collect();

        let mut primitives = if let Some((index, layout, origin)) = dragged_pane
        {
            let pane = panes.remove(index);
            let bounds = layout.bounds();

            // TODO: Fix once proper layering is implemented.
            // This is a pretty hacky way to achieve layering.
            let clip = Primitive::Clip {
                bounds: Rectangle {
                    x: cursor_position.x - origin.x,
                    y: bounds.y,
                    width: bounds.width + 0.5,
                    height: bounds.height + 0.5,
                },
                offset: Vector::new(0, 0),
                content: Box::new(Primitive::Translate {
                    translation: Vector::new(
                        cursor_position.x - bounds.x - origin.x,
                        0.,
                    ),
                    content: Box::new(pane),
                }),
            };

            panes.push(clip);

            panes
        } else {
            panes
        };

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

        (
            Primitive::Group { primitives },
            if dragging.is_some() {
                mouse::Interaction::Grabbing
            } else {
                mouse_interaction
            },
        )
    }

    fn draw_pane<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style_sheet: &<Self as container::Renderer>::Style,
        title_bar: Option<(&TitleBar<'_, Message, Self>, Layout<'_>)>,
        body: (&Element<'_, Message, Self>, Layout<'_>),
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output {
        let style = style_sheet.style();
        let (body, body_layout) = body;

        let (body_primitive, body_interaction) =
            body.draw(self, defaults, body_layout, cursor_position, viewport);

        let background = background(bounds, &style);

        if let Some((title_bar, title_bar_layout)) = title_bar {
            let show_controls = bounds.contains(cursor_position);
            let is_over_pick_area =
                title_bar.is_over_pick_area(title_bar_layout, cursor_position);

            let (title_bar_primitive, title_bar_interaction) = title_bar.draw(
                self,
                defaults,
                title_bar_layout,
                cursor_position,
                viewport,
                show_controls,
            );

            (
                Primitive::Group {
                    primitives: vec![
                        background.unwrap_or(Primitive::None),
                        title_bar_primitive,
                        body_primitive,
                    ],
                },
                if is_over_pick_area {
                    mouse::Interaction::Grab
                } else if title_bar_interaction > body_interaction {
                    title_bar_interaction
                } else {
                    body_interaction
                },
            )
        } else {
            (
                if let Some(background) = background {
                    Primitive::Group {
                        primitives: vec![background, body_primitive],
                    }
                } else {
                    body_primitive
                },
                body_interaction,
            )
        }
    }

    fn draw_title_bar<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style_sheet: &<Self as container::Renderer>::Style,
        content: (&Element<'_, Message, Self>, Layout<'_>),
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output {
        let style = style_sheet.style();
        let (title_content, title_layout) = content;

        let defaults = Self::Defaults {
            text: defaults::Text {
                color: style.text_color.unwrap_or(defaults.text.color),
            },
        };

        let background = background(bounds, &style);

        let (title_primitive, title_interaction) = title_content.draw(
            self,
            &defaults,
            title_layout,
            cursor_position,
            viewport,
        );

        if let Some((controls, controls_layout)) = controls {
            let (controls_primitive, controls_interaction) = controls.draw(
                self,
                &defaults,
                controls_layout,
                cursor_position,
                viewport,
            );

            (
                Primitive::Group {
                    primitives: vec![
                        background.unwrap_or(Primitive::None),
                        title_primitive,
                        controls_primitive,
                    ],
                },
                controls_interaction.max(title_interaction),
            )
        } else {
            (
                if let Some(background) = background {
                    Primitive::Group {
                        primitives: vec![background, title_primitive],
                    }
                } else {
                    title_primitive
                },
                title_interaction,
            )
        }
    }
}

fn background(
    bounds: Rectangle,
    style: &iced_style::container::Style,
) -> Option<Primitive> {
    if style.background.is_some() || style.border_width > 0.0 {
        Some(Primitive::Quad {
            bounds,
            background: style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        })
    } else {
        None
    }
}