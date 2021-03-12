use iced_native::{Rectangle, Point, keyboard};
use ganic_no_std::pattern::Pattern;
use crate::core::grid::{
    get_hovered_step,
    is_point_inside_clickable_area,
    pad_cursor,
    GridMessage,
    GridEvent
};
use super::{WidgetState, Transition, WidgetContext};
use super::Logo;
use super::Idle;

#[derive(Debug)]
pub struct Shift {
    nested: Box<dyn WidgetState>,
}

impl WidgetState for Shift {
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
        // println!("Shift: changing sub state {:?} => {:?}",
        //     self.nested,
        //     next_state
        // );

        self.nested = next_state;
    }


}

impl Default for Shift {
    fn default() -> Shift {
        Shift {
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
    fn on_double_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // check if we hover an event on the grid
        match context.base_pattern.clone().get_hovered(cursor, bounds) {
            // if yes remove event
            Some(((step, track), _)) => {
                context.base_pattern.toggle_select((step, track));
                context.output_pattern = context.base_pattern.clone();

                (Transition::DoNothing, None)
            }
            // otherwise add event
            None => {
                if is_point_inside_clickable_area(cursor, bounds) {
                    match get_hovered_step(cursor, bounds) {
                        Some((step, track, ..)) => {
                            context.base_pattern.data.insert((step, track), GridEvent::default());
                            context.output_pattern = context.base_pattern.clone();
                            return (Transition::DoNothing, Some(GridMessage::NewPattern(Pattern::from(context.output_pattern.clone()))))
                        }
                        None => {
                            return (Transition::DoNothing, None)
                        }
                    }
                } else {
                    return (Transition::DoNothing, None)
                }
            }
        }
    }

    fn on_click(&mut self, bounds: Rectangle, cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // check if we hover an event on the grid
        match context.base_pattern.clone().get_hovered(cursor, bounds) {
            // if yes remove event
            Some(((step, track), _)) => {
                context.base_pattern.toggle_select((step, track));
                context.output_pattern = context.base_pattern.clone();

                (Transition::DoNothing, None)
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
            keyboard::Modifiers { logo: false, shift: false, .. } => {
                (Transition::ChangeParentState(Box::new(Idle::default())), None)
            }
            _ => { (Transition::DoNothing, None) }
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
        let padded_cursor = pad_cursor(cursor, bounds);

        let selection = Rectangle {
            x: if padded_cursor.x - self.origin.x < 0.0 { padded_cursor.x } else { self.origin.x },
            y: if padded_cursor.y - self.origin.y < 0.0 { padded_cursor.y } else { self.origin.y },
            width: if padded_cursor.x - self.origin.x < 0.0 { self.origin.x - padded_cursor.x } else { padded_cursor.x - self.origin.x },
            height: if padded_cursor.y - self.origin.y < 0.0 { self.origin.y - padded_cursor.y } else { padded_cursor.y - self.origin.y }
        };

        context.output_pattern = context.base_pattern.clone();
        context.output_pattern.toggle_area(selection, bounds);

        // display selection Rectangle 
        context.selection_rectangle = Some(selection);

        (Transition::DoNothing, None)
    }

    fn on_button_release(&mut self, _bounds: Rectangle, _cursor: Point, context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        // erase selection Rectangle 
        context.selection_rectangle = None;

        // commit ouput pattern changes to base_patern
        context.base_pattern.data = context.output_pattern.data.clone();

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }

    fn on_modifier_change(&mut self, modifiers: keyboard::Modifiers, _context: &mut WidgetContext) -> (Transition, Option<GridMessage>) {
        match modifiers {
            keyboard::Modifiers { logo: true, .. } => {
                (Transition::ChangeParentState(Box::new(Logo::selecting(self.origin))), None)
            },
            keyboard::Modifiers { logo: false, shift: false, .. } => {
                (Transition::ChangeParentState(Box::new(Idle::selecting(self.origin))), None)
            }
            _ => { (Transition::DoNothing, None) }
        }
    }
}
