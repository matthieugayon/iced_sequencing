use std::collections::HashMap;
use iced_native::{Point, Rectangle, Size};
use ganic_no_std::{pattern::Pattern, NUM_PERCS, NUM_STEPS};
use crate::native::grid::State;

pub const TRACK_MARGIN_BOTTOM: f32 = 2.0;
pub const DEFAULT_VELOCITY: f32 = 1.0;
pub const OFFSET_THRESHOLD: f32 = 0.05;

pub fn get_step_dimensions(size: Size) -> Size {
    return Size {
        width: get_step_width(size),
        height: get_track_height(size) - TRACK_MARGIN_BOTTOM,
    };
}

pub fn get_track_height(size: Size) -> f32 {
    return size.height / NUM_PERCS as f32;
}

pub fn get_step_width(size: Size) -> f32 {
    return size.width / (NUM_STEPS + 2) as f32;
}

pub fn get_event_bounds(step: usize, track: usize, offset: f32, size: Size) -> Rectangle {
    let step_width = get_step_width(size);
    let track_height = get_track_height(size);

    Rectangle {
        x: (offset + 1. + step as f32) * step_width,
        y: track as f32 * track_height,
        width: step_width,
        height: track_height - TRACK_MARGIN_BOTTOM,
    }
}

pub fn get_hovered_step(cursor: Point, bounds: Rectangle, quantized: bool) -> (usize, usize, f32) {
    let size = bounds.size();
    let step_width = get_step_width(size);
    let track_height = get_track_height(size);

    let step_with_offset = (cursor.x - bounds.x - step_width) / step_width;
    let mut step = (step_with_offset as usize).max(0).min(NUM_STEPS - 1);
    let track = (((cursor.y - bounds.y) / track_height) as usize)
        .max(0)
        .min(NUM_PERCS - 1);

    let offset = {
        match quantized {
            true => 0.,
            false => {
                let unprocessed_offset = step_with_offset - step as f32;

                if step_with_offset < 0. {
                    step_with_offset.max(-0.99).min(0.)
                } else if step == NUM_STEPS - 1 {
                    unprocessed_offset.min(0.99)
                } else if unprocessed_offset > 0.5 {
                    step += 1;
                    1. - unprocessed_offset
                } else {
                    unprocessed_offset
                }
            }
        }
    };

    (step, track, offset)
}

pub fn get_hovered_track(cursor: Point, bounds: Rectangle) -> usize {
    (((cursor.y - bounds.y) / get_track_height(bounds.size())) as usize)
        .max(0)
        .min(NUM_PERCS - 1)
}

pub fn convert_rectangle_to_relative_coordinates(geometry: Rectangle, bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: geometry.x - bounds.x,
        y: geometry.y - bounds.y,
        ..geometry
    }
}

pub fn convert_point_to_relative_coordinates(geometry: Rectangle, bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: geometry.x - bounds.x,
        y: geometry.y - bounds.y,
        ..geometry
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridEvent {
    pub offset: f32,
    pub velocity: f32,
    pub selected: bool,
}

impl Default for GridEvent {
    fn default() -> Self {
        GridEvent {
            offset: 0.0,
            velocity: DEFAULT_VELOCITY,
            selected: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridPattern {
    pub data: HashMap<(usize, usize), GridEvent>,
}

impl GridPattern {
    pub fn new() -> Self {
        GridPattern {
            data: HashMap::new(),
        }
    }

    pub fn get_hovered(
        &self,
        cursor: Point,
        bounds: Rectangle,
    ) -> Option<(&(usize, usize), &GridEvent)> {
        let size = bounds.size();
        let step_width = get_step_width(size);
        let track_height = get_track_height(size);

        self.data.iter().find(|((step, track), grid_event)| {
            let grid_event_rect = Rectangle {
                x: ((1. + *step as f32 + grid_event.offset) * step_width) + bounds.x,
                y: (*track as f32 * track_height) + bounds.y,
                width: step_width,
                height: track_height - TRACK_MARGIN_BOTTOM,
            };

            grid_event_rect.contains(cursor)
        })
    }

    pub fn toggle_select(&mut self, grid_id: (usize, usize)) {
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
    }

    pub fn select_all(&mut self) {
        self.data.iter_mut().for_each(|(_, grid)| {
            grid.selected = true;
        });
    }

    pub fn select_one(&mut self, grid_id: (usize, usize)) {
        self.data.iter_mut().for_each(|((step, track), grid)| {
            if *step == grid_id.0 && *track == grid_id.1 {
                grid.selected = true;
            } else {
                grid.selected = false;
            }
        });
    }

    pub fn select_area(&mut self, selection: Rectangle, size: Size) {
        self.data
            .iter_mut()
            .for_each(|((step, track), grid_event)| {
                let event_bounds = get_event_bounds(*step, *track, grid_event.offset, size);

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

    pub fn toggle_area(&mut self, selection: Rectangle, size: Size) {
        self.data
            .iter_mut()
            .for_each(|((step, track), grid_event)| {
                let event_bounds = get_event_bounds(*step, *track, grid_event.offset, size);

                match selection.intersection(&event_bounds) {
                    Some(_) => {
                        grid_event.selected = !grid_event.selected;
                    }
                    _ => {}
                }
            });
    }

    pub fn remove_selection(&mut self) {
        for ((step, track), event) in self.data.to_owned() {
            if event.selected {
                self.data.remove(&(step, track));
            }
        }
    }

    pub fn empty_selection(&mut self) {
        self.data.iter_mut().for_each(|(_, grid)| {
            grid.selected = false;
        });
    }

    pub fn move_selection_quantized(
        &self,
        bounds: Rectangle,
        drag_bounds: Rectangle,
        cursor: Point,
        origin_event: (usize, usize, GridEvent),
    ) -> (f32, isize) {
        // cursor is normalized and padded, it cannto be outside : NOT TRUE ANYMORE
        // so it must be hovering a step
        // let step_offset: isize = hovered_step.0 as isize - origin_grid_id.0 as isize;

        let size = bounds.size();
        let step_size = get_step_dimensions(size);
        let hovered_track = get_hovered_track(cursor, bounds);
        let track_offset: isize = hovered_track as isize - origin_event.1 as isize;

        let max_positive_offset: f32 = (NUM_STEPS - origin_event.0) as f32 - 1.;
        let min_negative_offset: f32 = -1. * origin_event.0 as f32 - origin_event.2.offset;

        let mut selection_step_offset = {
            if drag_bounds.width >= 0. {
                (drag_bounds.width / step_size.width).floor()
            } else {
                (drag_bounds.width / step_size.width).ceil()
            }
        };

        if origin_event.2.offset != 0. {
            let step_offset = (drag_bounds.width / (step_size.width * 0.5)) as isize;
            let wrapped_offset = step_offset % 2;
            selection_step_offset = step_offset as f32 * 0.5;

            if wrapped_offset != 0 {
                if origin_event.2.offset < 0. {
                    selection_step_offset =
                        (step_offset as f32 * 0.5).floor() + (-1. * origin_event.2.offset);
                } else {
                    selection_step_offset =
                        (step_offset as f32 * 0.5).floor() + (1. - origin_event.2.offset);
                }
            }
        }

        (
            selection_step_offset
                .min(max_positive_offset)
                .max(min_negative_offset),
            track_offset,
        )
    }

    pub fn move_selection_unquantized(
        &self,
        bounds: Rectangle,
        drag_bounds: Rectangle,
        cursor: Point,
        origin_event: (usize, usize, GridEvent),
    ) -> (f32, isize) {
        let size = bounds.size();
        let max_positive_offset: f32 =
            (NUM_STEPS - origin_event.0) as f32 - origin_event.2.offset - OFFSET_THRESHOLD;
        let min_negative_offset: f32 = -1. * origin_event.0 as f32 - origin_event.2.offset - 0.99;
        let step_size = get_step_dimensions(size);
        let step_offset = (drag_bounds.width / step_size.width)
            .min(max_positive_offset)
            .max(min_negative_offset);
        let hovered_track = get_hovered_track(cursor, bounds);
        let track_offset: isize = hovered_track as isize - origin_event.1 as isize;

        (step_offset, track_offset)
    }

    pub fn move_selection(&mut self, step_offset: f32, track_offset: isize) {
        // init empty hashmap
        let mut output: HashMap<(usize, usize), GridEvent> = HashMap::new();

        // copy non selected events
        for ((step, track), event) in self.data.to_owned() {
            if !event.selected {
                output.insert((step, track), event);
            }
        }

        // clone version to query events
        let output_map = output.to_owned();

        for ((step, track), event) in self.data.to_owned() {
            if event.selected {
                // next step
                let next_step_offset =
                    (step as f32 + event.offset + step_offset + NUM_STEPS as f32)
                        % NUM_STEPS as f32;
                let next_step = next_step_offset.floor() as usize;
                let next_offset = next_step_offset - next_step as f32;

                // next track
                let next_track =
                    (track as isize + track_offset + NUM_PERCS as isize) as usize % NUM_PERCS;

                // check events at next locations
                let next_event = output_map.get(&(next_step, next_track));
                let next_event_plus_one =
                    output_map.get(&((next_step + 1) % NUM_STEPS, next_track));

                // build a tuple with all that data, then we pattern match on it
                let cases: (bool, f32, Option<&GridEvent>, Option<&GridEvent>) = (
                    step_offset >= 0.,
                    next_offset,
                    next_event,
                    next_event_plus_one,
                );

                self.replace_event(cases, &mut output, next_step, next_track, event);
            }
        }

        self.data = output;
    }

    fn replace_event(
        &self,
        cases: (bool, f32, Option<&GridEvent>, Option<&GridEvent>),
        output: &mut HashMap<(usize, usize), GridEvent>,
        step: usize,
        track: usize,
        event: GridEvent,
    ) {
        match cases {
            // we are dragging to the right
            // offset is <= 0.5
            // nothing on the step to be dragged on, nothing on the next one either
            (drag_right, offset, None, None) 
                if drag_right && offset <= 0.5 => {
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (drag_right, offset, None, None) 
                if drag_right && offset > 0.5 => {
                output.insert(
                    ((step + 1) % NUM_STEPS, track),
                    GridEvent {
                        offset: offset - 1.,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            // we are dragging to the right
            // something on the step to be dragged on, nothing on the next one
            (drag_right, offset, Some(found_event), None)
                if drag_right && offset > found_event.offset && offset >= OFFSET_THRESHOLD =>
            {
                output.insert(
                    ((step + 1) % NUM_STEPS, track),
                    GridEvent {
                        offset: offset - 1.,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (drag_right, offset, Some(found_event), None)
                if drag_right && (offset <= found_event.offset || offset < OFFSET_THRESHOLD) =>
            {
                output.remove(&(step, track));
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            // we are dragging to the right
            // nothing on the step to be dragged on, something on the next one though
            (drag_right, offset, None, Some(found_event))
                if drag_right
                    && (found_event.offset >= 0.
                        || (found_event.offset < 0.
                            && offset < (1. + found_event.offset - OFFSET_THRESHOLD))) =>
            {
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (drag_right, offset, None, Some(found_event))
                if drag_right
                    && found_event.offset < 0.
                    && offset >= (1. + found_event.offset - OFFSET_THRESHOLD) =>
            {
                output.remove(&((step + 1) % NUM_STEPS, track));
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            // we are dragging to the left
            // nothing on the step to be dragged on, nothing on the next one either
            (drag_right, offset, None, None) 
                if !drag_right && offset > 0.5  => {
                output.insert(
                    ((step + 1) % NUM_STEPS, track),
                    GridEvent {
                        offset: offset - 1.,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (drag_right, offset, None, None) 
                if !drag_right && offset <= 0.5  => {
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            // we are dragging to the left
            // something on the step to be dragged on, nothing on the next one
            (drag_right, offset, Some(found_event), None)
                if !drag_right
                    && offset > (found_event.offset + OFFSET_THRESHOLD)
                    && offset > OFFSET_THRESHOLD =>
            {
                output.insert(
                    ((step + 1) % NUM_STEPS, track),
                    GridEvent {
                        offset: offset - 1.,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (drag_right, offset, Some(found_event), None)
                if !drag_right
                    && (offset <= (found_event.offset + OFFSET_THRESHOLD)
                        || offset <= OFFSET_THRESHOLD) =>
            {
                output.remove(&(step, track));
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            // we are dragging to the left
            // nothing on the step to be dragged on, something on the next one
            (drag_right, offset, None, Some(found_event))
                if !drag_right
                    && (found_event.offset >= 0.
                        || (found_event.offset < 0.
                            && offset < (1. + event.offset - OFFSET_THRESHOLD))) =>
            {
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (drag_right, offset, None, Some(found_event))
                if !drag_right
                    && found_event.offset < 0.
                    && offset >= (1. + event.offset - OFFSET_THRESHOLD) =>
            {
                output.remove(&((step + 1) % NUM_STEPS, track));
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            // we are dragging
            // something on the step to be dragged on, something on the next one also
            (_, offset, Some(found_event), Some(_))
                if offset <= found_event.offset - OFFSET_THRESHOLD =>
            {
                output.remove(&(step, track));
                output.insert(
                    (step, track),
                    GridEvent {
                        offset,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            (_, offset, Some(found_event), Some(_))
                if offset > found_event.offset - OFFSET_THRESHOLD =>
            {
                output.remove(&((step + 1) % NUM_STEPS, track));
                output.insert(
                    ((step + 1) % NUM_STEPS, track),
                    GridEvent {
                        offset: offset - 1.,
                        velocity: event.velocity,
                        selected: true,
                    },
                );
            }
            _ => {
                println!("case not covered {:?}", cases);
            }
        }
    }

    pub fn set_velocity(&mut self, ratio: f32) {
        self.data.iter_mut().for_each(|(_, event)| {
            if event.selected {
                if ratio >= 0. {
                    event.velocity = (ratio * (1. - event.velocity) + event.velocity)
                        .min(1.)
                        .max(0.);
                } else {
                    let rate = 1. - ratio.max(-1.) * -1.;
                    event.velocity = rate * event.velocity;
                }
            }
        });
    }
}

impl From<Pattern> for GridPattern {
    fn from(pattern: Pattern) -> Self {
        let mut grid = GridPattern::new();

        for (i, step) in pattern.iter().enumerate() {
            for (j, perc) in step.iter().enumerate() {
                if perc[0] > 0.0 {
                    grid.data.insert(
                        (i, (NUM_PERCS - 1) - j),
                        GridEvent {
                            velocity: perc[0],
                            offset: perc[1],
                            selected: false,
                        },
                    );
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
            pattern.data[step][(NUM_PERCS - 1) - track][0] = event.velocity;
            pattern.data[step][(NUM_PERCS - 1) - track][1] = event.offset;
        }

        pattern
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    UI,
    STATE,
    NONE
}

#[derive(Debug, Clone)]
pub enum GridMessage {
    Add((usize, usize, f32)), // empty selection, add event, select event => COMMITS STATE
    Delete((usize, usize)), // delete selection
    ToggleOne((usize, usize)), // just mutate selection
    SelectOne((usize, usize)), // empty selection, select new one
    SelectArea(Rectangle, Size), // empty selection, select new one
    SelectAll(),
    EmptySelection(),
    ToggleArea(Rectangle, Size),
    MoveSelection((f32, isize), bool),
    DeleteSelection(),
    SetVelocity(f32),
    TrackSelected(usize),
    CommitState(),
    DiscardState(),
}


pub fn manage_state_update(
    message: GridMessage, 
    state: &mut State, 
    live_pattern: &mut GridPattern,
    focused_track: &mut usize
) {
    let mut next_grid = state.clone_base_pattern();

    match message {
        GridMessage::EmptySelection() => {
            live_pattern.empty_selection();
            state.set_pattern(live_pattern.clone());
        },
        GridMessage::Add((step, track, offset)) => {
            next_grid.data.insert((step, track), GridEvent {
                offset,
                ..GridEvent::default()
            });
            live_pattern.data = next_grid.data;
            state.set_pattern(live_pattern.clone());
        },
        GridMessage::Delete(grid_id) => {
            next_grid.data.remove(&grid_id);
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::ToggleOne(grid_id) => {
            next_grid.toggle_select(grid_id);
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::SelectOne(grid_id) => {
            next_grid.select_one(grid_id);
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::SelectArea(selection, bounds) => {
            next_grid.select_area(selection, bounds);
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::SelectAll() => {
            next_grid.select_all();
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::ToggleArea(selection, bounds) => {
            next_grid.toggle_area(selection, bounds);
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::MoveSelection(next_movement, relative) => {
            state.set_movement(next_movement, relative);
            
            match state.get_movement() {
                Some(movement) => {
                    next_grid.move_selection(movement.0, movement.1);
                },
                None => {},
            }

            live_pattern.data = next_grid.data;
        },
        GridMessage::DeleteSelection() => {
            next_grid.remove_selection();
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::SetVelocity(ratio) => {
            next_grid.set_velocity(ratio);
            live_pattern.data = next_grid.data.clone();
            state.set_pattern(next_grid);
        },
        GridMessage::TrackSelected(track) => {
            *focused_track = NUM_PERCS - track - 1;
        },
        GridMessage::DiscardState() => {
            live_pattern.data = state.clone_base_pattern().data;
        },
        _ => {}
    }
}

