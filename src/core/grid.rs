use std::collections::HashMap;

use iced_native::{
  keyboard, Point, Rectangle, Size
};

use ganic_no_std::{NUM_PERCS, NUM_STEPS, pattern::Pattern};

pub const STEP_MARGIN_RIGHT: f32 = 3.0;
pub const TRACK_MARGIN_BOTTOM: f32 = 8.0;
pub const CONTAINER_PADDING: f32 = 12.0;
pub const DEFAULT_VELOCITY: f32 = 1.0;
pub const OFFSET_THRESHOLD: f32 = 0.15;

pub fn normalize_point(point: Point, bounds: Rectangle) -> Point {
    return Point {
        x: (point.x - bounds.x).min(bounds.width).max(0.0),
        y: (point.y - bounds.y).min(bounds.height).max(0.0),
    }
}

pub fn is_point_inside_draggable_area(point: Point, bounds: Rectangle) -> bool {
    let draggable_area = Rectangle {
        x: CONTAINER_PADDING,
        y: CONTAINER_PADDING,
        width: bounds.width - 2.0 * CONTAINER_PADDING,
        height: bounds.height - 2.0 * CONTAINER_PADDING
    };
    
    return draggable_area.contains(point)
}

pub fn get_step_dimensions(bounds: Rectangle) -> Size {
  return Size {
      width: (bounds.width - (2.0 * CONTAINER_PADDING)) / NUM_STEPS as f32,
      height: ((bounds.height - (2.0 * CONTAINER_PADDING)) / NUM_PERCS as f32) - TRACK_MARGIN_BOTTOM
  }    
}

pub fn get_event_absolute_position(step: usize, track: usize, offset: f32, bounds: Rectangle) -> Point {
  let step_size = get_step_dimensions(bounds);

  return Point {
      x: CONTAINER_PADDING + (offset * step_size.width) + step as f32 * step_size.width,
      y: CONTAINER_PADDING + track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM)
  }
}

pub fn get_hovered_step(cursor: Point, bounds: Rectangle, bounded: bool) -> Option<(usize, usize, f32)> {
    let step_size = get_step_dimensions(bounds);

    println!("get_hovered_step {:?}", step_size);
    
    if bounded {
        if is_point_inside_draggable_area(cursor, bounds) {
            let step = ((cursor.x - CONTAINER_PADDING) / step_size.width) as usize;
            let track = ((cursor.y - CONTAINER_PADDING) / (step_size.height + TRACK_MARGIN_BOTTOM)) as usize;
            let offset = (cursor.x - (CONTAINER_PADDING + step as f32 * step_size.width)) / step_size.width;

            Some((step, track, offset))
        } else {
            None
        }
    } else {
        let step = (((cursor.x - CONTAINER_PADDING) / step_size.width) as usize).max(0).min(NUM_STEPS);
        let track = (((cursor.y - CONTAINER_PADDING) / (step_size.height + TRACK_MARGIN_BOTTOM)) as usize).max(0).min(NUM_PERCS);
        let offset = ((cursor.x - (CONTAINER_PADDING + step as f32 * step_size.width)) / step_size.width).max(-0.99).min(0.99);

        Some((step, track, offset))
    }
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
                    x: CONTAINER_PADDING + (grid_event.offset * step_size.width) + (*step as f32 * step_size.width),
                    y: CONTAINER_PADDING + (*track as f32 * (step_size.height + TRACK_MARGIN_BOTTOM)),
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
                width: step_size.width - STEP_MARGIN_RIGHT,
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

    pub fn remove_selection(&mut self) {
        for ((step, track), event) in self.data.to_owned() {
            if event.selected {
                self.data.remove(&(step, track));
            }
        }
    }

    pub fn move_selection_quantized(&mut self) {
        
    }

    pub fn move_selection_unquantized(&mut self) {
        
    }
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

      // println!("{:?}", grid.data);


      for ((step, track), event) in grid.data {
          pattern.data[step][track][0] = event.velocity;
          pattern.data[step][track][1] = event.offset;
      }

      pattern
  }
}

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
    ClickRelease,
    KeyAction(keyboard::KeyCode)
}

#[derive(Debug, Clone, Copy)]
pub enum DrawMode {
    Pen,
    Cursor
}