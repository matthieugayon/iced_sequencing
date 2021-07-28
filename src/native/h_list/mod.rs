mod content;
// mod pane;
mod state;
mod title_bar;

pub use content::Content;
// pub use pane::Pane;
pub use state::State;
pub use title_bar::TitleBar;

use iced_native::{
    container, Event, event, layout, 
    mouse, overlay, row, 
    Clipboard, Element, Hasher, Layout, 
    Length, Point, Rectangle, Size, Vector,
    Widget
};

#[allow(missing_debug_implementations)]
pub struct HList<'a, Message, Renderer: self::Renderer> {
    state: &'a mut state::Internal,
    elements: Vec<Content<'a, Message, Renderer>>,
    size: usize,
    width: Length,
    height: Length,
    spacing: u16,
    on_click: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, Message, Renderer> HList<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// Creates a [`HList`] with the given [`State`] and view function.
    ///
    /// The view function will be called to display each [`Pane`] present in the
    /// [`State`].
    pub fn new<T>(
        state: &'a mut State<T>,
        view: impl Fn(&'a mut T) -> Content<'a, Message, Renderer>,
    ) -> Self {
        let elements = {
            state
                .panes
                .iter_mut()
                .map(|pane_state| view(pane_state))
                .collect()
        };

        Self {
            state: &mut state.internal,
            elements,
            width: Length::Fill,
            height: Length::Fill,
            size: 16,
            spacing: 0,
            on_click: None,
            on_drag: None,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`HList`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`HList`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the spacing _between_ the panes of the [`HList`].
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the max number of panes of the [`HList`].
    pub fn size(mut self, number_of_panes: usize) -> Self {
        self.size = number_of_panes;
        self
    }

    /// Sets the message that will be produced when a [`Pane`] of the
    /// [`HList`] is clicked.
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(usize) -> Message,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Enables the drag and drop interactions of the [`HList`], which will
    /// use the provided function to produce messages.
    pub fn on_drag<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(DragEvent) -> Message,
    {
        self.on_drag = Some(Box::new(f));
        self
    }

    /// Sets the style of the [`HList`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
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
        let mut clicked_region =
            self.elements.iter().enumerate().zip(layout.children()).filter(
                |(_, layout)| layout.bounds().contains(cursor_position),
            );

        if let Some(((pick_index, content), layout)) = clicked_region.next() {
            if let Some(on_click) = &self.on_click {
                messages.push(on_click(pick_index));
            }

            if let Some(on_drag) = &self.on_drag {
                if content.can_be_picked_at(layout, cursor_position) {
                    let pane_position = layout.position();

                    let origin = cursor_position
                        - Vector::new(pane_position.x, pane_position.y);

                    self.state.pick_pane(&pick_index, origin);

                    messages.push(on_drag(DragEvent::Picked { pane: pick_index }));
                }
            }
        }
    }
}

/// An event produced during a drag and drop interaction of a [`HList`].
#[derive(Debug, Clone, Copy)]
pub enum DragEvent {
    /// A [`Pane`] was picked for dragging.
    Picked {
        /// The picked [`Pane`].
        pane: usize,
    },

    /// A [`Pane`] was dropped on top of another [`Pane`].
    Dropped {
        /// The picked [`Pane`].
        pane: usize,

        /// The [`Pane`] where the picked one was dropped on.
        target: usize,
    },

    /// A [`Pane`] was picked and then dropped outside of other [`Pane`]
    /// boundaries.
    Canceled {
        /// The picked [`Pane`].
        pane: usize,
    },
}


impl<'a, Message, Renderer> Widget<Message, Renderer>
    for HList<'a, Message, Renderer>
where
    Renderer: self::Renderer + container::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
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
                  width: area_width - 2. * self.spacing as f32,
                  height: size.height - 2. * self.spacing as f32
                }; 

                let size = Size::new(region.width, region.height);

                let mut node =
                    element.layout(renderer, &layout::Limits::new(size, size));

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
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let mut event_status = event::Status::Ignored;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    event_status = event::Status::Captured;
                    self.click_pane(layout, cursor_position, messages);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some((pane, _)) = self.state.picked_pane() {
                    if let Some(on_drag) = &self.on_drag {
                        let mut dropped_region =
                            self.elements.iter().enumerate().zip(layout.children()).filter(
                                |(_, layout)| {
                                    layout.bounds().contains(cursor_position)
                                },
                            );

                        let event = match dropped_region.next() {
                            Some(((target,_), _)) if pane != target => {
                                DragEvent::Dropped {
                                    pane,
                                    target
                                }
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
        }

        if self.state.picked_pane().is_none() {
            self.elements
                .iter_mut()
                .zip(layout.children())
                .map(|(pane, layout)| {
                    pane.on_event(
                        event.clone(),
                        layout,
                        cursor_position,
                        renderer,
                        clipboard,
                        messages,
                    )
                })
                .fold(event_status, event::Status::merge)
        } else {
            event::Status::Captured
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

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.elements
            .iter_mut()
            .zip(layout.children())
            .filter_map(|(pane, layout)| pane.overlay(layout))
            .next()
    }
}

/// The renderer of a [`HList`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`HList`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer: iced_native::Renderer + container::Renderer + Sized {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`HList`].
    ///
    /// It receives:
    /// - the elements of the [`HList`]
    /// - the [`Pane`] that is currently being dragged
    /// - the [`Axis`] that is currently being resized
    /// - the [`Layout`] of the [`HList`] and its elements
    /// - the cursor position
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

    /// Draws a [`Pane`].
    ///
    /// It receives:
    /// - the [`TitleBar`] of the [`Pane`], if any
    /// - the [`Content`] of the [`Pane`]
    /// - the [`Layout`] of the [`Pane`] and its elements
    /// - the cursor position
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

    /// Draws a [`TitleBar`].
    ///
    /// It receives:
    /// - the bounds, style of the [`TitleBar`]
    /// - the style of the [`TitleBar`]
    /// - the content of the [`TitleBar`] with its layout
    /// - the controls of the [`TitleBar`] with their [`Layout`], if any
    /// - the cursor position
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

impl<'a, Message, Renderer> From<HList<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + row::Renderer,
    Message: 'a,
{
    fn from(
        pane_grid: HList<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}