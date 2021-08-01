use std::collections::HashMap;
use itertools::Itertools;

use iced_native::{
  Point, Rectangle, Size
};

use ganic_no_std::{NUM_PERCS, NUM_STEPS, pattern::Pattern};

pub const TRACK_MARGIN_BOTTOM: f32 = 3.0;
pub const CONTAINER_PADDING_TOP: f32 = 0.;
pub const CONTAINER_PADDING_LEFT: f32 = 0.;

pub const DEFAULT_VELOCITY: f32 = 1.0;
pub const OFFSET_THRESHOLD: f32 = 0.05;

pub fn normalize_point(point: Point, bounds: Rectangle) -> Point {
    Point {
        x: (point.x - bounds.x).ceil(),
        y: (point.y - bounds.y).ceil(),
    }
}

pub fn is_point_inside_clickable_area(point: Point, bounds: Rectangle) -> bool {
    let step_size = get_step_dimensions(bounds);

    let clickable_area = Rectangle {
        x: CONTAINER_PADDING_LEFT,
        y: CONTAINER_PADDING_TOP,
        width: bounds.width - CONTAINER_PADDING_LEFT - step_size.width,
        height: bounds.height - CONTAINER_PADDING_TOP - TRACK_MARGIN_BOTTOM
    };
    
    clickable_area.contains(point)
}

pub fn is_point_inside_draggable_area(point: Point, bounds: Rectangle) -> bool {
    let draggable_area = Rectangle {
        x: CONTAINER_PADDING_LEFT,
        y: CONTAINER_PADDING_TOP,
        width: bounds.width - CONTAINER_PADDING_LEFT,
        height: bounds.height - CONTAINER_PADDING_TOP
    };
    
    draggable_area.contains(point)
}

// cursor and bounds are normalized normalized
pub fn pad_cursor(point: Point, bounds: Rectangle) -> Point {
    return Point {
        x: point.x.min(bounds.width).max(CONTAINER_PADDING_LEFT),
        y: point.y.min(bounds.height - TRACK_MARGIN_BOTTOM).max(CONTAINER_PADDING_TOP),
    }
}

pub fn get_step_dimensions(bounds: Rectangle) -> Size {
  return Size {
      width: (bounds.width - CONTAINER_PADDING_LEFT) / (NUM_STEPS + 1) as f32,
      height: ((bounds.height - CONTAINER_PADDING_TOP) / NUM_PERCS as f32) - TRACK_MARGIN_BOTTOM
  }    
}

pub fn get_event_absolute_position(step: usize, track: usize, offset: f32, bounds: Rectangle) -> Point {
  let step_size = get_step_dimensions(bounds);

  return Point {
      x: (CONTAINER_PADDING_LEFT + (offset * step_size.width) + step as f32 * step_size.width).ceil(),
      y: (CONTAINER_PADDING_TOP + track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM)).ceil()
  }
}

pub fn get_hovered_step(cursor: Point, bounds: Rectangle) -> Option<(usize, usize, f32)> {
    let step_size = get_step_dimensions(bounds);
    
    let step = (((cursor.x - CONTAINER_PADDING_LEFT) / step_size.width) as usize).max(0).min(NUM_STEPS - 1);
    let track = (((cursor.y - CONTAINER_PADDING_TOP) / (step_size.height + TRACK_MARGIN_BOTTOM)) as usize).max(0).min(NUM_PERCS - 1);
    let offset = ((cursor.x - (CONTAINER_PADDING_LEFT + step as f32 * step_size.width)) / step_size.width).max(-0.99).min(0.99);

    Some((step, track, offset))
}

#[derive(Debug, Clone, Copy)]
pub struct GridEvent {
    pub offset: f32,
    pub velocity: f32,
    pub selected: bool
}

impl Default for GridEvent {
    fn default() -> Self {
        GridEvent {
            offset: 0.0,
            velocity: DEFAULT_VELOCITY,
            selected: true
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridPattern {
    pub data: HashMap<(usize, usize), GridEvent>
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
                    x: CONTAINER_PADDING_LEFT + (grid_event.offset * step_size.width) + (*step as f32 * step_size.width),
                    y: CONTAINER_PADDING_TOP + (*track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM)),
                    width: step_size.width,
                    height: step_size.height
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

    pub fn select(&mut self, grid_id: (usize, usize)) {
        // add event
        match self.data.get_mut(&grid_id) {
            Some(grid_event) => {
                grid_event.selected = true;
            }
            None => {}
        }
    }

    pub fn select_all(&mut self) {
        // add event
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

    pub fn select_area(&mut self, selection: Rectangle, bounds: Rectangle) {
        let step_size = get_step_dimensions(bounds);
        self.data.iter_mut().for_each(|((step, track), grid_event)| {
            let event_origin = get_event_absolute_position(*step, *track, grid_event.offset, bounds);

            let event_bounds = Rectangle {
                x: event_origin.x,
                y: event_origin.y,
                width: step_size.width,
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

    pub fn toggle_area(&mut self, selection: Rectangle, bounds: Rectangle) {
        let step_size = get_step_dimensions(bounds);
        self.data.iter_mut().for_each(|((step, track), grid_event)| {
            let event_origin = get_event_absolute_position(*step, *track, grid_event.offset, bounds);

            let event_bounds = Rectangle {
                x: event_origin.x,
                y: event_origin.y,
                width: step_size.width,
                height: step_size.height,
            };

            match selection.intersection(&event_bounds) {
                Some(_) => {
                    grid_event.selected = !grid_event.selected;
                }
                _ => {}
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

    pub fn remove_selection(&mut self) {
        for ((step, track), event) in self.data.to_owned() {
            if event.selected {
                self.data.remove(&(step, track));
            }
        }
    }

    pub fn move_selection_quantized(
        &mut self, 
        bounds: Rectangle,
        drag_bounds: Rectangle,
        cursor: Point,
        origin_event: (usize, usize, GridEvent)
    ) -> bool {
        // cursor is normalized and padded, it cannto be outside
        // so it must be hovering a step
        // let step_offset: isize = hovered_step.0 as isize - origin_grid_id.0 as isize;
        let step_size = get_step_dimensions(bounds);
        let hovered_step = get_hovered_step(cursor, bounds).unwrap();
        let track_offset: isize = hovered_step.1 as isize - origin_event.1 as isize;

        let max_positive_offset: f32 = (NUM_STEPS - origin_event.0) as f32 - 1.;
        let min_negative_offset: f32 =  -1. * origin_event.0 as f32 - origin_event.2.offset;

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
                    selection_step_offset = (step_offset as f32 * 0.5).floor() + (-1. * origin_event.2.offset);
                } else {
                    selection_step_offset = (step_offset as f32 * 0.5).floor() + (1. - origin_event.2.offset);
                }
            }
        }

        self.move_selection(selection_step_offset.min(max_positive_offset).max(min_negative_offset), track_offset);

        track_offset != 0 || selection_step_offset != 0.
    }

    pub fn move_selection_unquantized(
        &mut self,
        bounds: Rectangle,
        drag_bounds: Rectangle,
        cursor: Point,
        origin_event: (usize, usize, GridEvent)
    ) {
        let max_positive_offset: f32 = (NUM_STEPS - origin_event.0) as f32 - origin_event.2.offset - OFFSET_THRESHOLD;
        let min_negative_offset: f32 =  -1. * origin_event.0 as f32 - origin_event.2.offset;
        let step_size = get_step_dimensions(bounds);
        let step_offset = (drag_bounds.width / step_size.width).min(max_positive_offset).max(min_negative_offset);
        let hovered_step = get_hovered_step(cursor, bounds).unwrap();
        let track_offset: isize = hovered_step.1 as isize - origin_event.1 as isize;

        self.move_selection(step_offset, track_offset);
    }

    pub fn move_selection(
        &mut self,
        step_offset: f32,
        track_offset: isize
    ) {
        // init empty hashmap
        let mut output: HashMap<(usize, usize), GridEvent>  = HashMap::new();

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
                let next_step_offset = (step as f32 + event.offset + step_offset + NUM_STEPS as f32) % NUM_STEPS as f32;
                let next_step = next_step_offset.floor() as usize;
                let next_offset = next_step_offset - next_step as f32;

                // next track
                let next_track = (track as isize + track_offset + NUM_PERCS as isize) as usize % NUM_PERCS;

                // check events at next locations
                let next_event = output_map.get(&(next_step, next_track));
                let next_event_plus_one = output_map.get(&((next_step + 1) % NUM_STEPS, next_track));

                // build a tuple with all that data, then we pattern match on it
                let cases: (bool, f32, Option<&GridEvent>, Option<&GridEvent>) = (
                    step_offset >= 0.,
                    next_offset,
                    next_event,
                    next_event_plus_one
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
        event: GridEvent) {
            
        match cases {
            // we are dragging to the right
            // nothing on the step to be dragged on, nothing on the next one either
            (drag_right, offset, None, None) if drag_right => {
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            // we are dragging to the right
            // something on the step to be dragged on, nothing on the next one
            (drag_right, offset, Some(found_event), None) if drag_right && offset > found_event.offset && offset >= OFFSET_THRESHOLD => {
                output.insert(((step + 1) % NUM_STEPS, track), GridEvent {
                    offset: offset - 1.,
                    velocity: event.velocity,
                    selected: true
                });
            }
            (drag_right, offset, Some(found_event), None) if drag_right && (offset <= found_event.offset || offset < OFFSET_THRESHOLD) => {
                output.remove(&(step, track));
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            // we are dragging to the right
            // nothing on the step to be dragged on, something on the next one though
            (drag_right, offset, None, Some(found_event)) 
                if drag_right && (found_event.offset >= 0. || (found_event.offset < 0. && offset < (1. + found_event.offset - OFFSET_THRESHOLD))) => {

                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            (drag_right, offset, None, Some(found_event)) 
                if drag_right && found_event.offset < 0. && offset >= (1. + found_event.offset - OFFSET_THRESHOLD) => {

                output.remove(&((step + 1) % NUM_STEPS, track));
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            // we are dragging to the left
            // nothing on the step to be dragged on, nothing on the next one either
            (drag_right, offset, None, None) if !drag_right => {
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            // we are dragging to the left
            // something on the step to be dragged on, nothing on the next one
            (drag_right, offset, Some(found_event), None) 
                if !drag_right && offset > (found_event.offset + OFFSET_THRESHOLD) && offset > OFFSET_THRESHOLD => {

                output.insert(((step + 1) % NUM_STEPS, track), GridEvent {
                    offset: offset - 1.,
                    velocity: event.velocity,
                    selected: true
                });
            }
            (drag_right, offset, Some(found_event), None) 
                if !drag_right && (offset <= (found_event.offset + OFFSET_THRESHOLD) || offset <= OFFSET_THRESHOLD) => {

                output.remove(&(step, track));
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            // we are dragging to the left
            // nothing on the step to be dragged on, something on the next one
            (drag_right, offset, None, Some(found_event)) 
                if !drag_right && (found_event.offset >= 0. || (found_event.offset < 0. && offset < (1. + event.offset - OFFSET_THRESHOLD))) => {

                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            (drag_right, offset, None, Some(found_event)) 
                if !drag_right && found_event.offset < 0. && offset >= (1. + event.offset - OFFSET_THRESHOLD) => {

                output.remove(&((step + 1) % NUM_STEPS, track));
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            // we are dragging
            // something on the step to be dragged on, something on the next one also
            (_, offset, Some(found_event), Some(_)) if offset <= found_event.offset - OFFSET_THRESHOLD => {
                output.remove(&(step, track));
                output.insert((step, track), GridEvent {
                    offset,
                    velocity: event.velocity,
                    selected: true
                });
            }
            (_, offset, Some(found_event), Some(_)) if offset > found_event.offset - OFFSET_THRESHOLD => {
                output.remove(&((step + 1) % NUM_STEPS, track));
                output.insert(((step + 1) % NUM_STEPS, track), GridEvent {
                    offset: offset - 1.,
                    velocity: event.velocity,
                    selected: true
                });
            }
            _ => {
                println!("case not covered {:?}", cases);
            }
        }
    }

    // clean negative offset which may stay on unselected events, when moving evenst around
    // since basic iterating on Hashmap is not ordered, we need to sort the iterator 
    // because only when cycling through in an orderly manner (per track / event in ascending order)
    // are we sure to treat every event correctly
    pub fn clean_negative_offsets(&mut self) {
        for ((step, track), event) in self.data.to_owned().into_iter()
            .sorted_by(|x, y| {
                if x.0.1 == y.0.1 {
                    return x.0.0.cmp(&y.0.0)
                }
                x.0.1.cmp(&y.0.1)
            }) {

            if event.offset < 0. {
                // let previous_step = (step - 1 + NUM_STEPS) % NUM_STEPS;
                let previous_step = step.checked_sub(1).unwrap_or(NUM_STEPS - 1);
                let previous_event = self.data.get(&(previous_step, track));

                match previous_event {
                    None => {
                        self.data.remove(&(step, track));
                        self.data.insert((previous_step, track), GridEvent {
                            offset: event.offset + 1.,
                            velocity: event.velocity,
                            selected: false
                        });
                    }
                    Some(_) => {}
                }
            }
        }
    }

    pub fn set_velocity(&mut self, ratio: f32) {
        self.data.iter_mut().for_each(|(_, event)| {
            if event.selected {
                if ratio >= 0. {
                    event.velocity = (ratio * (1. - event.velocity) + event.velocity).min(1.).max(0.);
                } else {
                    let rate = 1.- ratio.max(-1.) * -1.;
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
                  grid.data.insert((i, (NUM_PERCS - 1) - j), GridEvent { velocity: perc[0], offset: perc[1], selected: false });
              }
          }
      }

      grid
  }
}

impl From<GridPattern> for Pattern {
  fn from(grid: GridPattern) -> Self {
      let mut pattern = Pattern::new();

      // println!("{:?}", grid.data);


      for ((step, track), event) in grid.data {
          pattern.data[step][(NUM_PERCS - 1) - track][0] = event.velocity;
          pattern.data[step][(NUM_PERCS - 1) - track][1] = event.offset;
      }

      pattern
  }
}

#[derive(Debug)]
pub enum GridMessage {
    NewPattern(Pattern),
    TrackSelected(usize)
}
