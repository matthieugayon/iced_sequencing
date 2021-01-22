use std::{collections::HashMap, fmt::Debug};

use iced_native::{
    event, keyboard, layout, mouse, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Widget
};

use std::hash::Hash;

use ganic_no_std::{NUM_PERCS, NUM_STEPS, pattern::Pattern};

pub const STEP_MARGIN_RIGHT: f32 = 4.0;
pub const TRACK_MARGIN_BOTTOM: f32 = 16.0;
pub const CONTAINER_PADDING: f32 = 12.0;
pub const EVENT_WIDTH: f32 = 30.0;
pub const DEFAULT_VELOCITY: f32 = 1.0;
pub const OFFSET_THRESHOLD: f32 = 0.15;

pub fn get_step_dimensions(bounds: Rectangle) -> Size {
    return Size {
        width: (bounds.width - (2.0 * CONTAINER_PADDING)) / NUM_STEPS as f32,
        height: ((bounds.height - (2.0 * CONTAINER_PADDING)) / NUM_PERCS as f32) - TRACK_MARGIN_BOTTOM
    }    
}

#[derive(Debug, Clone, Copy)]
pub struct GridEvent {
    offset: f32,
    velocity: f32,
    selected: bool
}

impl Default for GridEvent {
    fn default() -> Self {
        GridEvent {
            offset: 0.0,
            velocity: DEFAULT_VELOCITY,
            selected: false
        }
    }
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

    pub fn get_hovered(self, cursor: Point, bounds: Rectangle) -> Option<((usize, usize), GridEvent)> {
        let step_size = get_step_dimensions(bounds);
        
        self.data.into_iter()
            .find(|((step, track), grid_event)| {
                let grid_event_rect = Rectangle {
                    x: CONTAINER_PADDING + (grid_event.offset * step_size.width) + (*step as f32 * (step_size.width + STEP_MARGIN_RIGHT)),
                    y: CONTAINER_PADDING + (*track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM)),
                    width: step_size.width,
                    height: step_size.height
                };
    
                grid_event_rect.contains(cursor)
            })
    }

    pub fn select(&mut self, grid_id: (usize, usize), modifiers: keyboard::Modifiers) {
        match modifiers {
            keyboard::Modifiers { shift: true, .. } => {
                match self.data.get_mut(&grid_id) {
                    Some(grid_event) => {
                        if grid_event.selected {
                            grid_event.selected = false;
                        } else {
                            grid_event.selected = true;
                        }
                    }
                    None => {}
                }
            },
            _ => {
                // empty selection and add event
                self.data.iter_mut().for_each(|((step, track), grid)| {

                    if *step == grid_id.0 && *track == grid_id.1 {
                        grid.selected = true;
                    } else {
                        grid.selected = false;
                    }
                });
            }
        }
    }

    pub fn select_area(&mut self, selection: Rectangle, bounds: Rectangle) {
        let step_size = get_step_dimensions(bounds);

        self.data.iter_mut().for_each(|((step, track), grid_event)| {
            let event_origin = get_event_absolute_position(*step, *track, grid_event.offset, bounds);
            let event_bounds = Rectangle {
                x: event_origin.x,
                y: event_origin.y,
                width: EVENT_WIDTH,
                height: step_size.height,
            };

            match selection.intersection(&event_bounds) {
                Some(_) => {
                    grid_event.selected = true;
                }
                None => {
                    grid_event.selected = false;
                }
            }
        });
    }

    pub fn get_selection(self) -> Vec<(usize, usize)> {
        self.data
            .into_iter()
            .filter(|(_, grid_event)| grid_event.selected)
            .map(|(grid_id, _)| grid_id)
            .collect()
    }

    pub fn empty_selection(&mut self) {
        self.data
            .iter_mut()
            .filter(|(_, grid_event)| grid_event.selected)
            .for_each(|(_, grid_event)| {
                grid_event.selected = false;
            });
    }
}

fn is_point_inside_draggable_area(point: Point, bounds: Rectangle) -> bool {
    let draggable_area = Rectangle {
        x: CONTAINER_PADDING,
        y: CONTAINER_PADDING,
        width: bounds.width - 2.0 * CONTAINER_PADDING,
        height: bounds.height - 2.0 * CONTAINER_PADDING
    };
    
    return draggable_area.contains(point)
}

fn get_hovered_step(cursor: Point, bounds: Rectangle, bounded: bool) -> Option<(usize, usize, f32)> {
    let step_size = get_step_dimensions(bounds);
    
    if bounded {
        if is_point_inside_draggable_area(cursor, bounds) {
            let step = ((cursor.x - CONTAINER_PADDING) / (step_size.width + STEP_MARGIN_RIGHT)) as usize;
            let track = ((cursor.y - CONTAINER_PADDING) / (step_size.height + TRACK_MARGIN_BOTTOM)) as usize;
            let offset = cursor.x - (CONTAINER_PADDING + step as f32 * (step_size.width + STEP_MARGIN_RIGHT));

            Some((step, track, offset))
        } else {
            None
        }
    } else {
        let step = (((cursor.x - CONTAINER_PADDING) / (step_size.width + STEP_MARGIN_RIGHT)) as usize).min(0).max(NUM_STEPS);
        let track = (((cursor.y - CONTAINER_PADDING) / (step_size.height + TRACK_MARGIN_BOTTOM)) as usize).min(0).max(NUM_PERCS);
        let offset = (cursor.x - (CONTAINER_PADDING + step as f32 * (step_size.width + STEP_MARGIN_RIGHT))).min(-0.99).max(0.99);

        Some((step, track, offset))
    }
}

fn get_event_absolute_position(step: usize, track: usize, offset: f32, bounds: Rectangle) -> Point {
    let step_size = get_step_dimensions(bounds);

    return Point {
        x: CONTAINER_PADDING + (offset * step_size.width) + step as f32 * (step_size.width + STEP_MARGIN_RIGHT),
        y: CONTAINER_PADDING + track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM)
    }
}

pub enum Direction {
    Positive,
    Negative
}

fn move_selection(
    drag_bounds: Rectangle,
    bounds: Rectangle,
    origin_event: GridEvent,
    quantized: bool,
    duplicate: bool,
    base_pattern: GridPattern) -> HashMap<(usize, usize), GridEvent> {

    let mut output: HashMap<(usize, usize), GridEvent>  = base_pattern.data.clone();

    let step_size = get_step_dimensions(bounds);

    // we iterate here over our source of truth , base_pattern
    // but we apply temporary changes on output_pattern
    for ((step, track), event) in base_pattern.data.to_owned() {
        if event.selected {
            let event_position = get_event_absolute_position(step, track, event.offset, bounds);
            let next_event_position = Point { x: event_position.x + drag_bounds.width , y: event_position.y + drag_bounds.height };
            
            // with unbounded flag we must get smthg back
            let cursor_step = get_hovered_step(next_event_position, bounds, false).unwrap();

            // cast y position to new track usize
            // WARNING : ther's a possibility of direction mistake here
            let track_offset: isize = (drag_bounds.height / (step_size.height + TRACK_MARGIN_BOTTOM)) as isize;
            let next_track = ((track as isize + track_offset) % NUM_PERCS as isize) as usize;

            // we move in quantized fashion only when whantized mode on 
            // and dragging width is superior to STEP_WIDTH + STEP_MARGIN_RIGHT

            // next step event, if there is any
            // let next_step_event = base_pattern.data.get();
            let same_step_offset_left = (step_size.width + STEP_MARGIN_RIGHT) * origin_event.offset;
            let same_step_offset_right = (step_size.width + STEP_MARGIN_RIGHT) - origin_event.offset * (step_size.width + STEP_MARGIN_RIGHT);

            // if we are quantized and drag width is superior to the bounds of the original selected event
            match output.get(&(step, track)) {
                Some(&event_to_process) => {
                    if quantized && (drag_bounds.width > same_step_offset_right) | (drag_bounds.width < (-1.0 * same_step_offset_left)) {
                        if (step != cursor_step.0) | (track != next_track) {
                            if duplicate {
                                // select event
                                let original_event = output.get_mut(&(step, track)).unwrap();
                                original_event.selected = true

                            } else {
                                output.remove(&(step, track));
                            }

                            match base_pattern.data.get(&(cursor_step.0, next_track)) {
                                Some(_) => {
                                    output.remove(&(cursor_step.0, next_track));
                                    output.insert((cursor_step.0, next_track), GridEvent {
                                        offset: event_to_process.offset,
                                        velocity: event_to_process.velocity,
                                        selected: true
                                    });
                                }
                                None => {
                                    output.insert((cursor_step.0, next_track), GridEvent {
                                        offset: event_to_process.offset,
                                        velocity: event_to_process.velocity,
                                        selected: true
                                    });
                                }
                            }
                        }
                    } else {
                        // unquantized mess ...
                        // we have few difficult cases:
                        // - if the next step holds an event with negative offset
                        // - if the previous step has an event , keep event on same step with negative offset, till it collides with offset of previous event
                        // - main rule : only used positive offset unless there is already an event on the same step
                        // - Duplicates : on small values , we will start with negative offsets on next step if we move to the right,
                        //   and juste move selection if we move to the left ... phew 
                        
                        let hovered_event = base_pattern.data.get(&(cursor_step.0, next_track));
                        let next_track_event = base_pattern.data.get(&(cursor_step.0 + 1, next_track));
        
                        if duplicate {
                            // select event
                            let original_event = output.get_mut(&(step, track)).unwrap();
                            original_event.selected = true
                        }

                        match hovered_event {
                            Some(hovered_grid_event) => {
                                if (step != cursor_step.0) | (track != next_track) {
                                    output.remove(&(step, track));
                                }

                                match next_track_event {
                                    Some(next_track_grid_event) => {
                                        // event hovered by the current moved step and event on next step same track
                                        // we have to remove the current one if next event offset is not colliding
                                        // otherwise we keep current one 
                                        if next_track_grid_event.offset < 0.0 && cursor_step.2 >= (1.0 + next_track_grid_event.offset - OFFSET_THRESHOLD)  {
                                            // TODO : could we mutate instead ? 
                                            output.remove(&(cursor_step.0 + 1, next_track));
                                            output.insert((cursor_step.0 + 1, next_track), GridEvent {
                                                offset: cursor_step.2 - 1.0,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        } else {
                                            // TODO : could we mutate instead ? 
                                            output.remove(&(cursor_step.0, next_track));
                                            output.insert((cursor_step.0, next_track), GridEvent {
                                                offset: cursor_step.2,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        }
                                    }
                                    None => {
                                        if cursor_step.2 <= hovered_grid_event.offset + OFFSET_THRESHOLD {
                                            output.remove(&(cursor_step.0, next_track));

                                            if cursor_step.2 <= 0.5 {
                                                output.insert((cursor_step.0, next_track), GridEvent {
                                                    offset: cursor_step.2,
                                                    velocity: event_to_process.velocity,
                                                    selected: true
                                                });
                                            } else {
                                                output.insert((cursor_step.0 + 1, next_track), GridEvent {
                                                    offset: cursor_step.2 - 1.0,
                                                    velocity: event_to_process.velocity,
                                                    selected: true
                                                });
                                            }
                                            
                                        } else {
                                            output.insert((cursor_step.0 + 1, next_track), GridEvent {
                                                offset: cursor_step.2 - 1.0,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        }
                                    }
                                }
                            }
                            None => {
                                if !duplicate {
                                    output.remove(&(step, track));
                                }

                                match next_track_event {
                                    Some(next_track_grid_event) => {
                                        // no event hovered by the current moved step but event on next step same track
                                        // i.e we need to check if next event as a negative offset 
                                        // if yes and it collides with calculated new offset, remove next step event
                                        if next_track_grid_event.offset < 0.0 && cursor_step.2 >= (1.0 + next_track_grid_event.offset - OFFSET_THRESHOLD)  {
                                            output.remove(&(cursor_step.0 + 1, next_track));
                                            output.insert((cursor_step.0 + 1, next_track), GridEvent {
                                                offset: cursor_step.2 - 1.0,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        } else {
                                            output.insert((cursor_step.0, next_track), GridEvent {
                                                offset: cursor_step.2,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        }
                                    }
                                    None => {
                                        // no event hovered by the current moved step and no event on next step same track
                                        // i.e we must have dragged the event to a new step
                                        if cursor_step.2 <= 0.5 {
                                            output.insert((cursor_step.0, next_track), GridEvent {
                                                offset: cursor_step.2,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        } else {
                                            output.insert((cursor_step.0 + 1, next_track), GridEvent {
                                                offset: cursor_step.2 - 1.0,
                                                velocity: event_to_process.velocity,
                                                selected: true
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                None => {}
            }
        }
    }

    return output.to_owned();
}

impl From<Pattern> for GridPattern {
    fn from(pattern: Pattern) -> Self {
        let mut grid = GridPattern::new();

        for (i, step) in pattern.iter().enumerate() {
            for (j, perc) in step.iter().enumerate() {
                if perc[0] > 0.0 {
                    grid.data.insert((i, j), GridEvent { velocity: perc[0], offset: perc[1], selected: false });
                }
            }
        }

        grid
    }
}

impl From<GridPattern> for Pattern {
    fn from(grid: GridPattern) -> Self {
        let mut pattern = Pattern::new();

        println!("{:?}", grid.data);


        for ((step, track), event) in grid.data {
            pattern.data[step][track][0] = event.velocity;
            pattern.data[step][track][1] = event.offset;
        }

        pattern
    }
}

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

    pub fn create_selection_area(&mut self, bounds: Rectangle) {
        self.state.selection_rectangle = Some(bounds);

    }

    pub fn on_action(&mut self, action: Actions, bounds: Rectangle) {

        println!("{:?}", action);

        match action {
            Actions::Drag(
                cursor,
                // is_cursor_inside_draggable_area,
                is_origin_inside_draggable_area,
                drag_bounds,
                origin_hovered_event,
                // prev_hovered_event,
                hovered_event,
                // hovered_step,
                hovered_event_change
            ) => {
                let step_size = get_step_dimensions(bounds);

                if self.state.is_logo_pressed {
                    // velocity mode
                    let drag_height_max = 3.0 * (step_size.height + TRACK_MARGIN_BOTTOM);
                    let velocity = 1.0 - (drag_bounds.height.max(drag_height_max) / drag_height_max);

                    self.state.output_pattern.data
                        .iter_mut()
                        .filter(|grid_event| grid_event.1.selected)
                        .for_each(|grid_event| {
                            grid_event.1.velocity = velocity;
                        });

                } else {
                    match self.state.draw_mode {
                        DrawMode::Pen => {
                            // For your information : we only draw on the same track / x axis
                            // we use the y axis for the velocity
                            // if it's the first drag position we're treating 
                            // or if the current hovered event is not the equal to the previous one
                            // => so the current hovered event is an event not having been drawn by the current dragging session
                            if (self.state.last_drag_position == cursor) | hovered_event_change.0 {

                                match hovered_event {
                                    // if an event is currently hovered, remove it
                                    Some((grid_id, ..)) => {
                                        self.state.base_pattern.to_owned().data.remove(&grid_id);
                                    }
                                    // if none, create one, but apply logo modifier (micro timing)
                                    None => {
                                        // get underlying step position, there is still a risk to have none (container padding)
                                        // offset = cursor x offset whithin that step 
                                        // velocity max = 3 * the step height plus the track margin
                                        let drag_height_max = 3.0 * (step_size.height + TRACK_MARGIN_BOTTOM);
                                        let mut grid_event = GridEvent {
                                            offset: 0.0,
                                            velocity: 1.0 - (drag_bounds.height.max(drag_height_max) / drag_height_max),
                                            selected: true
                                        };

                                        match get_hovered_step(Point { x: cursor.x, y: self.state.drag_origin.y }, bounds, false) {
                                            Some((step, track, offset)) => {
                                                match self.state.modifiers {
                                                    keyboard::Modifiers { logo: true, .. } => {
                                                        // with logo modifier on, we set the offset of the steps 
                                                        // to the cursor offset of the drag start position whithin its hovered step
                                                        grid_event.offset = offset;
                                                        self.state.base_pattern.data.insert((step, track), grid_event);
                                                    },
                                                    keyboard::Modifiers { logo: false, .. } => {
                                                        // with logo off, event is quantized to the track step number division
                                                        // => offset = 0
                                                        self.state.base_pattern.data.insert((step, track), grid_event);
                                                    }
                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                }

                                // we made changes to base_pattern
                                // => replicate base pattern to output pattern
                                self.state.output_pattern = self.state.base_pattern.clone();
                            }
                        }
                        DrawMode::Cursor => {
                            if is_origin_inside_draggable_area {
                                match origin_hovered_event {
                                    // we can only drag if origin of dragging is hovering an event
                                    Some((_, grid_event)) => {
                                        // TODO double check if we should check that on output_patern or base_pattern
                                                                                
                                        if !self.state.output_pattern.to_owned().get_selection().is_empty() {
                                            let quantize = self.state.modifiers.logo;
                                            let duplicate = self.state.modifiers.alt;

                                            // replace output pattern with new generated one with moving rules
                                            self.state.output_pattern.data = move_selection(
                                                drag_bounds,
                                                bounds,
                                                grid_event,
                                                quantize,
                                                duplicate,
                                                self.state.base_pattern.to_owned()
                                            );

                                        } else {
                                            // draw selection and add grid events to selection
                                            self.create_selection_area(drag_bounds);
                                        }        
                                    }
                                    None => {
                                        // draw selection and add grid events to selection
                                        self.create_selection_area(drag_bounds);
                                    }
                                }
                            } else {
                                // draw selection and add grid events to selection
                                self.create_selection_area(drag_bounds);
                            }   
                        }
                    }
                }
            }
            Actions::DoubleClick(cursor) => {
                match self.state.base_pattern.to_owned().get_hovered(cursor, bounds) {
                    Some((grid_id, _grid_event)) => {
                        // remove event
                        self.state.base_pattern.data.remove(&grid_id);
                    }
                    None => {
                        // add event
                        match get_hovered_step(cursor, bounds, true) {
                            Some((step, track, ..)) => {
                                self.state.base_pattern.data.insert((step, track), GridEvent::default());
                            }
                            None => {}
                        }
                    }
                }

                // replicate base pattern to output pattern
                self.state.output_pattern = self.state.base_pattern.clone();
            }
            Actions::Click(cursor) => {

                println!("Action click {:?}", self.state.base_pattern);

                match self.state.base_pattern.to_owned().get_hovered(cursor, bounds) {
                    Some((grid_id, _)) => {
                        self.state.base_pattern.to_owned().select(grid_id, self.state.modifiers);
                        self.state.output_pattern.to_owned().select(grid_id, self.state.modifiers);
                    }
                    _ => {}
                }
            }
            Actions::KeyAction(key_code) => {
                match key_code {
                    keyboard::KeyCode::Escape | keyboard::KeyCode::Delete => {
                        // reset dragging state 
                        self.state.reset_selection();

                        // reset output pattern
                        self.state.output_pattern = self.state.base_pattern.clone();
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

#[derive(Debug)]
pub enum Actions {
    Drag(
        Point,
        bool,
        Rectangle,
        Option<((usize, usize), GridEvent)>,
        Option<((usize, usize), GridEvent)>,
        (bool, bool)
    ),
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
    draw_mode: DrawMode,
    is_dragging: bool,
    is_logo_pressed: bool,
    drag_origin: Point,
    last_drag_position: Point,
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
            draw_mode: DrawMode::Pen,
            is_dragging: false,
            is_logo_pressed: false,
            drag_origin: Point { x: 0.0, y: 0.0 },
            last_drag_position: Point { x: 0.0, y: 0.0 },
            selection_rectangle: None,
            modifiers: Default::default(),
            last_click: None
        }
    }

    pub fn reset_selection(&mut self) {
        self.selection_rectangle = None;
        self.is_logo_pressed = false;
        self.output_pattern.empty_selection();
        self.base_pattern.empty_selection();
    }

    pub fn reset_dragging_state(&mut self) {
        self.is_dragging = false;
        self.selection_rectangle = None;
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

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    if self.state.is_dragging {
                        let bounds_height = layout.bounds().height;

                        if  bounds_height > 0.0 
                            && cursor_position.x - self.state.drag_origin.x > 0.01
                            && cursor_position.y - self.state.drag_origin.y > 0.01 {

                            // we'll use that to default Option results with map_or
                            let none_value: usize = 1000;

                            // origin x cursor position, we check if it holds an event or not
                            let origin_hovered_event = self.state.output_pattern
                                .to_owned()
                                .get_hovered(self.state.drag_origin, bounds);

                            // previous x cursor position, we check if it holds an event or not
                            let prev_hovered_event = self.state.output_pattern
                                .to_owned()
                                .get_hovered(self.state.last_drag_position, bounds);

                            // current x cursor position, we check if it holds an event or not
                            let hovered_event = self.state.output_pattern
                                .to_owned()
                                .get_hovered(cursor_position, bounds);

                            // get unbounded cursor step hover
                            // let hovered_step = get_hovered_step(cursor_position, bounds, false);   
                                
                            // let's just extract the (step, track) to compare them
                            // (we cannot have two events on the same step, so they must either be different or the same one)
                            let prev_hovered_id = prev_hovered_event.map_or((none_value, none_value), |x| x.0);
                            let hovered_id = hovered_event.map_or((none_value, none_value), |x| x.0);

                            // if we left an event or entered a new one
                            let hovered_event_change = (prev_hovered_id.0 != hovered_id.0, prev_hovered_id.1 != hovered_id.1);

                            let drag_bounds = Rectangle {
                                x: self.state.drag_origin.x,
                                y: self.state.drag_origin.y,
                                width: cursor_position.x - self.state.drag_origin.x,
                                height: cursor_position.y - self.state.drag_origin.y
                            };

                            let is_origin_inside_draggable_area = is_point_inside_draggable_area(self.state.drag_origin, bounds);
                            // let is_cursor_inside_draggable_area = is_point_inside_draggable_area(self.state.drag_origin, bounds);

                            self.on_action(Actions::Drag(
                                cursor_position,
                                // is_cursor_inside_draggable_area,
                                is_origin_inside_draggable_area,
                                drag_bounds,
                                origin_hovered_event,
                                // prev_hovered_event,
                                hovered_event,
                                // hovered_step,
                                hovered_event_change
                            ), bounds);

                            messages.push((self.on_change)(Pattern::from(self.state.output_pattern.to_owned())));

                            return event::Status::Captured;
                        }

                        self.state.last_drag_position = cursor_position
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
                                self.state.drag_origin = cursor_position;
                                self.state.last_drag_position = cursor_position;
                                self.on_action(Actions::Click(cursor_position), bounds);
                            },
                            mouse::click::Kind::Double => {
                                self.on_action(Actions::DoubleClick(cursor_position), bounds);
                            },
                            _ => {
                                self.state.reset_dragging_state();
                            }
                        }

                        self.state.last_click = Some(click);

                        messages.push((self.on_change)(Pattern::from(self.state.output_pattern.to_owned())));

                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.state.reset_dragging_state();

                    // commit ouput pattern changes to base_patern
                    self.state.base_pattern.data = self.state.output_pattern.data.clone();

                    messages.push((self.on_change)(Pattern::from(self.state.output_pattern.to_owned())));

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

                    messages.push((self.on_change)(Pattern::from(self.state.output_pattern.to_owned())));

                    return event::Status::Captured;
                }
                keyboard::Event::KeyReleased { modifiers, .. } => {
                    self.state.modifiers = modifiers;

                    // reset velocity mode
                    if !self.state.modifiers.logo {
                        self.state.is_logo_pressed = false;

                        messages.push((self.on_change)(Pattern::from(self.state.output_pattern.to_owned())));
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
