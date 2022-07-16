mod idle;
mod logoctrl;
mod shift;

pub use idle::Idle;
pub use logoctrl::LogoCtrl;
pub use shift::Shift;

use super::WidgetContext;
use crate::core::grid::{GridMessage,GridPattern};
use iced_native::{keyboard, Point, Rectangle};
use std::fmt::Debug;

pub trait WidgetState: Debug {
    fn on_cancelled(&mut self, _context: &mut WidgetContext) {}

    fn on_blur(
        &mut self
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, Some(vec![GridMessage::EmptySelection()]))
    }

    fn on_click(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn on_double_click(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn on_button_release(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn on_cursor_moved(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _base_pattern: GridPattern,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn on_key_pressed(
        &mut self,
        _key_code: keyboard::KeyCode,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn on_key_released(
        &mut self,
        _key_code: keyboard::KeyCode,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn on_modifier_change(
        &mut self,
        _modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext,
    ) -> (Transition, Option<Vec<GridMessage>>) {
        (Transition::DoNothing, None)
    }

    fn next(&mut self, _next_state: Box<dyn WidgetState + Send>) {}
}

#[derive(Debug)]
pub enum Transition {
    ChangeState(Box<dyn WidgetState + Send>),
    ChangeParentState(Box<dyn WidgetState + Send>),
    DoNothing,
}
