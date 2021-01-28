use iced_native::{Widget, Length, layout, Size, event, Event, mouse, Point, Layout, Clipboard, Color, Element, Rectangle, Hasher};

use std::hash::Hash;
use crate::core::grid::normalize_point;

/// A vertical [`MultiSlider`] GUI widget.
///
/// [`MultiSlider`]: struct.MultiSlider.html
pub struct MultiSlider<'a, Message> {
    state: &'a mut State,
    on_change: Box<dyn Fn(Vec<f32>) -> Message>,
    width: u16,
    height: u16,
    sliders: Vec<Slider>,
    background_color: Color
}

impl<'a, Message> MultiSlider<'a, Message> {
    /// Creates a new [`MultiSlider`].
    ///
    /// It expects:
    /// * a multi_slider [`State`] with its initial parameters
    /// * a on_change function
    /// * a width as an integer
    /// * a height as an integer
    /// * a number_of_sliders as an integer
    /// * a background_color
    ///
    /// [`State`]: struct.State.html
    /// [`MultiSlider`]: struct.MultiSlider.html
    pub fn new<F>(
        state: &'a mut State,
        on_change: F,
        width: u16,
        height: u16,
        number_of_sliders: u16,
        background_color: Color
    ) -> Self
        where
            F: 'static + Fn(Vec<f32>) -> Message,
    {
        MultiSlider {
            state,
            on_change: Box::new(on_change),
            width,
            height,
            sliders: Self::generate_sliders(
                number_of_sliders, width, height
            ),
            background_color
        }
    }

    fn generate_sliders(
        number_of_sliders: u16,
        window_width: u16,
        window_height: u16
    ) -> Vec<Slider> {
        let mut sliders: Vec<Slider> = Vec::new();
        for index in 0..number_of_sliders {
            sliders.push(Slider::new(
                window_width,
                window_height,
                number_of_sliders,
                index
            ));
        }
        sliders
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
for MultiSlider<'a, Message>
    where
        Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(Length::from(self.width))
            .height(Length::from(self.height));

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let normalized_cursor_position = normalize_point(cursor_position, bounds);

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(_click) => {
                    self.state.clicked = self.state.focused;
                }
                mouse::Event::ButtonReleased(_click) => {
                    self.state.clicked = false;
                }
                mouse::Event::CursorMoved{ .. } => {
                    self.state.focused = cursor_position.x < bounds.x + bounds.width &&
                        cursor_position.x > bounds.x &&
                        cursor_position.y > bounds.y &&
                        cursor_position.y < bounds.y + bounds.height
                }
                _ => {}
            }
            _ => {}
        }

        for slider in &mut self.sliders {
            slider.update(
                bounds,
                normalized_cursor_position,
                self.state.clicked
            );
            /*self.state.values[slider.index as usize] = slider.value;*/
        }

        if self.state.clicked && self.state.focused {
            /*messages.push((self.on_change)(self.state.values));*/
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            self.background_color,
            &self.sliders,
            self.state.focused
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}


/// The local state of a [`MultiSlider`].
///
/// It knows when the widget has been clicked and when it is focused.
///
/// It also stores the current values of all the [`Slider`].
///
/// [`MultiSlider`]: struct.MultiSlider.html
/// [`Slider`]: struct.Slider.html
#[derive(Debug, Clone)]
pub struct State {
    clicked: bool,
    focused: bool,
    values: Vec<f32>
}

impl State {
    /// Creates a new [`MultiSlider`] state.
    ///
    /// [`MultiSlider`]: struct.MultiSlider.html
    pub fn new(initial_values: Vec<f32>) -> Self {
        Self {
            clicked: false,
            focused: false,
            values: initial_values
        }
    }
}

/// The renderer of a [`MultiSlider`].
///
/// [`MultiSlider`]: struct.MultiSlider.html
pub trait Renderer: iced_native::Renderer {
    /// Draws a [`MultiSlider`].
    ///
    /// It receives:
    ///   * the bounds of the [`MultiSlider`]
    ///   * the background_color of the [`MultiSlider`]
    ///   * a vector of [`Slider`] composing the [`MultiSlider`]
    ///   * the focused state of the [`MultiSlider`]
    ///
    /// [`MultiSlider`]: struct.MultiSlider.html
    /// [`Slider`]: struct.Slider.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        background_color: Color,
        sliders: &Vec<Slider>,
        focused: bool
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<MultiSlider<'a, Message>>
for Element<'a, Message, Renderer>
    where
        Renderer: 'a + self::Renderer,
        Message: 'a,
{
    fn from(
        multi_slider: MultiSlider<'a, Message>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(multi_slider)
    }
}

/// The [`Slider`] of a [`MultiSlider`].
///
/// It handles a value scaled between 0.0 and 1.0 reflecting the slider range.
/// 
/// [`Slider`]: struct.Slider.html
/// [`MultiSlider`]: struct.MultiSlider.html
#[derive(Debug, Clone)]
pub struct Slider {
    pub origin: Point,
    pub size: Size,
    bounds: (f32, f32),
    index: u16,
    value: f32,
    pub hovered: bool
}

impl Slider {
    pub fn new(
        widget_width: u16,
        widget_height: u16,
        number_of_sliders: u16,
        index: u16
    ) -> Slider {
        let slider_width = widget_width / number_of_sliders;
        let slider_height = 0;

        let origin = Point::new((slider_width * index) as f32, widget_height as f32);
        let size = Size::new(slider_width as f32, slider_height as f32);

        Slider {
            origin,
            size,
            bounds: (origin.x, origin.x + size.width),
            index,
            value: 0.0,
            hovered: false
        }
    }

    /// A [`Slider`] will update itself according to the user's actions.
    ///
    /// It will change if hovered and if the user is clicking (and moving).
    ///
    /// It receives:
    ///   * the bounds of the [`MultiSlider`]
    ///   * the mouse_position in the [`MultiSlider`]
    ///   * the clicked state of the [`MultiSlider`]
    ///
    /// [`Slider`]: struct.Slider.html
    /// [`MultiSlider`]: struct.MultiSlider.html
    pub fn update(
        &mut self,
        bounds: Rectangle,
        mouse_position: Point,
        is_widget_clicked: bool
    ) {
        self.hovered = mouse_position.x > self.bounds.0 &&
            mouse_position.x < self.bounds.1;

        if self.hovered && is_widget_clicked {
            self.size.height = mouse_position.y - bounds.height;
            self.value = (self.size.height / bounds.height).abs();
        }
    }
}
