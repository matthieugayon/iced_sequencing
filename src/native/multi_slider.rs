use iced_native::{
    event, layout, mouse, touch, Clipboard, Color,
    Element, Event, Layout, Length, Shell,
    Point, Rectangle, Size, Widget, renderer,Background
};
use std::ops::RangeInclusive;
pub use crate::style::multi_slider::{Style, StyleSheet};

#[allow(missing_debug_implementations)]
pub struct MultiSlider<'a, T, Message> {
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
    base_color: Color,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, T, Message> MultiSlider<'a, T, Message>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone
{
    pub fn new<F>(
        state: &'a mut State,
        range: RangeInclusive<T>,
        values: Vec<T>,
        on_change: F,
        base_color: Color,
    ) -> Self
    where
        F: 'static + Fn(Vec<T>) -> Message,
    {
        let slider_values: Vec<T> = values
            .into_iter()
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
            base_color,
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

    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet + 'a>>) -> Self {
        self.style_sheet = style.into();
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

pub fn draw<T, Renderer>(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    cursor_position: Point,
    values: Vec<T>,
    range: &RangeInclusive<T>,
    style_sheet: &dyn StyleSheet,
    base_color: Color,
    spacing: u16,
    active: Option<usize>
) where
    T: Into<f64> + Copy,
    Renderer: iced_native::Renderer,
{
    let style = style_sheet.default(base_color);
    let highlight_slider_style = style_sheet.highlight(base_color);
    let hovered_slider_style = style_sheet.hovered(base_color);
    let bounds = layout.bounds();
    let slider_width = bounds.width / values.len() as f32;
    let (range_start, range_end) = {
        let (start, end) = range.clone().into_inner();
        (start.into() as f32, end.into() as f32)
    };

    if style.background.is_some() || style.border_width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: style.border_radius,
                border_width: style.border_width,
                border_color: style.border_color,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    values.into_iter()
        .enumerate()
        .for_each(|(index, value)| {
            let value = value.into() as f32;
            let ranged_value = if range_start >= range_end {
                0.0
            } else {
                (value - range_start) / (range_end - range_start)
            };
            let slider_height = bounds.height * ranged_value;

            let slider_range_bounds = Rectangle {
                x: bounds.x + index as f32 * slider_width,
                y: bounds.y,
                width: slider_width,
                height: bounds.height,
            };
            let slider_bounds = Rectangle {
                x: (bounds.x + index as f32 * slider_width + spacing as f32)
                    .round(),
                y: bounds.y + bounds.height - slider_height,
                width: (slider_width - 2. * spacing as f32).round(),
                height: slider_height,
            };

            let slider_style = match active {
                Some(active_slider) => {
                    if slider_range_bounds.contains(cursor_position) {
                        hovered_slider_style
                    } else if active_slider == index {
                        highlight_slider_style
                    } else {
                        style.slider
                    }
                }
                None => {
                    if slider_range_bounds.contains(cursor_position) {
                        hovered_slider_style
                    } else {
                        style.slider
                    }
                }
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: slider_bounds,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color(slider_style.color)
            );
        });
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    last_cursor_position: Option<Point>,
}

impl Default for State {
    fn default() -> Self {
        State {
            last_cursor_position: None,
        }
    }
}

impl State {
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer> for MultiSlider<'a, T, Message>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let slider_width = bounds.width / self.values.len() as f32;
        let max_index = self.values.len() - 1;
        let get_slider_index = |cursor: Point| -> usize {
            if cursor.x >= bounds.x + bounds.width {
                max_index
            } else if cursor.x >= bounds.x {
                (((cursor.x - bounds.x) / slider_width) as usize).min(max_index)
            } else {
                0
            }
        };

        let interpolate_value = |(cursor_y, slider)| -> (Option<T>, usize) {
            if cursor_y >= bounds.y + bounds.height {
                (Some(*self.range.start()), slider)
            } else if cursor_y <= bounds.y {
                (Some(*self.range.end()), slider)
            } else {
                let step = self.step.into();
                let start = (*self.range.start()).into();
                let end = (*self.range.end()).into();
                let percent = f64::from(bounds.y + bounds.height - cursor_y)
                    / f64::from(bounds.height);

                let steps = (percent * (end - start) / step).round();
                let value = steps * step + start;

                (T::from_f64(value), slider)
            }
        };

        let map_slider_fct =
            |min_index: usize, max_index: usize, position_y_a: f32, position_y_b: f32| {
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
                    let mut sliders_to_edit = ((previous_slider_index + 1)
                        ..(current_slider_index + 1))
                        .into_iter()
                        .map(map_slider_fct(
                            previous_slider_index,
                            current_slider_index,
                            last_cursor_position.y,
                            cursor_position.y,
                        ))
                        .collect();

                    if (current_slider_index as isize - previous_slider_index as isize).abs() <= 1 {
                        sliders_to_edit = vec![(cursor_position.y, current_slider_index)];
                    } else if previous_slider_index > current_slider_index {
                        sliders_to_edit = (current_slider_index..previous_slider_index)
                            .into_iter()
                            .map(map_slider_fct(
                                current_slider_index,
                                previous_slider_index,
                                cursor_position.y,
                                last_cursor_position.y,
                            ))
                            .collect();
                    }

                    let new_values: Vec<(Option<T>, usize)> =
                        sliders_to_edit.into_iter().map(interpolate_value).collect();

                    let mut values: Vec<T> = self.values.clone();

                    if new_values.iter().any(|(val, _)| val.is_some()) {
                        for (val, slider) in new_values {
                            match val {
                                Some(value) => values[slider] = value,
                                None => {}
                            }
                        }

                        shell.publish((self.on_change)(values));
                    }
                }
                None => {
                    let new_value = interpolate_value((cursor_position.y, current_slider_index));
                    match new_value.0 {
                        Some(value) => {
                            let mut values: Vec<T> = self.values.clone();
                            values[current_slider_index] = value;
                            shell.publish((self.on_change)(values));
                        }
                        None => {}
                    }
                }
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
                        shell.publish(on_release);
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
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            layout,
            cursor_position,
            self.values.clone(),
            &self.range,
            self.style_sheet.as_ref(),
            self.base_color,
            self.spacing,
            self.active
        );
    }

    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, T, Message, Renderer> From<MultiSlider<'a, T, Message>>
    for Element<'a, Message, Renderer>
where
    T: 'a + Copy + Into<f64> + num_traits::FromPrimitive,
    Message: 'a + Clone,
    Renderer: 'a + iced_native::Renderer,
{
    fn from(multi_slider: MultiSlider<'a, T, Message>) -> Element<'a, Message, Renderer> {
        Element::new(multi_slider)
    }
}