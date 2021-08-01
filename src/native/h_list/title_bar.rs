use super::super::h_list;
use iced_native::{
    container, event, Event, layout, Padding,
    Clipboard, Element, Hasher, Layout, Point, Rectangle, Size
};

#[allow(missing_debug_implementations)]
pub struct TitleBar<'a, Message, Renderer: h_list::Renderer> {
    content: Element<'a, Message, Renderer>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: Padding,
    always_show_controls: bool,
    style: <Renderer as container::Renderer>::Style,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: h_list::Renderer,
{
    pub fn new<E>(content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            content: content.into(),
            controls: None,
            padding: Padding::ZERO,
            always_show_controls: false,
            style: Default::default(),
        }
    }

    pub fn controls(
        mut self,
        controls: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.controls = Some(controls.into());
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn style(
        mut self,
        style: impl Into<<Renderer as container::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    pub fn always_show_controls(mut self) -> Self {
        self.always_show_controls = true;
        self
    }
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: h_list::Renderer,
{
    /// Draws the [`TitleBar`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: crate::widget::pane_grid::Renderer
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        show_controls: bool,
    ) -> Renderer::Output {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();

        let controls = if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();

            if show_controls || self.always_show_controls {
                Some((controls, controls_layout))
            } else {
                None
            }
        } else {
            None
        };

        renderer.draw_title_bar(
            defaults,
            layout.bounds(),
            &self.style,
            (&self.content, title_layout),
            controls,
            cursor_position,
            viewport,
        )
    }

    /// Returns whether the mouse cursor is over the pick area of the
    /// [`TitleBar`] or not.
    ///
    /// The whole [`TitleBar`] is a pick area, except its controls.
    pub fn is_over_pick_area(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> bool {
        if layout.bounds().contains(cursor_position) {
            let mut children = layout.children();
            let padded = children.next().unwrap();

            if self.controls.is_some() {
                let mut children = padded.children();
                let _ = children.next().unwrap();
                let controls_layout = children.next().unwrap();

                !controls_layout.bounds().contains(cursor_position)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub(crate) fn hash_layout(&self, hasher: &mut Hasher) {
        use std::hash::Hash;

        self.content.hash_layout(hasher);
        self.padding.hash(hasher);

        if let Some(controls) = &self.controls {
            controls.hash_layout(hasher);
        }
    }

    pub(crate) fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.pad(self.padding);
        let max_size = limits.max();

        let title_layout = self
            .content
            .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));
        let title_size = title_layout.size();

        let mut node = if let Some(controls) = &self.controls {
            let mut controls_layout = controls
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let controls_size = controls_layout.size();
            let space_before_controls = max_size.width - controls_size.width;

            let height = title_size.height.max(controls_size.height);

            controls_layout.move_to(Point::new(space_before_controls, 0.0));

            layout::Node::with_children(
                Size::new(max_size.width, height),
                vec![title_layout, controls_layout],
            )
        } else {
            layout::Node::with_children(
                Size::new(max_size.width, title_size.height),
                vec![title_layout],
            )
        };

        node.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        layout::Node::with_children(node.size().pad(self.padding), vec![node])
    }

    pub(crate) fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        if let Some(controls) = &mut self.controls {
            let mut children = layout.children();
            let padded = children.next().unwrap();

            let mut children = padded.children();
            let _ = children.next();
            let controls_layout = children.next().unwrap();

            controls.on_event(
                event,
                controls_layout,
                cursor_position,
                renderer,
                clipboard,
                messages,
            )
        } else {
            event::Status::Ignored
        }
    }
}