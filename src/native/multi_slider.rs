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

    // pub fn number_of_sliders(mut self, number_of_sliders: u16) -> Self {
    //     // match let sliders = self.state.values.len() {
    //     //     number_of_sliders > sliders {

    //     //     }
    //     // }
    //     // self.number_of_sliders = number_of_sliders;
    //     self
    // }

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    last_cursor_position: Option<Point>
}

impl Default for State {
    fn default() -> Self {
        State {
            last_cursor_position: None
        }
    }
}

impl State {
    pub fn new() -> State {
        State::default()
    }
}


impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for MultiSlider<'a, T, Message, Renderer>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive + std::fmt::Debug,
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
        let slider_width = content_bounds.width / self.values.len() as f32;
        let max_index = self.values.len() - 1;
        let get_slider_index = |cursor: Point| -> usize {
            if cursor.x >= content_bounds.x + content_bounds.width {
                max_index
            } else if cursor.x >= content_bounds.x {
                (((cursor.x - content_bounds.x) / slider_width) as usize).min(max_index)
            } else {
                0
            }
        };

        let interpolate_value = |(cursor_y, slider)| -> (Option<T>, usize) {
            if cursor_y >= content_bounds.y + content_bounds.height {
                (Some(*self.range.start()), slider)
            } else if cursor_y <= content_bounds.y {
                (Some(*self.range.end()), slider)
            } else {
                let step = self.step.into();
                let start = (*self.range.start()).into();
                let end = (*self.range.end()).into();
                let percent = f64::from(content_bounds.y + content_bounds.height - cursor_y)
                    / f64::from(content_bounds.height);

                let steps = (percent * (end - start) / step).round();
                let value = steps * step + start;

                (T::from_f64(value), slider)
            }
        };

        let map_slider_fct = |min_index: usize, max_index: usize, position_y_a: f32, position_y_b: f32| {
            move |index: usize| -> (f32, usize) {
                let factor = (index - min_index) as f32 / (max_index - min_index) as f32;
                let interpolated_y = position_y_a + factor * (position_y_b - position_y_a);
                (interpolated_y, index)
            }
        };

        let mut change = || -> () {
            let current_slider_index = get_slider_index(cursor_position);

            match self.state.last_cursor_position {
                Some(last_cursor_position) => {
                    let previous_slider_index = get_slider_index(last_cursor_position);
                    let mut sliders_to_edit = ((previous_slider_index + 1)..(current_slider_index + 1)).into_iter()
                        .map(map_slider_fct(previous_slider_index, current_slider_index, last_cursor_position.y, cursor_position.y))
                        .collect();
                    
                    if (current_slider_index as isize - previous_slider_index as isize).abs() <= 1 {
                        sliders_to_edit = vec![(cursor_position.y, current_slider_index)];
                    } else if previous_slider_index > current_slider_index {
                        sliders_to_edit = (current_slider_index..previous_slider_index).into_iter()
                            .map(map_slider_fct(current_slider_index, previous_slider_index, cursor_position.y, last_cursor_position.y))
                            .collect();
                    }

                    let new_values: Vec<(Option<T>, usize)> = sliders_to_edit
                        .into_iter()
                        .map(interpolate_value)
                        .collect();

                    let mut values: Vec<T> = self.values.clone();

                    if new_values.iter().any(|(val, _)| val.is_some()) {
                        for (val, slider) in new_values {
                            match val {
                                Some(value) => {
                                    values[slider] = value
                                },
                                None => {},
                            }
                        }

                        messages.push((self.on_change)(values));
                    }
                },
                None => {
                    let new_value = interpolate_value((cursor_position.y, current_slider_index));
                    match new_value.0 {
                        Some(value) => {
                            let mut values: Vec<T> = self.values.clone();
                            values[current_slider_index] = value;
                            messages.push((self.on_change)(values));
                        },
                        None => {},
                    }
                },
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    change();
                    self.state.last_cursor_position = Some(cursor_position);
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if self.state.last_cursor_position.is_some() {
                    if let Some(on_release) = self.on_release.clone() {
                        messages.push(on_release);
                    }
                    self.state.last_cursor_position = None;
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if self.state.last_cursor_position.is_some() {
                    change();
                    self.state.last_cursor_position = Some(cursor_position);
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
        spacing: u16,
        base_color: Color,
        style: &Self::Style
    ) -> Self::Output;
}


impl<'a, T, Message, Renderer> From<MultiSlider<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: 'a + Copy + Into<f64> + num_traits::FromPrimitive + std::fmt::Debug,
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer,
{
    fn from(
        multi_slider: MultiSlider<'a, T, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(multi_slider)
    }
}
