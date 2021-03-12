use iced_native::{Rectangle, Point, keyboard, mouse};
use ganic_no_std::pattern::Pattern;
use crate::core::grid::{
    GridEvent,
    is_point_inside_clickable_area,
    get_hovered_step,
    pad_cursor,
    GridMessage
};
use super::{WidgetState, Transition, WidgetContext};
use super::Logo;
use super::Shift;

#[derive(Debug)]
pub struct Idle {
    nested: Box<dyn WidgetState>,
}

impl WidgetState for Idle {
    fn on_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if let Transition::ChangeState(new_state) =
            self.nested.on_click(bounds, cursor, context).0
        {
            self.next(new_state);
        }

        (Transition::DoNothing, None)
    }

    fn on_double_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if let Transition::ChangeState(new_state) =
            self.nested.on_double_click(bounds, cursor, context).0
        {
            self.next(new_state);
        }

        (Transition::DoNothing, None)
    }

    fn on_button_release(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if let Transition::ChangeState(new_state) =
            self.nested.on_button_release(bounds, cursor, context).0
        {
            self.next(new_state);
        }

        (Transition::DoNothing, None)
    }

    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if let Transition::ChangeState(new_state) =
            self.nested.on_cursor_moved(bounds, cursor, context).0
        {
            self.next(new_state);
        }

        (Transition::DoNothing, None)
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let transition: Transition = self.nested.on_modifier_change(modifiers, context).0;

        match transition {
            Transition::ChangeState(new_state) => {
                self.next(new_state);
                return (Transition::DoNothing, None)
            }
            Transition::ChangeParentState(new_state) => {
                return (Transition::ChangeState(new_state), None)
            }
            Transition::DoNothing => {
                return (Transition::DoNothing, None)
            }
        }
    }

    fn on_key_pressed(&mut self, key_code: keyboard::KeyCode, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if let Transition::ChangeState(new_state) =
            self.nested.on_key_pressed(key_code, context).0
        {
            self.next(new_state);
        }

        (Transition::DoNothing, None)
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

impl Idle {
    pub fn selecting(point: Point) -> Idle {
        Idle {
            nested: Box::new(Selecting::from_args(point))
        }
    }
}

#[derive(Debug, Default)]
struct Waiting;

impl WidgetState for Waiting {
    // in Waiting context double click add or remove events
    // but doesn't change nested context
    // it's just a side effect
    fn on_double_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let padded_cursor = pad_cursor(cursor, bounds);
        // check if we hover an event on the grid
        match context.base_pattern.clone().get_hovered(padded_cursor, bounds) {
            // if yes remove event
            Some((grid_id, _grid_event)) => {
                context.base_pattern.data.remove(&grid_id);
            }
            // otherwise add event
            None => {
                if is_point_inside_clickable_area(cursor, bounds) {
                    match get_hovered_step(cursor, bounds) {
                        Some((step, track, ..)) => {
                            context.base_pattern.data.insert((step, track), GridEvent::default());
                        }
                        None => {}
                    }
                }
            }
        }

        // replicate base pattern to drawing pattern
        context.output_pattern = context.base_pattern.clone();

        (Transition::DoNothing, Some(GridMessage::NewPattern(Pattern::from(context.output_pattern.clone()))))
    }

    fn on_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // check if we hover an event on the grid
        match context.base_pattern.clone().get_hovered(cursor, bounds) {
            // if yes remove event
            Some(((step, track), grid_event)) => {
                if !grid_event.selected {
                    context.base_pattern.select_one((step, track));
                    // replicate base pattern to drawing pattern
                    context.output_pattern = context.base_pattern.clone();
                }

                context.mouse_interaction = mouse::Interaction::Grab;

                (Transition::ChangeState(Box::new(MovingSelectionQuantized::from_args(cursor, (step, track, grid_event)))), None)
            }
            // otherwise add event
            None => {
                (Transition::ChangeState(Box::new(Selecting::from_args(cursor))), None)
            }
        }
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        match modifiers {
            keyboard::Modifiers { logo: true, .. } => {
                (Transition::ChangeParentState(Box::new(Logo::default())), None)
            },
            keyboard::Modifiers { logo: false, shift: true, .. } => {
                (Transition::ChangeParentState(Box::new(Shift::default())), None)
            }
            _ => { (Transition::DoNothing, None) }
        }
    }

    fn on_key_pressed(&mut self, key_code: keyboard::KeyCode, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        match key_code {
            keyboard::KeyCode::Backspace => {
                context.base_pattern.remove_selection();
            }
            keyboard::KeyCode::Left => {
                context.base_pattern.move_selection(-1., 0);
            }
            keyboard::KeyCode::Up => {
                context.base_pattern.move_selection(0., -1);
            }
            keyboard::KeyCode::Right => {
                context.base_pattern.move_selection(1., 0);
            }
            keyboard::KeyCode::Down => {
                context.base_pattern.move_selection(0., 1);
            }
            _ => {}
        }

        // replicate base pattern to drawing pattern
        context.output_pattern = context.base_pattern.clone();

        (Transition::DoNothing, None)
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
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let padded_cursor = pad_cursor(cursor, bounds);

        let selection = Rectangle {
            x: if padded_cursor.x - self.origin.x < 0.0 { padded_cursor.x } else { self.origin.x },
            y: if padded_cursor.y - self.origin.y < 0.0 { padded_cursor.y } else { self.origin.y },
            width: if padded_cursor.x - self.origin.x < 0.0 { self.origin.x - padded_cursor.x } else { padded_cursor.x - self.origin.x },
            height: if padded_cursor.y - self.origin.y < 0.0 { self.origin.y - padded_cursor.y } else { padded_cursor.y - self.origin.y }
        };
        context.base_pattern.select_area(selection, bounds);

        // replicate base pattern to drawing pattern
        context.output_pattern = context.base_pattern.clone();

        // display selection Rectangle 
        context.selection_rectangle = Some(selection);

        (Transition::DoNothing, None)
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // erase selection Rectangle 
        context.selection_rectangle = None;

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if modifiers.logo {
            (Transition::ChangeState(Box::new(Logo::default())), None)
        } else {
            (Transition::DoNothing, None)
        }
    }
}

#[derive(Debug, Default)]
struct MovingSelectionQuantized {
    origin: Point,
    origin_event: (usize, usize, GridEvent)
}

impl MovingSelectionQuantized {
    fn from_args(point: Point, event: (usize, usize, GridEvent)) -> Self {
        MovingSelectionQuantized {
            origin: point,
            origin_event: event
        }
    }
}

impl WidgetState for MovingSelectionQuantized {
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {  
        let drag_bounds = Rectangle {
            x: self.origin.x,
            y: self.origin.y,
            width: cursor.x - self.origin.x,
            height: cursor.y - self.origin.y
        };

        // set and mutate output pattern
        context.output_pattern = context.base_pattern.clone();
        context.output_pattern.move_selection_quantized(bounds, drag_bounds, cursor, self.origin_event);

        (Transition::DoNothing, Some(GridMessage::NewPattern(Pattern::from(context.output_pattern.clone()))))
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if modifiers.logo {
            (Transition::ChangeState(Box::new(MovingSelectionUnquantized::from_args(self.origin, self.origin_event))), None)
        } else {
            (Transition::DoNothing, None)
        }
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // commit ouput pattern changes to base_patern
        context.output_pattern.clean_negative_offsets();
        context.base_pattern.data = context.output_pattern.data.clone();

        context.mouse_interaction = mouse::Interaction::default();

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }
}

#[derive(Debug, Default)]
struct MovingSelectionUnquantized {
    origin: Point,
    origin_event: (usize, usize, GridEvent)
}

impl MovingSelectionUnquantized {
    fn from_args(point: Point, event: (usize, usize, GridEvent)) -> Self {
        MovingSelectionUnquantized {
            origin: point,
            origin_event: event
        }
    }
}

impl WidgetState for MovingSelectionUnquantized {
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // cursor cannot get out of the grid area (container padding excluded)
        // let padded_cursor = pad_cursor(cursor, bounds);
        let drag_bounds = Rectangle {
            x: self.origin.x,
            y: self.origin.y,
            width: cursor.x - self.origin.x,
            height: cursor.y - self.origin.y
        };

        // set and mutate output pattern
        context.output_pattern = context.base_pattern.clone();
        context.output_pattern.move_selection_unquantized(bounds, drag_bounds, cursor, self.origin_event);

        (Transition::DoNothing, Some(GridMessage::NewPattern(Pattern::from(context.output_pattern.clone()))))
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if !modifiers.logo {
            (Transition::ChangeState(Box::new(MovingSelectionQuantized::from_args(self.origin, self.origin_event))), None)
        } else {
            (Transition::DoNothing, None)
        }
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // commit ouput pattern changes to base_patern
        context.output_pattern.clean_negative_offsets();
        context.base_pattern.data = context.output_pattern.data.clone();

        context.mouse_interaction = mouse::Interaction::default();

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }
}
