use std::fmt::Debug;
use iced_native::{
    event, keyboard, layout, mouse, Clipboard,
    Element, Event, Layout, Length, Padding,
    Point, Rectangle, Size, Widget, Shell,
};
use iced_graphics::canvas;

use ganic_no_std::{NUM_PERCS, NUM_STEPS};
use crate::core::grid::{GridMessage, GridPattern};
pub use crate::style::multi_slider::{Style, StyleSheet};

pub mod modes;
use modes::{Idle, Transition, WidgetState};

pub struct Grid<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    live_pattern: GridPattern,
    on_event: Box<dyn Fn(GridMessage) -> Message>,
    width: Length,
    height: Length,
    padding: Padding,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer> Grid<'a, Message, Renderer> {
    pub fn new<F>(
        state: &'a mut State,
        live_pattern: GridPattern,
        on_event: F,
        width: Length,
        height: Length,
    ) -> Self
    where
        F: 'static + Fn(GridMessage) -> Message,
    {
        Grid {
            state,
            live_pattern,
            on_event: Box::new(on_event),
            width,
            height,
            padding: Padding::ZERO,
            style: Renderer::Style::default(),
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

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.state.event_cache.clear();
        self.state.grid_cache.clear();
        self.style = style.into();
        self
    }

    fn handle_event<F>(&mut self, handler: F, messages_queue: &mut Shell<'_, Message>)
    where
        F: FnOnce(
            &mut dyn WidgetState,
            &mut WidgetContext,
            GridPattern,
        ) -> (Transition, Option<Vec<GridMessage>>),
    {
        let (transition, grid_messages) = handler(
            &mut *self.state.current_state,
            &mut self.state.context,
            self.state.base_pattern.clone(),
        );

        self.handle_transition(transition);

        match grid_messages {
            Some(messages) => {
                // clear event cache to update events display
                self.state.event_cache.clear();

                messages.into_iter().for_each(|message| {
                    messages_queue.publish((self.on_event)(message));
                });
            }
            None => {}
        }
    }

    fn handle_transition(&mut self, transition: Transition) {
        match transition {
            Transition::ChangeState(new_state) => {
                // println!("Changing state {:?} => {:?}",
                //     self.state.current_state,
                //     new_state
                // );
                self.state.current_state = new_state
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct WidgetContext {
    // base pattern we use as a base (sometimes modifications are not applied, ex: when you drag and press Escape)
    selection_rectangle: Option<Rectangle>,
    mouse_interaction: mouse::Interaction,
}

#[derive(Debug)]
pub struct State {
    current_state: Box<dyn WidgetState + Send>, // state machine state
    context: WidgetContext,                     // context we'll mutate in our state machine
    base_pattern: GridPattern,
    temp_movement: Option<(f32, isize)>,
    last_click: Option<mouse::Click>,
    highlight: [usize; NUM_PERCS],
    is_playing: bool,
    grid_cache: canvas::Cache,
    event_cache: canvas::Cache,
    highlight_cache: canvas::Cache,
    mutes: [bool; NUM_PERCS],
}

impl State {
    pub fn new(grid: GridPattern) -> Self {
        Self {
            current_state: Box::new(Idle::default()),
            context: WidgetContext {
                selection_rectangle: None,
                mouse_interaction: mouse::Interaction::default(),
            },
            base_pattern: grid,
            temp_movement: None,
            last_click: None,
            highlight: [0; NUM_PERCS],
            is_playing: false,
            grid_cache: Default::default(),
            event_cache: Default::default(),
            highlight_cache: Default::default(),
            mutes: [false; NUM_PERCS],
        }
    }

    pub fn set_pattern(&mut self, grid: GridPattern) {
        self.event_cache.clear();
        self.temp_movement = None;
        self.base_pattern = grid;
    }

    pub fn set_movement(&mut self, movement: (f32, isize), relative: bool) {
        match self.temp_movement {
            Some(current_movement) => {
                if relative {
                    self.temp_movement = Some((
                        (current_movement.0 + movement.0) % NUM_STEPS as f32,
                        current_movement.1 + movement.1
                    ));
                } else {
                    self.temp_movement = Some(movement);
                }
            },
            None => {
                self.temp_movement = Some(movement);
            },
        }
    }

    pub fn get_movement(&self) -> Option<(f32, isize)> {
        return self.temp_movement;
    }

    pub fn clone_base_pattern(&self) -> GridPattern {
        self.base_pattern.clone()
    }

    pub fn is_playing(&mut self, is_playing: bool) {
        self.is_playing = is_playing;
    }

    pub fn transport(&mut self, highlight: [Option<usize>; NUM_PERCS]) {
        self.event_cache.clear();
        for (pidx, option_step) in highlight.iter().enumerate() {
            match option_step {
                Some(step) => self.highlight[NUM_PERCS - pidx - 1] = *step,
                None => {}
            }
        }
    }

    pub fn set_mute(&mut self, pidx: usize, mute: bool) {
        self.event_cache.clear();
        self.mutes[NUM_PERCS - pidx - 1] = mute;
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Grid<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
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
        messages: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.children().next().unwrap().bounds();

        // dispatch events to our state machine whose states (modes) and substates are defined
        // in ./modes

        // this is for a bug happening randomly when the cursor leaves the window
        // @TODO: write an issue in the iced repo
        if cursor_position.x < 0. && cursor_position.y < 0. {
            return event::Status::Ignored;
        }

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    self.handle_event(
                        |widget_state, context, base_pattern| {
                            widget_state.on_cursor_moved(
                                bounds,
                                cursor_position,
                                base_pattern,
                                context,
                            )
                        },
                        messages,
                    );

                    return event::Status::Captured;
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if bounds.contains(cursor_position) {
                        let click = mouse::Click::new(cursor_position, self.state.last_click);

                        match click.kind() {
                            mouse::click::Kind::Single => {
                                self.handle_event(
                                    |widget_state, context, base_pattern| {
                                        widget_state.on_click(
                                            bounds,
                                            cursor_position,
                                            base_pattern,
                                            context,
                                        )
                                    },
                                    messages,
                                );
                            }
                            mouse::click::Kind::Double => {
                                self.handle_event(
                                    |widget_state, context, base_pattern| {
                                        widget_state.on_double_click(
                                            bounds,
                                            cursor_position,
                                            base_pattern,
                                            context,
                                        )
                                    },
                                    messages,
                                );
                            }
                            _ => {}
                        }

                        self.state.last_click = Some(click);

                        return event::Status::Captured;
                    } else {
                        self.handle_event(
                            |widget_state, _context, _base_pattern| {
                                widget_state.on_blur()
                            },
                            messages,
                        );
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.handle_event(
                        |widget_state, context, base_pattern| {
                            widget_state.on_button_release(
                                bounds,
                                cursor_position,
                                base_pattern,
                                context,
                            )
                        },
                        messages,
                    );

                    return event::Status::Captured;
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { key_code, .. } => {
                    self.handle_event(
                        |widget_state, context, _| widget_state.on_key_pressed(key_code, context),
                        messages,
                    );

                    return event::Status::Captured;
                }
                keyboard::Event::KeyReleased { key_code, .. } => {
                    self.handle_event(
                        |widget_state, context, _| widget_state.on_key_released(key_code, context),
                        messages,
                    );

                    return event::Status::Captured;
                }
                keyboard::Event::ModifiersChanged(modifiers) => {
                    self.handle_event(
                        |widget_state, context, _| {
                            widget_state.on_modifier_change(modifiers, context)
                        },
                        messages,
                    );

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
        _style: &iced_native::renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        renderer.draw(
            layout.bounds(),
            layout.children().next().unwrap().bounds(),
            cursor_position,
            &self.live_pattern,
            self.state.context.selection_rectangle,
            self.state.context.mouse_interaction,
            self.state.is_playing,
            self.state.highlight,
            self.state.mutes,
            &self.style,
            &self.state.grid_cache,
            &self.state.event_cache,
            &self.state.highlight_cache,
        )
    }
}
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    fn draw(
        &mut self,
        bounds: Rectangle,
        drawable_area: Rectangle,
        cursor_position: Point,
        grid_pattern: &GridPattern,
        selection: Option<Rectangle>,
        mouse_interaction: mouse::Interaction,
        is_playing: bool,
        highlight: [usize; NUM_PERCS],
        mutes: [bool; NUM_PERCS],
        style: &Self::Style,
        grid_cache: &canvas::Cache,
        event_cache: &canvas::Cache,
        highlight_cache: &canvas::Cache,
    );
}

impl<'a, Message, Renderer> From<Grid<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(grid: Grid<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(grid)
    }
}
