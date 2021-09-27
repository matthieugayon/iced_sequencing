use super::{Idle, Transition, WidgetContext, WidgetState};
use crate::core::grid::{
    get_hovered_track, get_hovered_step, get_step_width, 
    GridMessage, GridPattern
};
use iced_native::{keyboard, mouse, Point, Rectangle};

#[derive(Debug)]
pub struct Logo {
    nested: Box<dyn WidgetState + Send>,
}

impl WidgetState for Logo {
    fn on_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let (next_transition, messages) =
            self.nested.on_click(bounds, cursor, base_pattern, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, messages)
    }

    fn on_double_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let (next_transition, messages) =
            self.nested
                .on_double_click(bounds, cursor, base_pattern, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, messages)
    }

    fn on_button_release(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let (next_transition, messages) =
            self.nested
                .on_button_release(bounds, cursor, base_pattern, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        context.mouse_interaction = mouse::Interaction::default();

        (Transition::DoNothing, messages)
    }

    fn on_cursor_moved(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let (next_transition, messages) =
            self.nested
                .on_cursor_moved(bounds, cursor, base_pattern, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, messages)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let transition: Transition = self.nested.on_modifier_change(modifiers, context).0;

        match transition {
            Transition::ChangeState(new_state) => {
                self.next(new_state);
                return (Transition::DoNothing, None);
            }
            Transition::ChangeParentState(new_state) => {
                return (Transition::ChangeState(new_state), None)
            }
            Transition::DoNothing => return (Transition::DoNothing, None),
        }
    }

    fn on_key_pressed(
        &mut self,
        key_code: keyboard::KeyCode,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let (next_transition, messages) = self.nested.on_key_pressed(key_code, context);

        if let Transition::ChangeState(new_state) = next_transition {
            self.next(new_state);
        }

        (Transition::DoNothing, messages)
    }

    fn next(&mut self, next_state: Box<dyn WidgetState + Send>) {
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
            nested: Box::new(Selecting::from_args(point)),
        }
    }
}

#[derive(Debug, Default)]
struct Waiting;

impl WidgetState for Waiting {
    fn on_double_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let mut grid_messages = vec![
            GridMessage::TrackSelected(get_hovered_track(cursor, bounds)),
            GridMessage::EmptySelection()
        ];

        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            // if yes remove event
            Some((grid_id, _grid_event)) => {
                grid_messages.push(GridMessage::Delete(*grid_id));
            },
            // otherwise add event
            None => {
                let step_width = get_step_width(bounds.size());
                let interactive_area = Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width - step_width,
                    height: bounds.height,
                };

                if interactive_area.contains(cursor) {
                    let (step, track, offset) = get_hovered_step(cursor, bounds, true);
                    grid_messages.push(GridMessage::Add((step, track, offset)));
                }
            }
        }

        (Transition::DoNothing, Some(grid_messages))
    }

    fn on_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let mut grid_messages = vec![
            GridMessage::TrackSelected(get_hovered_track(cursor, bounds)),
            GridMessage::EmptySelection()
        ];

        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            // if yes select event
            Some(((step, track), grid_event)) => {
                if !grid_event.selected {
                    grid_messages.push(GridMessage::SelectOne((*step, *track)));
                }

                (
                    Transition::ChangeState(Box::new(SetVelocity::from_args(cursor))),
                    Some(grid_messages),
                )
            }
            // otherwise change to area selection mode
            None => (
                Transition::ChangeState(Box::new(Selecting::from_args(cursor))),
                None,
            ),
        }
    }

    fn on_cursor_moved(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        match base_pattern.get_hovered(cursor, bounds) {
            Some((_, _)) => {
                context.mouse_interaction = mouse::Interaction::ResizingVertically;
            }
            None => {
                context.mouse_interaction = mouse::Interaction::default();
            }
        }

        (Transition::DoNothing, None)
    }

    fn on_key_pressed(
        &mut self,
        key_code: keyboard::KeyCode,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let grid_message = match key_code {
            keyboard::KeyCode::A => Some(vec![GridMessage::EmptySelection(), GridMessage::SelectAll()]),
            keyboard::KeyCode::Backspace => Some(vec![GridMessage::DeleteSelection()]),
            keyboard::KeyCode::Left => Some(vec![GridMessage::MoveSelection((-0.05, 0), true)]),
            keyboard::KeyCode::Up => Some(vec![GridMessage::MoveSelection((0., -1), true)]),
            keyboard::KeyCode::Right => Some(vec![GridMessage::MoveSelection((0.05, 0), true)]),
            keyboard::KeyCode::Down => Some(vec![GridMessage::MoveSelection((0., 1), true)]),
            _ => None,
        };

        (Transition::DoNothing, grid_message)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if !modifiers.logo() {
            context.mouse_interaction = mouse::Interaction::default();
            (
                Transition::ChangeParentState(Box::new(Idle::default())),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
    }
}

#[derive(Debug, Default)]
struct Selecting {
    origin: Point,
}

impl Selecting {
    fn from_args(point: Point) -> Self {
        Selecting { origin: point }
    }
}

impl WidgetState for Selecting {
    fn on_cursor_moved(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        _base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let selection = Rectangle {
            x: if cursor.x - self.origin.x < 0.0 {
                cursor.x - bounds.x
            } else {
                self.origin.x - bounds.x
            },
            y: if cursor.y - self.origin.y < 0.0 {
                cursor.y - bounds.y
            } else {
                self.origin.y - bounds.y
            },
            width: if cursor.x - self.origin.x < 0.0 {
                self.origin.x - cursor.x
            } else {
                cursor.x - self.origin.x
            },
            height: if cursor.y - self.origin.y < 0.0 {
                self.origin.y - cursor.y
            } else {
                cursor.y - self.origin.y
            },
        };

        // display selection Rectangle
        context.selection_rectangle = Some(selection);

        (
            Transition::DoNothing,
            Some(vec![GridMessage::SelectArea(selection, bounds.size())]),
        )
    }

    fn on_button_release(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        // erase selection Rectangle
        context.selection_rectangle = None;

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if !modifiers.logo() {
            (
                Transition::ChangeParentState(Box::new(Idle::selecting(self.origin))),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
    }
}

#[derive(Debug, Default)]
struct SetVelocity {
    origin: Point,
    increment_factor: f32
}

impl SetVelocity {
    fn from_args(point: Point) -> Self {
        // todo: make increment_factor customizable
        // increment_factor is set to 2.0 by default but
        // could be lowered if shift is pressed for example
        SetVelocity { origin: point, increment_factor: 2.0 }
    }
}

impl WidgetState for SetVelocity {
    fn on_cursor_moved(
        &mut self,
        _bounds: Rectangle,
        cursor: Point,
        _base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let ratio = ((self.origin.y - cursor.y) * self.increment_factor)
            .min(127.).max(-127.) / 127.;
        self.origin.y = cursor.y; // reset origin with current cursor position

        (
            Transition::DoNothing,
            Some(vec![GridMessage::SetVelocity(ratio)]),
        )
    }

    fn on_button_release(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        // erase selection Rectangle
        context.selection_rectangle = None;

        (Transition::ChangeState(Box::new(Waiting::default())), None)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if !modifiers.logo() {
            context.mouse_interaction = mouse::Interaction::default();
            (
                Transition::ChangeParentState(Box::new(Idle::default())),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
    }
}
