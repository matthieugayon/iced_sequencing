use iced_native::{Rectangle, Point, keyboard};
use crate::core::grid::{GridPattern, GridEvent, get_hovered_step, pad_cursor};
use super::{WidgetState, Transition, WidgetContext};

#[derive(Debug)]
pub struct Idle {
    nested: Box<dyn WidgetState>,
}

impl WidgetState for Idle {
    fn on_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        if let Transition::ChangeState(new_state) =
            self.nested.on_click(bounds, cursor, context)
        {
            self.next(new_state);
        }

        Transition::DoNothing
    }

    fn on_double_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        if let Transition::ChangeState(new_state) =
            self.nested.on_double_click(bounds, cursor, context)
        {
            self.next(new_state);
        }

        Transition::DoNothing
    }

    fn on_button_release(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        if let Transition::ChangeState(new_state) =
            self.nested.on_button_release(bounds, cursor, context)
        {
            self.next(new_state);
        }

        Transition::DoNothing
    }

    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        if let Transition::ChangeState(new_state) =
            self.nested.on_cursor_moved(bounds, cursor, context)
        {
            self.next(new_state);
        }

        Transition::DoNothing
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, context: &mut WidgetContext) -> Transition {
        if let Transition::ChangeState(new_state) =
            self.nested.on_modifier_change(modifiers, context)
        {
            self.next(new_state);
        }

        Transition::DoNothing
    }

    fn next(&mut self, next_state: Box<dyn WidgetState>) {
        // println!("Idle: changing sub state {:?} => {:?}",
        //     self.nested,
        //     next_state
        // );

        self.nested = next_state;
    }
}

impl Default for Idle {
    fn default() -> Idle {
        Idle {
            nested: Box::new(Waiting),
        }
    }
}

#[derive(Debug, Default)]
struct Waiting;

impl WidgetState for Waiting {
    // in Waiting context double click add or remove events
    // but doesn't change nested context
    // it's just a side effect
    fn on_double_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {

        // check if we hover an event on the grid
        match context.base_pattern.clone().get_hovered(cursor, bounds) {
            // if yes remove event
            Some((grid_id, _grid_event)) => {
                context.base_pattern.data.remove(&grid_id);
            }
            // otherwise add event
            None => {
                match get_hovered_step(cursor, bounds, true) {
                    Some((step, track, ..)) => {
                        context.base_pattern.data.insert((step, track), GridEvent::default());
                    }
                    None => {}
                }
            }
        }

        // replicate base pattern to drawing pattern
        context.output_pattern = context.base_pattern.clone();

        Transition::DoNothing
    }

    fn on_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        // check if we hover an event on the grid
        match context.base_pattern.clone().get_hovered(cursor, bounds) {
            // if yes remove event
            Some((grid_id, grid_event)) => {
                if !grid_event.selected {
                    context.base_pattern.select_one(grid_id);
                    // replicate base pattern to drawing pattern
                    context.output_pattern = context.base_pattern.clone();
                }

                Transition::ChangeState(Box::new(MovingSelectionQuantized::from_args(cursor, grid_id)))
            }
            // otherwise add event
            None => {
                Transition::ChangeState(Box::new(Selecting::from_args(cursor)))
            }
        }
    }
}

#[derive(Debug, Default)]
struct Selecting {
    origin: Point
}

impl Selecting {
    fn from_args(point: Point) -> Self {
        Selecting {
            origin: point,
        }
    }
}

impl WidgetState for Selecting {
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        // modify base pattern
        let selection = Rectangle {
            x: if cursor.x - self.origin.x < 0.0 { cursor.x } else { self.origin.x },
            y: if cursor.y - self.origin.y < 0.0 { cursor.y } else { self.origin.y },
            width: if cursor.x - self.origin.x < 0.0 { self.origin.x - cursor.x } else { cursor.x - self.origin.x },
            height: if cursor.y - self.origin.y < 0.0 { self.origin.y - cursor.y } else { cursor.y - self.origin.y }
        };
        context.base_pattern.select_area(selection, bounds);

        // replicate base pattern to drawing pattern
        context.output_pattern = context.base_pattern.clone();

        // display selection Rectangle 
        context.selection_rectangle = Some(selection);

        Transition::DoNothing
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> Transition {
        // erase selection Rectangle 
        context.selection_rectangle = None;

        Transition::ChangeState(Box::new(Waiting::default()))
    }
}

#[derive(Debug, Default)]
struct MovingSelectionQuantized {
    origin: Point,
    origin_grid_id: (usize, usize)
}

impl MovingSelectionQuantized {
    fn from_args(point: Point, grid_id: (usize, usize)) -> Self {
        MovingSelectionQuantized {
            origin: point,
            origin_grid_id: grid_id
        }
    }
}

impl WidgetState for MovingSelectionQuantized {
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {  
        // cursor cannot get out of the grid area (container padding excluded)
        let padded_cursor = pad_cursor(cursor, bounds);

        // set and mutate output pattern
        context.output_pattern = context.base_pattern.clone();
        context.output_pattern.move_selection_quantized(bounds, padded_cursor, self.origin_grid_id);

        Transition::DoNothing
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> Transition {
        if modifiers.logo {
            Transition::ChangeState(Box::new(MovingSelectionUnquantized::from_args(self.origin, self.origin_grid_id)))
        } else {
            Transition::DoNothing
        }
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> Transition {
        // commit ouput pattern changes to base_patern
        context.base_pattern.data = context.output_pattern.data.clone();

        Transition::ChangeState(Box::new(Waiting::default()))
    }
}

#[derive(Debug, Default)]
struct MovingSelectionUnquantized {
    origin: Point,
    origin_grid_id: (usize, usize)
}

impl MovingSelectionUnquantized {
    fn from_args(point: Point, grid_id: (usize, usize)) -> Self {
        MovingSelectionUnquantized {
            origin: point,
            origin_grid_id: grid_id
        }
    }
}

impl WidgetState for MovingSelectionUnquantized {
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> Transition {
        // cursor cannot get out of the grid area (container padding excluded)
        let padded_cursor = pad_cursor(cursor, bounds);
        let drag_bounds = Rectangle {
            x: self.origin.x,
            y: self.origin.y,
            width: padded_cursor.x - self.origin.x,
            height: padded_cursor.y - self.origin.y
        };

        // set and mutate output pattern
        context.output_pattern = context.base_pattern.clone();
        context.output_pattern.move_selection_unquantized(bounds, drag_bounds, padded_cursor, self.origin_grid_id);

        Transition::DoNothing
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> Transition {
        if !modifiers.logo {
            Transition::ChangeState(Box::new(MovingSelectionQuantized::from_args(self.origin, self.origin_grid_id)))
        } else {
            Transition::DoNothing
        }
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> Transition {
        // commit ouput pattern changes to base_patern
        context.base_pattern.data = context.output_pattern.data.clone();

        Transition::ChangeState(Box::new(Waiting::default()))
    }
}
