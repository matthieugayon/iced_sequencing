mod content;
mod state;
mod title_bar;

pub use content::Content;
pub use state::State;
pub use title_bar::TitleBar;

use iced_native::{
    container, event, layout, mouse, overlay, row, Clipboard, Element, Event, Hasher, Layout,
    Length, Padding, Point, Rectangle, Size, Vector, Widget,
};

#[allow(missing_debug_implementations)]
pub struct HList<'a, Message, Renderer: self::Renderer> {
    state: &'a mut state::Internal,
    elements: Vec<Content<'a, Message, Renderer>>,
    size: usize,
    width: Length,
    height: Length,
    spacing: u16,
    padding: Padding,
    on_click: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, Message, Renderer> HList<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
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
            state: &mut state.internal,
            elements,
            width: Length::Fill,
            height: Length::Fill,
            size: 16,
            spacing: 0,
            padding: Padding::ZERO,
            on_click: None,
            on_drag: None,
            style: Default::default(),
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

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
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

    pub fn style(mut self, style: impl Into<<Renderer as self::Renderer>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> HList<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn click_pane(
        &mut self,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        let mut clicked_region = self
            .elements
            .iter()
            .enumerate()
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().contains(cursor_position));

        if let Some(((pick_index, content), layout)) = clicked_region.next() {
            if let Some(on_click) = &self.on_click {
                messages.push(on_click(pick_index));
            }

            if let Some(on_drag) = &self.on_drag {
                if content.can_be_picked_at(layout, cursor_position) {
                    let pane_position = layout.position();

                    let origin = cursor_position - Vector::new(pane_position.x, pane_position.y);

                    self.state.pick_pane(&pick_index, origin);

                    messages.push(on_drag(DragEvent::Picked { pane: pick_index }));
                }
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
    Renderer: self::Renderer + container::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
            .pad(self.padding);
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

        let mut node = layout::Node::with_children(size, children);

        node.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        layout::Node::with_children(node.size().pad(self.padding), vec![node])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let content_layout = layout.children().next().unwrap();
        let mut event_status = event::Status::Ignored;
        let picked_pane = self.state.picked_pane().map(|(pane, _)| pane);

        let items_event_status = self
            .elements
            .iter_mut()
            .enumerate()
            .zip(content_layout.children())
            .map(|((index, content), layout)| {
                let is_picked = picked_pane == Some(index);

                content.on_event(
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    messages,
                    is_picked,
                )
            })
            .fold(event_status, event::Status::merge);

        if items_event_status == event::Status::Captured {
            return items_event_status;
        } else {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    let bounds = content_layout.bounds();

                    if bounds.contains(cursor_position) {
                        event_status = event::Status::Captured;
                        self.click_pane(content_layout, cursor_position, messages);
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if let Some((pane, _)) = self.state.picked_pane() {
                        if let Some(on_drag) = &self.on_drag {
                            let mut dropped_region = self
                                .elements
                                .iter()
                                .enumerate()
                                .zip(content_layout.children())
                                .filter(|(_, layout)| layout.bounds().contains(cursor_position));

                            let event = match dropped_region.next() {
                                Some(((target, _), _)) if pane != target => {
                                    DragEvent::Dropped { pane, target }
                                }
                                _ => DragEvent::Canceled { pane },
                            };

                            messages.push(on_drag(event));
                        }

                        self.state.idle();

                        event_status = event::Status::Captured;
                    }
                }
                _ => {}
            };

            return event_status;
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        self::Renderer::draw(
            renderer,
            defaults,
            &self.elements,
            self.state.picked_pane(),
            layout,
            &self.style,
            cursor_position,
            viewport,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);

        for element in &self.elements {
            element.hash_layout(state);
        }
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        let content_layout = layout.children().next().unwrap();

        self.elements
            .iter_mut()
            .zip(content_layout.children())
            .filter_map(|(pane, layout)| pane.overlay(layout))
            .next()
    }
}

pub trait Renderer: iced_native::Renderer + container::Renderer + Sized {
    type Style: Default;

    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[Content<'_, Message, Self>],
        dragging: Option<(usize, Point)>,
        layout: Layout<'_>,
        style: &<Self as self::Renderer>::Style,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output;

    fn draw_pane<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style: &<Self as container::Renderer>::Style,
        title_bar: Option<(&TitleBar<'_, Message, Self>, Layout<'_>)>,
        body: (&Element<'_, Message, Self>, Layout<'_>),
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output;

    fn draw_title_bar<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style: &<Self as container::Renderer>::Style,
        content: (&Element<'_, Message, Self>, Layout<'_>),
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<HList<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + row::Renderer,
    Message: 'a,
{
    fn from(pane_grid: HList<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}
