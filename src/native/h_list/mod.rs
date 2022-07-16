mod content;
mod draggable;
mod state;

pub use content::Content;
pub use draggable::Draggable;
pub use state::State;

use iced_native::{
    event, layout, mouse, overlay, renderer, touch, Clipboard, Element, Event, Layout, Length,
    Point, Rectangle, Shell, Size, Vector, Widget,
};

pub use crate::style::h_list::{Style, StyleSheet};

#[allow(missing_debug_implementations)]
pub struct HList<'a, Message, Renderer: iced_native::Renderer> {
    action: &'a mut state::Action,
    elements: Vec<Content<'a, Message, Renderer>>,
    size: usize,
    width: Length,
    height: Length,
    spacing: u16,
    on_click: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer: iced_native::Renderer> HList<'a, Message, Renderer> {
    pub fn new<T>(
        state: &'a mut State<T>,
        view: impl Fn(usize, &'a mut T) -> Content<'a, Message, Renderer>,
    ) -> Self {
        let elements = {
            state
                .panes
                .iter_mut()
                .enumerate()
                .map(|(pane_index, pane_state)| view(pane_index, pane_state))
                .collect()
        };

        Self {
            action: &mut state.action,
            elements,
            width: Length::Fill,
            height: Length::Fill,
            size: 16,
            spacing: 0,
            on_click: None,
            on_drag: None,
            style_sheet: Default::default(),
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    pub fn size(mut self, number_of_panes: usize) -> Self {
        self.size = number_of_panes;
        self
    }

    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(usize) -> Message,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn on_drag<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(DragEvent) -> Message,
    {
        self.on_drag = Some(Box::new(f));
        self
    }

    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet + 'a>>) -> Self {
        self.style_sheet = style.into();
        self
    }
}

pub fn update<'a, Message, T: Draggable>(
    action: &mut state::Action,
    event: &Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    spacing: u16,
    elements: impl Iterator<Item = (usize, T)>,
    on_click: &Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_drag: &Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
) -> event::Status {
    let mut event_status = event::Status::Ignored;

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let bounds = layout.bounds();

            if bounds.contains(cursor_position) {
                event_status = event::Status::Captured;

                click_pane(
                    action,
                    layout,
                    cursor_position,
                    shell,
                    elements,
                    on_click,
                    on_drag,
                );
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            if let Some((pane, _)) = action.picked_pane() {
                if let Some(on_drag) = on_drag {
                    let mut dropped_region = elements
                        .zip(layout.children())
                        .filter(|(_, layout)| layout.bounds().contains(cursor_position));

                    let event = match dropped_region.next() {
                        Some(((target, _), _)) if pane != target => {
                            DragEvent::Dropped { pane, target }
                        }
                        _ => DragEvent::Canceled { pane },
                    };

                    shell.publish(on_drag(event));
                }

                *action = state::Action::Idle;

                event_status = event::Status::Captured;
            }
        }
        _ => {}
    }

    event_status
}

fn click_pane<'a, Message, T>(
    action: &mut state::Action,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    elements: impl Iterator<Item = (usize, T)>,
    on_click: &Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_drag: &Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
) where
    T: Draggable,
{
    let mut clicked_region = elements
        .zip(layout.children())
        .filter(|(_, layout)| layout.bounds().contains(cursor_position));

    if let Some(((id, content), layout)) = clicked_region.next() {
        if let Some(on_click) = &on_click {
            shell.publish(on_click(id));
        }

        if let Some(on_drag) = &on_drag {
            if content.can_be_dragged_at(layout, cursor_position) {
                let pane_position = layout.position();

                let origin = cursor_position - Vector::new(pane_position.x, pane_position.y);

                *action = state::Action::Dragging { index: id, origin };

                shell.publish(on_drag(DragEvent::Picked { pane: id }));
            }
        }
    }
}

pub fn draw<Renderer, T>(
    action: &state::Action,
    layout: Layout<'_>,
    cursor_position: Point,
    renderer: &mut Renderer,
    style: &renderer::Style,
    viewport: &Rectangle,
    spacing: u16,
    resize_leeway: Option<u16>,
    style_sheet: &dyn StyleSheet,
    elements: impl Iterator<Item = (usize, T)>,
    draw_pane: impl Fn(T, &mut Renderer, &renderer::Style, Layout<'_>, Point, &Rectangle),
) where
    Renderer: iced_native::Renderer,
{
    let picked_pane = action.picked_pane();

    let pane_cursor_position = if picked_pane.is_some() {
        // TODO: Remove once cursor availability is encoded in the type
        // system
        Point::new(-1.0, -1.0)
    } else {
        cursor_position
    };

    for ((id, pane), layout) in elements.zip(layout.children()) {
        match picked_pane {
            Some((dragging, origin)) if id == dragging => {
                let bounds = layout.bounds();

                // translate the pane along the x-axis only
                let mut t = cursor_position - Point::new(bounds.x + origin.x, bounds.y + origin.y);
                t.y = 0.0;

                renderer.with_translation(t, |renderer| {
                    renderer.with_layer(bounds, |renderer| {
                        draw_pane(
                            pane,
                            renderer,
                            style,
                            layout,
                            pane_cursor_position,
                            viewport,
                        );
                    });
                });
            }
            _ => {
                draw_pane(
                    pane,
                    renderer,
                    style,
                    layout,
                    pane_cursor_position,
                    viewport,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DragEvent {
    Picked { pane: usize },
    Dropped { pane: usize, target: usize },
    Canceled { pane: usize },
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for HList<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);
        let number_of_elements = self.elements.len();

        let children = self
            .elements
            .iter()
            .enumerate()
            .filter_map(|(pane, element)| {
                let area_width = size.width / number_of_elements as f32;
                let region = Rectangle {
                    x: ((pane as f32 * area_width) + self.spacing as f32).round(),
                    y: 0.,
                    width: area_width.round() - 2. * self.spacing as f32,
                    height: size.height,
                };

                let size = Size::new(region.width, region.height);
                let mut node = element.layout(renderer, &layout::Limits::new(size, size));
                node.move_to(Point::new(region.x, region.y));
                Some(node)
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let event_status = update(
            self.action,
            &event,
            layout,
            cursor_position,
            shell,
            self.spacing,
            self.elements
                .iter()
                .enumerate()
                .map(|(idx, content)| (idx, content)),
            &self.on_click,
            &self.on_drag,
        );

        let picked_pane = self.action.picked_pane().map(|(pane, _)| pane);

        self.elements
            .iter_mut()
            .enumerate()
            .zip(layout.children())
            .map(|((pane, content), layout)| {
                let is_picked = picked_pane == Some(pane);

                content.on_event(
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                    is_picked,
                )
            })
            .fold(event_status, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.action.picked_pane().is_some() {
            return mouse::Interaction::Grabbing;
        } else {
            self.elements
                .iter()
                .enumerate()
                .zip(layout.children())
                .map(|((_pane, content), layout)| {
                    content.mouse_interaction(layout, cursor_position, viewport, renderer)
                })
                .max()
                .unwrap_or_default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        draw(
            self.action,
            layout,
            cursor_position,
            renderer,
            style,
            viewport,
            self.spacing,
            None,
            self.style_sheet.as_ref(),
            self.elements
                .iter()
                .enumerate()
                .map(|(pane, content)| (pane, content)),
            |pane, renderer, style, layout, cursor_position, rectangle| {
                pane.draw(renderer, style, layout, cursor_position, rectangle);
            },
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.elements
            .iter_mut()
            .enumerate()
            .zip(layout.children())
            .find_map(|((_, pane), layout)| pane.overlay(layout, renderer))
    }
}

impl<'a, Message, Renderer> From<HList<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Message: 'a,
{
    fn from(hlist: HList<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(hlist)
    }
}
