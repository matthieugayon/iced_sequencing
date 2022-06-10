use ganic_no_std::{pattern::Pattern, NUM_STEPS, NUM_PERCS};
use crate::core::{grid::{GridPattern, GridEvent}, utils::get_step_dimension};
pub use crate::style::snapshot::{Style, StyleSheet};

use iced_native::{
    event, layout, Clipboard,
    Element, Event, Layout, Length, Padding, Point,
    Rectangle, Size, Widget, Shell, renderer, Color,
    Background, mouse
};

pub struct Snapshot<'a> {
    pattern: GridPattern,
    selected: bool,
    width: Length,
    height: Length,
    style_sheet: Box<dyn StyleSheet + 'a>,
    padding: Padding
}

impl<'a> Snapshot<'a> {
    pub fn new(pattern: GridPattern, width: Length, height: Length) -> Self {
        Snapshot {
            pattern,
            selected: false,
            width,
            height,
            style_sheet: Default::default(),
            padding: Padding::ZERO
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

    pub fn select(mut self, select: bool) -> Self {
        self.selected = select;
        self
    }
}

pub fn draw<'a, Message, Renderer> (
    renderer: &mut Renderer,
    bounds: Rectangle,
    pattern: GridPattern,
    selected: bool,
    style: &dyn StyleSheet,
    cursor_position: Point,
    viewport: &Rectangle,
) where
    Renderer: iced_native::Renderer,
{
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
        height: bounds.height - 3.,
        y: bounds.y + 2.,
        ..bounds
    };
    let step_dim: Size = get_step_dimension(step_bounds, NUM_STEPS + 2, NUM_PERCS);
    let step_width = 0.85 * step_dim.width;
    let step_height = (step_dim.height - 1.).floor();
    let style: Style = if selected {
        style.selected()
    } else {
        style.default()
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
            style.background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    (0..=grid).into_iter()
        .for_each(|step| {
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
                        y: bounds.y + 1.,
                        width: step_dim.width + 1.,
                        height: bounds.height - 2.,
                    },
                    border_radius: 0.,
                    border_width: 1.,
                    border_color: color
                },
                Background::Color(Color::TRANSPARENT),
            );
        });

    events.iter().for_each(|(step, track, grid_event)| {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    // x: (step_bounds.x + (((*step + 1) as f32 + grid_event.offset) * step_dim.width)).round(),
                    x: step_bounds.x + (((*step + 1) as f32 + grid_event.offset) * step_dim.width),
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

impl<'a, Message, Renderer> Widget<Message, Renderer> for Snapshot<'a>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(Length::Units(self.height));
        let size = limits.resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        draw(
            renderer,
            layout.bounds(),
            self.pattern,
            self.selected,
            self.style_sheet.as_ref(),
            cursor_position,
            viewport,
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, Message, Renderer> From<Snapshot<'a>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Message: 'a,
{
    fn from(snapshot: Snapshot<'a>) -> Element<'a, Message, Renderer> {
        Element::new(snapshot)
    }
}