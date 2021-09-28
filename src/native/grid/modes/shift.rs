use super::Idle;
use super::LogoCtrl;
use super::{Transition, WidgetContext, WidgetState};
use crate::core::grid::{
    get_hovered_track, get_hovered_step, get_step_width, 
    GridMessage, GridPattern
};
use iced_native::{keyboard, Point, Rectangle, mouse};

#[derive(Debug)]
pub struct Shift {
    nested: Box<dyn WidgetState + Send>,
}

impl WidgetState for Shift {
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
    fn on_double_click(
        &mut self,
        bounds: Rectangle,
        cursor: Point,
        base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        let mut grid_messages = vec![
            GridMessage::TrackSelected(get_hovered_track(cursor, bounds)),
        ];

        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            // if yes toggle select flag for event
            Some(((step, track), _)) => {
                grid_messages.push(GridMessage::ToggleOne((*step, *track)));
            },
            // otherwise add event
            None => {
                grid_messages.push(GridMessage::EmptySelection());

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
        ];
        
        // check if we hover an event on the grid
        match base_pattern.get_hovered(cursor, bounds) {
            Some(((step, track), _)) => {
                grid_messages.push(GridMessage::ToggleOne((*step, *track)));
                (Transition::DoNothing, Some(grid_messages))
            },
            // otherwise add event
            None => {
                grid_messages.push(GridMessage::EmptySelection());

                (
                    Transition::ChangeState(Box::new(Selecting::from_args(cursor))),
                    Some(grid_messages),
                )
            },
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
        } else if !modifiers.shift() && !modifiers.logo() && !modifiers.control() {
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
        if modifiers.logo() || modifiers.control() {
            (
                Transition::ChangeParentState(Box::new(LogoCtrl::selecting(self.origin))),
                None,
            )
        } else if !modifiers.shift() && !modifiers.logo() && !modifiers.control() {
            (
                Transition::ChangeParentState(Box::new(Idle::selecting(self.origin))),
                None,
            )
        } else {
            (Transition::DoNothing, None)
        }
    }
}
