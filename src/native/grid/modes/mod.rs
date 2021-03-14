mod idle;
mod logo;
mod shift;

pub use idle::Idle;
pub use logo::Logo;
pub use shift::Shift;

use std::fmt::Debug;
use iced_native::{keyboard, Rectangle, Point};
use super::WidgetContext;
use crate::core::grid::GridMessage;

pub trait WidgetState: Debug {
    fn on_cancelled(&mut self, _context: &mut WidgetContext) {}

    fn on_click(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn on_double_click(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn on_button_release(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn on_cursor_moved(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn on_key_pressed(
        &mut self,
        _key_code: keyboard::KeyCode,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn on_key_released(
        &mut self,
        _key_code: keyboard::KeyCode,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn on_modifier_change(
        &mut self,
        _modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext
    ) -> (Transition, Option<GridMessage>) {
        (Transition::DoNothing, None)
    }

    fn next(
       &mut self,
       _next_state: Box<dyn WidgetState + Send>
    ) {}
}


#[derive(Debug)]
pub enum Transition {
    ChangeState(Box<dyn WidgetState + Send>),
    ChangeParentState(Box<dyn WidgetState + Send>),
    DoNothing,
}

