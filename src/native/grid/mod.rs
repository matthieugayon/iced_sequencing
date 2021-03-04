use std::{collections::HashMap, fmt::Debug};

use iced_native::{
    event, keyboard, layout, mouse, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Widget
};

use std::hash::Hash;

use ganic_no_std::{NUM_PERCS, NUM_STEPS, pattern::Pattern};

use crate::core::grid::{
    STEP_MARGIN_RIGHT, TRACK_MARGIN_BOTTOM, 
    CONTAINER_PADDING, OFFSET_THRESHOLD,
    Actions, DrawMode,
    GridEvent, GridPattern, 
    normalize_point, is_point_inside_draggable_area,
    get_event_absolute_position, get_step_dimensions
}; 

pub mod modes;

use modes::{WidgetState, Transition, Idle};


pub struct Grid<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    on_change: Box<dyn Fn(Pattern) -> Message>,
    width: Length,
    height: Length,
    style: Renderer::Style
}

impl<'a, Message, Renderer: self::Renderer> Grid<'a, Message, Renderer> {
    pub fn new<F>(
        state: &'a mut State,
        on_change: F,
        width: Length,
        height: Length
    ) -> Self
    where
        F: 'static + Fn(Pattern) -> Message,
    {
        Grid {
            state,
            on_change: Box::new(on_change),
            width,
            height,
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

    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct WidgetContext {
    // base pattern we use as a base (sometimes modifications are not applied, ex: when you drag and press Escape)
    base_pattern: GridPattern,
    output_pattern: GridPattern,
    selection_rectangle: Option<Rectangle>
}

#[derive(Debug)]
pub struct State {
    current_state: Box<dyn WidgetState>, // state machine state
    context: WidgetContext, // context we'll mutate in our state machine
    last_click: Option<mouse::Click>
}

impl State {
    pub fn new(initial_pattern: Option<Pattern>) -> Self {
        let base_pattern= {
            match initial_pattern {
                Some(pattern) => {
                    GridPattern::from(pattern)
                }
                None => {
                    GridPattern::new()
                }
            }
        };

        Self {
            current_state: Box::new(Idle::default()),
            context: WidgetContext {
                base_pattern: base_pattern.clone(),
                output_pattern: base_pattern.clone(),
                selection_rectangle: None,
            },
            last_click: None
        }
    }

    fn handle_event<F>(&mut self, handler: F)
        where F: FnOnce(&mut dyn WidgetState, &mut WidgetContext) -> Transition,
    {
        let transition = handler(
            &mut *self.current_state,
            &mut self.context
        );
        self.handle_transition(transition);
    }

    fn handle_transition(&mut self, transition: Transition) {
        match transition {
            Transition::ChangeState(new_state) => {
                println!("Changing state {:?} => {:?}",
                    self.current_state,
                    new_state
                );
                self.current_state = new_state
            },
            Transition::DoNothing => {},
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Grid<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        let bounds = layout.bounds();

        // println!("on_event bounds {:?}", bounds);
        // println!("cursor_position {:?}", cursor_position);

        // dispatch events to our state machine whose states (modes) and substates are defined 
        // in ./modes

        let normalized_cursor_position = normalize_point(cursor_position, bounds);
        let normalized_bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: bounds.width,
            height: bounds.height
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    self.state.handle_event(|widget_state, context| {
                        widget_state.on_cursor_moved(
                            normalized_bounds,
                            normalized_cursor_position,
                            context
                        )
                    });

                    return event::Status::Captured;
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if bounds.contains(cursor_position) {
                        let click = mouse::Click::new(
                            normalized_cursor_position,
                            self.state.last_click,
                        );

                        match click.kind() {
                            mouse::click::Kind::Single => {
                                self.state.handle_event(|widget_state, context| {
                                    widget_state.on_click(
                                        normalized_bounds,
                                        normalized_cursor_position,
                                        context
                                    )
                                });
                            },
                            mouse::click::Kind::Double => {
                                self.state.handle_event(|widget_state, context| {
                                    widget_state.on_double_click(
                                        normalized_bounds,
                                        normalized_cursor_position,
                                        context
                                    )
                                });
                            },
                            _ => {}
                        }

                        self.state.last_click = Some(click);

                        // messages.push((self.on_change)(Pattern::from(self.state.output_pattern.to_owned())));

                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.state.handle_event(|widget_state, context| {
                        widget_state.on_button_release(
                            normalized_bounds,
                            normalized_cursor_position,
                            context
                        )
                    });

                    return event::Status::Captured;
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { key_code, .. } => {
                    self.state.handle_event(|widget_state, context| {
                        widget_state.on_key_pressed(
                            key_code,
                            context
                        )
                    });

                    return event::Status::Captured;
                } 
                keyboard::Event::KeyReleased { key_code, .. } => {
                    self.state.handle_event(|widget_state, context| {
                        widget_state.on_key_released(
                            key_code,
                            context
                        )
                    });

                    return event::Status::Captured;
                }            
                keyboard::Event::ModifiersChanged(modifiers) => {
                    self.state.handle_event(|widget_state, context| {
                        widget_state.on_modifier_change(
                            modifiers,
                            context
                        )
                    });

                    return event::Status::Captured;
                }
                _ => {}
            },
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
        renderer.draw(
            layout.bounds(),
            cursor_position,
            self.state.context.output_pattern.to_owned(),
            self.state.context.selection_rectangle,
            &self.style
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        grid_pattern: GridPattern,
        selection: Option<Rectangle>,
        style: &Self::Style
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Grid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        grid: Grid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(grid)
    }
}
