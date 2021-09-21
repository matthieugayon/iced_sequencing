use super::Logo;
use super::Shift;
use super::{Transition, WidgetContext, WidgetState};
use crate::core::grid::{
    get_hovered_step, get_hovered_track, get_step_width, GridEvent, GridMessage, GridMessageKind,
    GridPattern, Target,
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
        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            // if yes remove event
            Some((grid_id, _grid_event)) => (
                Transition::DoNothing,
                Some(vec![GridMessage {
                    message: GridMessageKind::Delete(*grid_id),
                    target: Target::STATE,
                }]),
            ),
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

                    (
                        Transition::DoNothing,
                        Some(vec![GridMessage {
                            message: GridMessageKind::Add((step, track, 0.)),
                            target: Target::STATE,
                        }]),
                    )
                } else {
                    (Transition::DoNothing, None)
                }
            }
        }
    }

    fn on_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            // if yes remove event
            Some(((step, track), grid_event)) => {
                let mut grid_messages = vec![GridMessage {
                    message: GridMessageKind::TrackSelected(*track),
                    target: Target::STATE,
                }];

                if !grid_event.selected {
                    grid_messages.push(GridMessage {
                        message: GridMessageKind::SelectOne((*step, *track)),
                        target: Target::STATE,
                    })
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
                let grid_message = {
                    if bounds.contains(cursor) {
                        Some(vec![GridMessage {
                            message: GridMessageKind::TrackSelected(get_hovered_track(
                                cursor, bounds,
                            )),
                            target: Target::STATE,
                        }])
                    } else {
                        None
                    }
                };

                (
                    Transition::ChangeState(Box::new(Selecting::from_args(cursor))),
                    grid_message,
                )
            }
        }
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if modifiers.logo() {
            (
                Transition::ChangeParentState(Box::new(Logo::default())),
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
        let message_kind = match key_code {
            keyboard::KeyCode::A => Some(GridMessageKind::SelectAll()),
            keyboard::KeyCode::Backspace => Some(GridMessageKind::DeleteSelection()),
            keyboard::KeyCode::Left => Some(GridMessageKind::MoveSelection((-1., 0))),
            keyboard::KeyCode::Up => Some(GridMessageKind::MoveSelection((0., -1))),
            keyboard::KeyCode::Right => Some(GridMessageKind::MoveSelection((1., 0))),
            keyboard::KeyCode::Down => Some(GridMessageKind::MoveSelection((0., 1))),
            _ => None,
        };

        match message_kind {
            Some(grid_message_kind) => (
                Transition::DoNothing,
                Some(vec![GridMessage {
                    message: grid_message_kind,
                    target: Target::STATE,
                }]),
            ),
            None => (Transition::DoNothing, None),
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
                cursor.x
            } else {
                self.origin.x
            },
            y: if cursor.y - self.origin.y < 0.0 {
                cursor.y
            } else {
                self.origin.y
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
            Some(vec![GridMessage {
                message: GridMessageKind::SelectArea(selection, bounds.size()),
                target: Target::UI,
            }]),
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

        (
            Transition::ChangeState(Box::new(Waiting::default())),
            Some(vec![GridMessage {
                message: GridMessageKind::CommitState(),
                target: Target::STATE,
            }]),
        )
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if modifiers.logo() {
            (Transition::ChangeState(Box::new(Logo::default())), None)
        } else {
            (Transition::DoNothing, None)
        }
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

        if movement.0 != 0. && movement.1 != 0 {
            return (
                Transition::DoNothing,
                Some(vec![GridMessage {
                    message: GridMessageKind::MoveSelection(movement),
                    target: Target::UI,
                }]),
            );
        }

        (Transition::DoNothing, None)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if modifiers.logo() {
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

        (
            Transition::ChangeState(Box::new(Waiting::default())),
            Some(vec![GridMessage {
                message: GridMessageKind::CommitState(),
                target: Target::STATE,
            }]),
        )
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
                Some(vec![GridMessage {
                    message: GridMessageKind::MoveSelection(movement),
                    target: Target::UI,
                }]),
            );
        }

        (Transition::DoNothing, None)
    }

    fn on_modifier_change(
        &mut self,
        modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        if !modifiers.logo() {
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

        (
            Transition::ChangeState(Box::new(Waiting::default())),
            Some(vec![GridMessage {
                message: GridMessageKind::CommitState(),
                target: Target::STATE,
            }]),
        )
    }
}
