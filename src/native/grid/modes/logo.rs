use super::{Idle, Transition, WidgetContext, WidgetState};
use crate::core::grid::{
    get_hovered_step, get_step_width, GridMessage, GridMessageKind, GridPattern, Target,
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
        context: &mut WidgetContext,
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
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width - step_width,
                    height: bounds.height,
                };

                if interactive_area.contains(cursor) {
                    let (step, track, offset) = get_hovered_step(cursor, bounds, true);
                    (
                        Transition::DoNothing,
                        Some(vec![GridMessage {
                            message: GridMessageKind::Add((step, track, offset)),
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
            // if yes select event
            Some(((step, track), grid_event)) => {
                let grid_messages = {
                    if !grid_event.selected {
                        Some(vec![GridMessage {
                            message: GridMessageKind::SelectOne((*step, *track)),
                            target: Target::STATE,
                        }])
                    } else {
                        None
                    }
                };

                (
                    Transition::ChangeState(Box::new(SetVelocity::from_args(cursor))),
                    grid_messages,
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
        let message_kind = match key_code {
            keyboard::KeyCode::A => Some(GridMessageKind::SelectAll()),
            keyboard::KeyCode::Backspace => Some(GridMessageKind::DeleteSelection()),
            keyboard::KeyCode::Left => Some(GridMessageKind::MoveSelection((-0.05, 0))),
            keyboard::KeyCode::Up => Some(GridMessageKind::MoveSelection((0., -1))),
            keyboard::KeyCode::Right => Some(GridMessageKind::MoveSelection((0.05, 0))),
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
                target: Target::NONE,
            }]),
        )
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
}

impl SetVelocity {
    fn from_args(point: Point) -> Self {
        SetVelocity { origin: point }
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
        let ratio = (self.origin.y - cursor.y).min(127.).max(-127.) / 127.;

        (
            Transition::DoNothing,
            Some(vec![GridMessage {
                message: GridMessageKind::SetVelocity(ratio),
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
                target: Target::NONE,
            }]),
        )
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
