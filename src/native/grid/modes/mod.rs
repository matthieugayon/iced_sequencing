mod idle;

pub use idle::Idle;

use std::fmt::Debug;
use iced_native::{event, keyboard, mouse, Event, Rectangle, Point};

use super::WidgetContext;

pub trait WidgetState: Debug {
    fn on_cancelled(&mut self, _context: &mut WidgetContext) {}

    fn on_click(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }

    fn on_double_click(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }

    fn on_button_release(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }

    fn on_cursor_moved(
        &mut self,
        _bounds: Rectangle,
        _cursor: Point,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }

    fn on_key_pressed(
        &mut self,
        _keyCode: keyboard::KeyCode,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }

    fn on_key_released(
        &mut self,
        _keyCode: keyboard::KeyCode,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }

    fn on_modifier_change(
        &mut self,
        _modifiers: keyboard::Modifiers,
        _context: &mut WidgetContext
    ) -> Transition {
        Transition::DoNothing
    }
}


#[derive(Debug)]
pub enum Transition {
    ChangeState(Box<dyn WidgetState>),
    DoNothing,
}

