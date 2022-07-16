use super::Draggable;
use iced_native::{
    event, layout, mouse, overlay, widget::container, Clipboard, Element, Event, Layout, Point,
    Rectangle, Shell, Size,
};

pub struct Content<'a, Message, Renderer> {
    body: Element<'a, Message, Renderer>,
    controls: Option<Element<'a, Message, Renderer>>,
    style_sheet: Box<dyn container::StyleSheet + 'a>,
    always_show_controls: bool
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    pub fn new(body: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            body: body.into(),
            controls: None,
            style_sheet: Default::default(),
            always_show_controls: false
        }
    }

    pub fn style(mut self, style_sheet: impl Into<Box<dyn container::StyleSheet + 'a>>) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }

    pub fn controls(mut self, controls: impl Into<Element<'a, Message, Renderer>>) -> Self {
        self.controls = Some(controls.into());
        self
    }

    pub fn always_show_controls(mut self) -> Self {
        self.always_show_controls = true;
        self
    }
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        style: &iced_native::renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        {
            let style = self.style_sheet.style();
            container::draw_background(renderer, &style, bounds);
        }

        if let Some(controls) = &self.controls {
            let mut children = layout.children();
            let body_layout = children.next().unwrap();
            let controls_layout = children.next().unwrap();

            self.body
                .draw(renderer, style, body_layout, cursor_position, viewport);

            if bounds.contains(cursor_position) || self.always_show_controls {
                controls.draw(
                    renderer,
                    &style,
                    controls_layout,
                    cursor_position,
                    viewport
                );
            }
        } else {
            self.body
                .draw(renderer, style, layout, cursor_position, viewport);
        }
    }

    pub(crate) fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let body_layout = self.body.layout(renderer, limits);

        if let Some(controls) = &self.controls {
            let max_size = limits.max();
            let mut controls_layout = controls
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let controls_size = controls_layout.size();
            let space_before_controls = max_size.width - controls_size.width;

            let height = body_layout.size().height.max(controls_size.height);

            controls_layout.move_to(Point::new(space_before_controls, 0.0));

            layout::Node::with_children(
                Size::new(max_size.width, height),
                vec![body_layout, controls_layout],
            )
        } else {
            self.body.layout(renderer, limits)
        }
    }

    pub(crate) fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        is_picked: bool,
    ) -> event::Status {
        let mut event_status = event::Status::Ignored;

        let body_layout = if let Some(controls) = &mut self.controls {
            let mut children = layout.children();
            let body_layout = children.next().unwrap();
            let controls_layout = children.next().unwrap();

            event_status = controls.on_event(
                event.clone(),
                controls_layout,
                cursor_position,
                renderer,
                clipboard,
                shell
            );

            body_layout
        } else {
            layout
        };

        let body_status = if is_picked {
            event::Status::Ignored
        } else {
            self.body.on_event(
                event,
                body_layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        };

        event_status.merge(body_status)
    }

    pub(crate) fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut mouse_interaction = mouse::Interaction::default();
        let body_layout = if self.controls.is_some() {
            let mut children = layout.children();
            children.next().unwrap()
        } else {
            layout
        };

        if body_layout.bounds().contains(cursor_position) {
            mouse_interaction = mouse::Interaction::Grab;
        }

        if let Some(controls) = &self.controls {
            let mut children = layout.children();
            children.next();
            let controls_layout = children.next().unwrap();
            let hover_controls = controls_layout.bounds().contains(cursor_position);

            if hover_controls {
                mouse_interaction = controls.mouse_interaction(controls_layout, cursor_position, viewport, renderer);
            }
        }

        self.body
            .mouse_interaction(body_layout, cursor_position, viewport, renderer)
            .max(mouse_interaction)
    }

    pub(crate) fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        if let Some(controls) = self.controls.as_mut() {
            let mut children = layout.children();
            children.next()?;
            let controls_layout = children.next()?;
            controls.overlay(controls_layout, renderer)
        } else {
            self.body.overlay(layout, renderer)
        }
    }
}

impl<'a, Message, Renderer> Draggable for &Content<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn can_be_dragged_at(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        if self.controls.is_some() {
            let mut children = layout.children();
            let body_layout = children.next().unwrap();
            let controls_layout = children.next().unwrap();

            let hover_body = body_layout.bounds().contains(cursor_position);
            let hover_controls = controls_layout.bounds().contains(cursor_position);

            if hover_body && !hover_controls {
                return true;
            }
        } else {
            if layout.bounds().contains(cursor_position) {
                return true;
            }
        };
        return false;
    }
}

impl<'a, T, Message, Renderer> From<T> for Content<'a, Message, Renderer>
where
    T: Into<Element<'a, Message, Renderer>>,
    Renderer: iced_native::Renderer,
{
    fn from(element: T) -> Self {
        Self::new(element)
    }
}
