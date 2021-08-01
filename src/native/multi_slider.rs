use iced_native::{
    Widget, Length, layout, Size, event, 
    Event, mouse, Point, Layout, Clipboard, 
    Element, Rectangle, Hasher, Color, touch, Padding
};
use std::{hash::Hash, ops::RangeInclusive};

#[allow(missing_debug_implementations)]
pub struct MultiSlider<'a, T, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    values: Vec<T>,
    range: RangeInclusive<T>,
    step: T,
    active: Option<usize>,
    on_change: Box<dyn Fn(Vec<T>) -> Message>,
    on_release: Option<Message>,
    width: Length,
    height: Length,
    spacing: u16,
    padding: Padding,
    base_color: Color,
    style: Renderer::Style
}

impl<'a, T, Message, Renderer> MultiSlider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: self::Renderer,
{
    pub fn new<F>(
        state: &'a mut State,
        range: RangeInclusive<T>,
        values: Vec<T>,
        on_change: F,
        base_color: Color
    ) -> Self
    where
        F: 'static + Fn(Vec<T>) -> Message
    {
        let slider_values: Vec<T> = values.into_iter()
            .map(|value| {
                if value >= *range.start() && value <= *range.end() {
                    value
                } else if value <= *range.start() {
                    *range.start()
                } else {
                    *range.end()
                }
            })
            .collect();

        MultiSlider {
            state,
            values: slider_values,
            range,
            step: T::from(1),
            active: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            height: Length::Fill,
            spacing: 0,
            padding: Padding::ZERO,
            base_color,
            style: Renderer::Style::default()
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

    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    pub fn number_of_sliders(mut self, number_of_sliders: u16) -> Self {
        // match let sliders = self.state.values.len() {
        //     number_of_sliders > sliders {

        //     }
        // }
        // self.number_of_sliders = number_of_sliders;
        self
    }

    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }

    pub fn active(mut self, active: Option<usize>) -> Self {
        self.active = active;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_dragging: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}


impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for MultiSlider<'a, T, Message, Renderer>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
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
            .height(Length::from(self.height))
            .pad(self.padding);

        let mut content = layout::Node::new(limits.resolve(Size::ZERO));
        content.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        let size = limits.resolve(content.size()).pad(self.padding);

        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let content_bounds = layout.children().next().unwrap().bounds();
        let slider_width = (content_bounds.width / self.values.len() as f32).floor();

        let mut change = || -> () {
            let slider_index = {
                if cursor_position.x >= content_bounds.x + content_bounds.width {
                    self.values.len() - 1
                } else if cursor_position.x >= content_bounds.x {
                    ((cursor_position.x - content_bounds.x) / slider_width) as usize
                } else {
                    0
                }
            };

            let changed_value: Option<T> = {
                if cursor_position.y >= content_bounds.y + content_bounds.height {
                    Some(*self.range.start())
                } else if cursor_position.y <= content_bounds.y {
                    Some(*self.range.end())
                } else {
                    let step = self.step.into();
                    let start = (*self.range.start()).into();
                    let end = (*self.range.end()).into();
                    let percent = f64::from(content_bounds.y + content_bounds.height - cursor_position.y)
                        / f64::from(content_bounds.height);

                    let steps = (percent * (end - start) / step).round();
                    let value = steps * step + start;

                    T::from_f64(value)
                }
            };


            match changed_value {
                Some(value) => {
                    let mut values: Vec<T> = self.values.clone();
                    values[slider_index] = value;
                    messages.push((self.on_change)(values));
                },
                _ => {},
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    change();
                    self.state.is_dragging = true;

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if self.state.is_dragging {
                    if let Some(on_release) = self.on_release.clone() {
                        messages.push(on_release);
                    }
                    self.state.is_dragging = false;

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if self.state.is_dragging {
                    change();

                    return event::Status::Captured;
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let content_bounds = layout.children().next().unwrap().bounds();
        let start = *self.range.start();
        let end = *self.range.end();
        let values: Vec<f32> = self.values.iter()
            .map(|&value| {
                value.into() as f32
            })
            .collect();

        renderer.draw(
            layout.bounds(),
            content_bounds,
            cursor_position,
            start.into() as f32..=end.into() as f32,
            values,
            self.active,
            self.state.is_dragging,
            self.spacing,
            self.base_color,
            &self.style,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.padding.hash(state);
    }
}

pub trait Renderer: iced_native::Renderer {
    type Style: Default;

    fn draw(
        &mut self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        cursor_position: Point,
        range: RangeInclusive<f32>,
        values: Vec<f32>,
        active: Option<usize>,
        is_dragging: bool,
        spacing: u16,
        base_color: Color,
        style: &Self::Style
    ) -> Self::Output;
}


impl<'a, T, Message, Renderer> From<MultiSlider<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: 'a + Copy + Into<f64> + num_traits::FromPrimitive,
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer,
{
    fn from(
        multi_slider: MultiSlider<'a, T, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(multi_slider)
    }
}
