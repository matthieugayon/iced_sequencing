use super::LogoCtrl;
use super::Shift;
use super::{Transition, WidgetContext, WidgetState};
use crate::core::grid::{
    get_hovered_step, get_hovered_track, get_step_width, 
    GridEvent, GridMessage, GridPattern
};
use iced_native::{keyboard, mouse, Point, Rectangle};

#[derive(Debug)]
pub struct Idle {
    nested: Box<dyn WidgetState + Send>,
}

impl WidgetState for Idle {
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
            nested: Box::new(Selecting::from_args(point)),
        }
    }
}

#[derive(Debug, Default)]
struct Waiting;

impl WidgetState for Waiting {
    // in Waiting context double click add or remove events
    // but doesn't change nested context
    // it's just a side effect
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
                    x: bounds.x + step_width,
                    y: bounds.y,
                    width: bounds.width - 2. * step_width,
                    height: bounds.height,
                };

                if interactive_area.contains(cursor) {
                    let (step, track, _) = get_hovered_step(cursor, bounds, true);
                    grid_messages.push(GridMessage::Add((step, track, 0.)));
                }
            }
        }

        (
            Transition::DoNothing,
            Some(grid_messages),
        )
    }

    fn on_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let mut grid_messages = vec![
            GridMessage::TrackSelected(get_hovered_track(cursor, bounds)),
        ];

        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            // if yes remove event
            Some(((step, track), grid_event)) => {
                if !grid_event.selected {
                    println!("event not selected");
                    grid_messages.push(GridMessage::EmptySelection());
                    grid_messages.push(GridMessage::SelectOne((*step, *track)))
                }

                context.mouse_interaction = mouse::Interaction::Grab;

                (
                    Transition::ChangeState(Box::new(MovingSelectionQuantized::from_args(
                        cursor,
                        (*step, *track, *grid_event),
                    ))),
                    Some(grid_messages),
                )
            }
            // otherwise add event
            None => {
                grid_messages.push(GridMessage::EmptySelection());

                (
                    Transition::ChangeState(Box::new(Selecting::from_args(cursor))),
                    Some(grid_messages),
                )
            }
        }
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if modifiers.logo() || modifiers.control() {
            (
                Transition::ChangeParentState(Box::new(LogoCtrl::default())),
                None,
            )
        } else if modifiers.shift() {
            (
                Transition::ChangeParentState(Box::new(Shift::default())),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
    }

    fn on_key_pressed(
        &mut self,
        key_code: keyboard::KeyCode,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let grid_message = match key_code {
            keyboard::KeyCode::A => Some(vec![GridMessage::EmptySelection(), GridMessage::SelectAll()]),
            keyboard::KeyCode::Backspace => Some(vec![GridMessage::DeleteSelection()]),
            keyboard::KeyCode::Left => Some(vec![GridMessage::MoveSelection((-1., 0), true)]),
            keyboard::KeyCode::Up => Some(vec![GridMessage::MoveSelection((0., -1), true)]),
            keyboard::KeyCode::Right => Some(vec![GridMessage::MoveSelection((1., 0), true)]),
            keyboard::KeyCode::Down => Some(vec![GridMessage::MoveSelection((0., 1), true)]),
            _ => None,
        };

        (Transition::DoNothing, grid_message)
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
}

#[derive(Debug, Default)]
struct MovingSelectionQuantized {
    origin: Point,
    origin_event: (usize, usize, GridEvent),
}

impl MovingSelectionQuantized {
    fn from_args(point: Point, event: (usize, usize, GridEvent)) -> Self {
        MovingSelectionQuantized {
            origin: point,
            origin_event: event,
        }
    }
}

impl WidgetState for MovingSelectionQuantized {
    fn on_cursor_moved(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let drag_bounds = Rectangle {
            x: self.origin.x,
            y: self.origin.y,
            width: cursor.x - self.origin.x,
            height: cursor.y - self.origin.y,
        };

        // debounce a bit
        let movement =
            base_pattern.move_selection_quantized(bounds, drag_bounds, cursor, self.origin_event);

        if movement.0 != 0. || movement.1 != 0 {
            return (
                Transition::DoNothing,
                Some(vec![GridMessage::MoveSelection(movement, false)]),
            );
        }

        (Transition::DoNothing, None)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if modifiers.logo() || modifiers.control() {
            (
                Transition::ChangeState(Box::new(MovingSelectionUnquantized::from_args(
                    self.origin,
                    self.origin_event,
                ))),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
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
}

#[derive(Debug, Default)]
struct MovingSelectionUnquantized {
    origin: Point,
    origin_event: (usize, usize, GridEvent),
}

impl MovingSelectionUnquantized {
    fn from_args(point: Point, event: (usize, usize, GridEvent)) -> Self {
        MovingSelectionUnquantized {
            origin: point,
            origin_event: event,
        }
    }
}

impl WidgetState for MovingSelectionUnquantized {
    fn on_cursor_moved(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        // cursor cannot get out of the grid area (container padding excluded)
        // let padded_cursor = pad_cursor(cursor, bounds);
        let drag_bounds = Rectangle {
            x: self.origin.x,
            y: self.origin.y,
            width: cursor.x - self.origin.x,
            height: cursor.y - self.origin.y,
        };

        if drag_bounds.width.abs() >= 1. || drag_bounds.height.abs() >= 1. {
            let movement = base_pattern.move_selection_unquantized(
                bounds,
                drag_bounds,
                cursor,
                self.origin_event,
            );

            return (
                Transition::DoNothing,
                Some(vec![GridMessage::MoveSelection(movement, false)]),
            );
        }

        (Transition::DoNothing, None)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if !modifiers.logo() && !modifiers.control() {
            (
                Transition::ChangeState(Box::new(MovingSelectionQuantized::from_args(
                    self.origin,
                    self.origin_event,
                ))),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
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
}
