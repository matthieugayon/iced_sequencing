use iced_native::{
    layout, mouse, Element, Hasher, Layout,
    Length, Point, Rectangle, Size,
    Background, Color, Widget
};

use iced_graphics::{Backend, Primitive, Renderer, Defaults};

use std::hash::Hash;

use ganic_no_std::{pattern::Pattern, NUM_PERCS, NUM_STEPS};

use crate::core::grid::{GridPattern, GridEvent}; 
use crate::core::utils::get_step_dimension; 

pub use crate::style::snapshot::{Style, StyleSheet, Default};

pub struct Snapshot {
    pattern: GridPattern,
    selected: bool,
    width: Length,
    height: Length,
    style: Box<dyn StyleSheet>
}

impl Snapshot {
    pub fn new(
        pattern: Option<Pattern>,
        width: Length,
        height: Length
    ) -> Self {
        let pattern= {
            match pattern {
                Some(pattern) => {
                    GridPattern::from(pattern)
                }
                None => {
                    GridPattern::new()
                }
            }
        };

        Snapshot {
            pattern,
            selected: false,
            width,
            height,
            style: Box::new(Default)
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

    pub fn style(mut self, style: Box<dyn StyleSheet>) -> Self {
        self.style = style.into();
        self
    }

    pub fn new_pattern(&mut self, pattern: Pattern) {
        self.pattern = GridPattern::from(pattern);
    }

    pub fn select(&mut self, select: bool) {
        self.selected = select;
    }
}


impl<Message, B> Widget<Message, Renderer<B>> for Snapshot
where
    B: Backend,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer<B>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);
        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }

    fn draw(
        &self,
        _renderer: &mut Renderer<B>,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        let bounds = layout.bounds();
    
        let mut events: Vec<(usize, usize, GridEvent)> = self.pattern.data
            .iter()
            .map(|((step, track), grid_event)| {
                (*step, *track, *grid_event)
            })
            .collect();

        events.sort_by(|x,y| {
            if x.1 == y.1 {
                return x.0.cmp(&y.0)
            }
            x.1.cmp(&y.1)
        });

        let step_dim: Rectangle = get_step_dimension(bounds, NUM_STEPS + 1, NUM_PERCS);
        let offset_x = step_dim.x;
        let offset_y = step_dim.y;

        let style: Style = if self.selected { self.style.default() } else { self.style.selected() };

        let mut primitives: Vec<Primitive> = events.iter().map(|(step, track, grid_event)| {
            Primitive::Quad {
                bounds: Rectangle{
                    x: bounds.x + offset_x + ((*step as f32 + grid_event.offset) * step_dim.width).floor(),
                    y: bounds.y + offset_y + *track as f32 * step_dim.height,
                    width: step_dim.width,
                    height: step_dim.height
                },
                background: Background::Color(style.step_color),
                border_radius: 0.,
                border_width: 0.,
                border_color: Color::TRANSPARENT
            }
        }).collect();

        primitives.push(
            Primitive::Quad {
                bounds,
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0.,
                border_width: 1.,
                border_color: Color::WHITE
            }
        );

        (
            Primitive::Group { primitives },
            mouse::Interaction::default(),
        )
    }
}

impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for Snapshot
where
    B: Backend,
{
    fn into(self) -> Element<'a, Message, Renderer<B>> {
        Element::new(self)
    }
}
