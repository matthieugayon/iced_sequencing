use std::{collections::HashMap, fmt::Debug};

use iced_native::{
    event, keyboard, layout, mouse, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Widget
};

use std::hash::Hash;

use ganic_no_std::pattern::Pattern;

pub const STEP_WIDTH: f32 = 40.0;
pub const STEP_HEIGHT: f32 = 40.0;
pub const STEP_MARGIN_RIGHT: f32 = 4.0;
pub const TRACK_MARGIN_BOTTOM: f32 = 16.0;
pub const CONTAINER_PADDING: f32 = 12.0;


#[derive(Debug, Clone, Copy)]
pub struct GridEvent {
    offset: f32,
    velocity: f32
}

#[derive(Debug, Clone)]
pub struct GridPattern {
    data: HashMap<(usize, usize), GridEvent>
}

impl GridPattern {
    pub fn new() -> Self {
        GridPattern {
            data: HashMap::new()
        }
    }

    pub fn get_hovered(self, cursor: Point) -> Option<((usize, usize), GridEvent)>{
        self.data.into_iter()
            .find(|((step, track), grid_event)| {
                let grid_event_rect = Rectangle {
                    x: CONTAINER_PADDING + (grid_event.offset * STEP_WIDTH) + (*step as f32 * (STEP_WIDTH + STEP_MARGIN_RIGHT)),
                    y: CONTAINER_PADDING + (*track as f32 * (STEP_HEIGHT + TRACK_MARGIN_BOTTOM)),
                    width: STEP_WIDTH,
                    height: STEP_HEIGHT
                };
    
                grid_event_rect.contains(cursor)
            })
    }
}

impl From<Pattern> for GridPattern {
    fn from(pattern: Pattern) -> Self {
        let mut grid = GridPattern::new();

        for (i, step) in pattern.iter().enumerate() {
            for (j, perc) in step.iter().enumerate() {
                if perc[0] > 0.0 {
                    grid.data.insert((i, j), GridEvent { velocity: perc[0], offset: perc[1] });
                }
            }
        }

        grid
    }
}

impl From<GridPattern> for Pattern {
    fn from(grid: GridPattern) -> Self {
        let mut pattern = Pattern::new();

        for ((step, track), event) in grid.data {
            pattern.data[step][track][0] = event.velocity;
            pattern.data[step][track][1] = event.offset;
        }

        pattern
    }
}

pub struct Grid<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    on_change: Box<dyn Fn() -> Message>,
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
        F: 'static + Fn() -> Message,
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

    pub fn on_action(&mut self, action: Actions, bounds: Rectangle) {
        match action {
            Actions::Drag(bounds) => {
                if self.state.is_logo_pressed {
                    // velocity mode
                } else {
                    match self.state.draw_mode {
                        DrawMode::Pen => {
                            // draw 
                            match self.state.modifiers {
                                keyboard::Modifiers { logo: true, .. } => {
                                    // draw in step + micro timing mode

                                },
                                keyboard::Modifiers { logo: false, .. } => {
                                    // draw in step mode only
                                },
                                _ => {}
                            }
                        }
                        DrawMode::Cursor => {
                            if !self.state.selection.data.is_empty() {
                                match self.state.modifiers {
                                    keyboard::Modifiers { alt: false, logo: true, .. } => {
                                        // micro timing
                                    },
                                    keyboard::Modifiers { alt: true, logo: false, .. } => {
                                        // duplication
                                    },
                                    keyboard::Modifiers { alt: true, logo: true, .. } => {
                                        // micro timing + duplication
                                    },
                                    keyboard::Modifiers { alt: false, logo: false, .. } => {
                                        // step move only
                                    },
                                    _ => {}
                                }
                            } else {
                                // draw selection and add grid events to selection
                                self.state.selection_rectangle = Some(bounds);
                            }
                        }
                    }
                }
            }
            Actions::DoubleClick(cursor) => {
                match self.state.base_pattern.to_owned().get_hovered(cursor) {
                    Some(grid_event) => {
                        // remove event
                    }
                    None => {
                        // add event
                    }
                }
            }
            Actions::Click(cursor) => {
                match self.state.selection.to_owned().get_hovered(cursor) {
                    Some((grid_id, grid_event)) => {
                        match self.state.modifiers {
                            keyboard::Modifiers { shift: true, .. } => {
                                if self.state.selection.data.contains_key(&grid_id) {
                                    self.state.selection.data.remove(&grid_id);
                                } else {
                                    self.state.selection.data.insert(grid_id, grid_event);
                                }
                            },
                            _ => {
                                // empty selection and add event
                                self.state.selection = GridPattern::new();
                                self.state.selection.data.insert(grid_id, grid_event);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Actions::KeyAction(key_code) => {
                match key_code {
                    keyboard::KeyCode::Escape | keyboard::KeyCode::Delete => {
                        // reset dragging state 
                        self.state.reset();
                    },
                    keyboard::KeyCode::B => {
                        // reset dragging state 
                        match self.state.draw_mode {
                            DrawMode::Pen => {
                                self.state.draw_mode = DrawMode::Cursor;
                            }
                            DrawMode::Cursor => {
                                self.state.draw_mode = DrawMode::Pen;
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

/**
* Actions we should implement :
* - double ckick => Toggle (Add or Remove) event on cursor position (if Add => quantized)
* - dragging => if event on drag origin : Move event (x & y: quantized) (keep locked events
* - shift + click => if on event : add event to / remove event from selection
* - dragging over a threshold before the beginning of an event => remove Event (keep locked events)
* - selection + dragging  => Move Selection (x & y: quantized) (keep locked events)
* - selection + dragging + cmd => Move (x: micro timing, y: quantized) (keep locked events)
* - selection + dragging + alt | selection + alt + dragging 
*   => Duplicate + Move Duplicates (x & y: quantized) (keep locked events)
* - selection + dragging + alt + cmd | selection + alt + cmd + dragging 
*   => Duplicate + Move Duplicates (x: micro timing, y: quantized) (keep locked events)
* - release dragging: commit changes to locked events
* - (selection + dragging | selection + dragging + cmd 
*   | selection + dragging + alt | selection + alt + dragging
*   | selection + dragging + alt + cmd | selection + alt + cmd + dragging) + (Esc or Del)
*   => Reset State (reset dragging and selection)
* - selection + cmd + dragging => setVelocity
* - selection + modifier key Esc or Del => reset state (empty) => Delete Selection
*/

pub enum Actions {
    Drag(Rectangle),
    DoubleClick(Point),
    Click(Point),
    KeyAction(keyboard::KeyCode)
}

#[derive(Debug, Clone, Copy)]
pub enum DrawMode {
    Pen,
    Cursor
}

#[derive(Debug, Clone)]
pub struct State {
    base_pattern: GridPattern,
    output_pattern: GridPattern,
    selection: GridPattern,
    draw_mode: DrawMode,
    is_dragging: bool,
    is_logo_pressed: bool,
    drag_origin_x: f32,
    drag_origin_y: f32,
    selection_rectangle: Option<Rectangle>,
    modifiers: keyboard::Modifiers,
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
            base_pattern: base_pattern.clone(),
            output_pattern: base_pattern.clone(),
            selection: GridPattern::new(),
            draw_mode: DrawMode::Pen,
            is_dragging: false,
            is_logo_pressed: false,
            drag_origin_x: 0.0,
            drag_origin_y: 0.0,
            selection_rectangle: None,
            modifiers: Default::default(),
            last_click: None
        }
    }

    pub fn reset(&mut self) {
        self.selection = GridPattern::new();
        self.is_logo_pressed = false;
        self.is_logo_pressed = false;
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
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    if self.state.is_dragging {
                        let bounds_height = layout.bounds().height;

                        if bounds_height > 0.0 {
                            let drag_bounds = Rectangle {
                                x: self.state.drag_origin_x,
                                y: self.state.drag_origin_y,
                                width: cursor_position.x - self.state.drag_origin_x,
                                height: cursor_position.y - self.state.drag_origin_y
                            };

                            self.on_action(Actions::Drag(drag_bounds), bounds);

                            return event::Status::Captured;
                        }
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if bounds.contains(cursor_position) {
                        let click = mouse::Click::new(
                            cursor_position,
                            self.state.last_click,
                        );

                        match click.kind() {
                            mouse::click::Kind::Single => {
                                self.state.is_dragging = true;
                                self.state.drag_origin_x = cursor_position.x;
                                self.state.drag_origin_y = cursor_position.y;
                                self.on_action(Actions::Click(cursor_position), bounds);
                            },
                            mouse::click::Kind::Double => {
                                self.on_action(Actions::DoubleClick(cursor_position), bounds);
                            },
                            _ => {
                                self.state.is_dragging = false;
                            }
                        }

                        self.state.last_click = Some(click);

                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.state.is_dragging = false;
                    self.state.selection_rectangle = None;

                    return event::Status::Captured;
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { modifiers, key_code } => {
                    self.state.modifiers = modifiers;

                    // set velocity mode if not dragging
                    if !self.state.is_dragging && self.state.modifiers.logo {
                        self.state.is_logo_pressed = true;
                    } else if !self.state.modifiers.logo {
                        // or reset it
                        self.state.is_logo_pressed = false;
                    }

                    self.on_action(Actions::KeyAction(key_code), bounds);

                    return event::Status::Captured;
                }
                keyboard::Event::KeyReleased { modifiers, .. } => {
                    self.state.modifiers = modifiers;

                    // reset velocity mode
                    if !self.state.modifiers.logo {
                        self.state.is_logo_pressed = false;
                    }

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
            self.state.to_owned(),
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
        state: State,
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
