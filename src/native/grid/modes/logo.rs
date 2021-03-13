use iced_native::{Rectangle, Point, keyboard, mouse};
use ganic_no_std::{pattern::Pattern};
use crate::core::grid::{
    GridEvent,
    is_point_inside_clickable_area,
    get_hovered_step,
    pad_cursor,
    GridMessage,
    DEFAULT_VELOCITY
};
use super::{WidgetState, Transition, WidgetContext, Idle};

#[derive(Debug)]
pub struct Logo {
    nested: Box<dyn WidgetState>,
}

impl WidgetState for Logo {
    fn on_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let (next_transition, message) = self.nested.on_click(bounds, cursor, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, message)
    }

    fn on_double_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let (next_transition, message) = self.nested.on_double_click(bounds, cursor, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, message)
    }

    fn on_button_release(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let (next_transition, message) = self.nested.on_button_release(bounds, cursor, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, message)
    }

    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        let (next_transition, message) = self.nested.on_cursor_moved(bounds, cursor, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, message)
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
        let (next_transition, message) = self.nested.on_key_pressed(key_code, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, message)
    }

    fn next(&mut self, next_state: Box<dyn WidgetState>) {
        // println!("Idle: changing sub state {:?} => {:?}",
        //     self.nested,
        //     next_state
        // );

        self.nested = next_state;
    }
}

impl Default for Logo {
    fn default() -> Logo {
        Logo {
            nested: Box::new(Waiting),
        }
    }
}

impl Logo {
    pub fn selecting(point: Point) -> Logo {
        Logo {
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
            // otherwise add event
            None => {
                if is_point_inside_clickable_area(cursor, bounds) {
                    match get_hovered_step(cursor, bounds) {
                        Some((step, track, offset)) => {
                            context.base_pattern.data.insert((step, track), GridEvent {
                                offset,
                                velocity: DEFAULT_VELOCITY,
                                selected: true
                            });
                        }
                        None => {
    
                        }
                    }
                }
            }
            _ => {}
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

                (Transition::ChangeState(Box::new(SetVelocity::from_args(cursor))), None)
            }
            // otherwise add event
            None => {
                (Transition::ChangeState(Box::new(Selecting::from_args(cursor))), None)
            }
        }
    }

    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        match context.base_pattern.clone().get_hovered(cursor, bounds) {
            // if yes remove event
            Some((_, _)) => {
                context.mouse_interaction = mouse::Interaction::ResizingVertically;
            }
            // otherwise add event
            None => {
                context.mouse_interaction = mouse::Interaction::default();
            }
        }
        
        (Transition::DoNothing, None)
    }

    fn on_key_pressed(&mut self, key_code: keyboard::KeyCode, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        match key_code {
            keyboard::KeyCode::A => {
                context.base_pattern.select_all();
            }
            keyboard::KeyCode::Backspace => {
                context.base_pattern.remove_selection();
            }
            keyboard::KeyCode::Left => {
                context.base_pattern.move_selection(-0.05, 0);
            }
            keyboard::KeyCode::Up => {
                context.base_pattern.move_selection(0., -1);
            }
            keyboard::KeyCode::Right => {
                context.base_pattern.move_selection(0.05, 0);
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

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if !modifiers.logo {
            context.mouse_interaction = mouse::Interaction::default();
            (Transition::ChangeParentState(Box::new(Idle::default())), None)
        } else {
            (Transition::DoNothing, None)
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
    fn on_cursor_moved(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // modify base pattern
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
        if !modifiers.logo {
            (Transition::ChangeParentState(Box::new(Idle::selecting(self.origin))), None)
        } else {
            (Transition::DoNothing, None)
        }
    }
}

#[derive(Debug, Default)]
struct SetVelocity {
    origin: Point
}

impl SetVelocity {
    fn from_args(point: Point) -> Self {
        SetVelocity {
            origin: point
        }
    }
}

impl WidgetState for SetVelocity {
    fn on_cursor_moved(&mut self, _bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {  
        let ratio = (self.origin.y - cursor.y).min(127.).max(-127.) / 127.;

        context.output_pattern = context.base_pattern.clone();
        context.output_pattern.set_velocity(ratio);

        (Transition::DoNothing, Some(GridMessage::NewPattern(Pattern::from(context.output_pattern.clone()))))
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        context.base_pattern.data = context.output_pattern.data.clone();
        context.mouse_interaction = mouse::Interaction::default();

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        if !modifiers.logo {
            context.mouse_interaction = mouse::Interaction::default();
            (Transition::ChangeParentState(Box::new(Idle::default())), None)
        } else {
            (Transition::DoNothing, None)
        }
    }
}

