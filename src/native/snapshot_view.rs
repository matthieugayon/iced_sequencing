use crate::core::{
    grid::{GridEvent, GridPattern},
    utils::get_step_dimension,
};
pub use crate::style::snapshot::{Style, StyleSheet};
use ganic_no_std::{pattern::Pattern, NUM_PERCS, NUM_STEPS};

use iced_native::{
    event, layout, mouse, renderer, Background, Clipboard, Color, Element, Event, Layout, Length,
    Padding, Point, Rectangle, Shell, Size, Widget,
};

pub enum SelectionState {
    Selected(),
    NotSelected(),
    Dirty(),
}

pub struct SnapshotView<'a> {
    pattern: GridPattern,
    selection_state: SelectionState,
    width: Length,
    height: Length,
    style_sheet: Box<dyn StyleSheet + 'a>,
    padding: Padding,
}

impl<'a> SnapshotView<'a> {
    pub fn new(pattern: GridPattern, width: Length, height: Length) -> Self {
        SnapshotView {
            pattern,
            selection_state: SelectionState::NotSelected(),
            width,
            height,
            style_sheet: Default::default(),
            padding: Padding::ZERO,
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet + 'a>>) -> Self {
        self.style_sheet = style.into();
        self
    }

    pub fn new_pattern(mut self, pattern: Pattern) -> Self {
        self.pattern = GridPattern::from(pattern);
        self
    }

    pub fn set_selection_state(mut self, state: SelectionState) -> Self {
        self.selection_state = state;
        self
    }
}


impl<'a, Message, Renderer> Widget<Message, Renderer> for SnapshotView<'a>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        {
            let bounds = layout.bounds();
            let pattern = &self.pattern;
            let style: &dyn StyleSheet = self.style_sheet.as_ref();

            let mut events: Vec<(usize, usize, GridEvent)> = pattern
                .data
                .iter()
                .map(|((step, track), grid_event)| (*step, *track, *grid_event))
                .collect();

            events.sort_by(|x, y| {
                if x.1 == y.1 {
                    return x.0.cmp(&y.0);
                }
                x.1.cmp(&y.1)
            });

            let step_bounds = Rectangle {
                height: bounds.height,
                y: bounds.y,
                ..bounds
            };
            let step_dim: Size = get_step_dimension(step_bounds, NUM_STEPS + 2, NUM_PERCS);
            let step_width = 0.85 * step_dim.width;
            let step_height = (step_dim.height - 1.).floor();

            let style: Style = match self.selection_state {
                SelectionState::Selected() => style.selected(),
                SelectionState::NotSelected() => style.default(),
                SelectionState::Dirty() => style.dirty(),
            };

            let division: usize = {
                if step_dim.width <= 2. {
                    8
                } else if step_dim.width <= 3. {
                    4
                } else {
                    2
                }
            };
            let grid = NUM_STEPS / division;

            if style.background.is_some() || style.border_width > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: style.border_radius,
                        border_width: style.border_width,
                        border_color: style.border_color,
                    },
                    style
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                );
            }

            (0..=grid).into_iter().for_each(|step| {
                let color = {
                    if step == 0 || step == grid {
                        style.line_edge_color
                    } else {
                        style.line_division_color
                    }
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            // x: (bounds.x + ((division * step + 1) as f32 * step_dim.width)).round(),
                            x: bounds.x + ((division * step + 1) as f32 * step_dim.width),
                            y: bounds.y,
                            width: step_dim.width + 1.,
                            height: bounds.height,
                        },
                        border_radius: 0.,
                        border_width: 1.,
                        border_color: color,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            });

            events.iter().for_each(|(step, track, grid_event)| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            // x: (step_bounds.x + (((*step + 1) as f32 + grid_event.offset) * step_dim.width)).round(),
                            x: step_bounds.x
                                + (((*step + 1) as f32 + grid_event.offset) * step_dim.width),
                            y: step_bounds.y + (*track as f32 * step_dim.height),
                            width: step_width,
                            height: step_height,
                        },
                        border_radius: 0.,
                        border_width: 0.,
                        border_color: Color::TRANSPARENT,
                    },
                    Background::Color(style.step_color),
                );
            });
        }
    }

    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, Message, Renderer> From<SnapshotView<'a>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Message: 'a,
{
    fn from(snapshot: SnapshotView<'a>) -> Element<'a, Message, Renderer> {
        Element::new(snapshot)
    }
}
