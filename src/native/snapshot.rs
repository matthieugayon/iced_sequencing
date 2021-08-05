use iced_native::{
    layout, Element, Hasher, Layout, Event, Clipboard, Padding,
    Length, Point, Rectangle, Size, Widget, overlay, event
};

use std::hash::Hash;
use ganic_no_std::pattern::Pattern;

use crate::core::grid::GridPattern; 

pub use crate::style::snapshot::{Style, StyleSheet};

pub struct Snapshot<'a, Message, Renderer: self::Renderer> {
    pattern: GridPattern,
    selected: bool,
    width: Length,
    height: Length,
    style: Renderer::Style,
    padding: Padding,
    controls: Option<Element<'a, Message, Renderer>>,
    always_show_controls: bool
}

impl<'a, Message, Renderer> Snapshot<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
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
            style: Renderer::Style::default(),
            padding: Padding::ZERO,
            controls: None,
            always_show_controls: false
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

    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
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

    pub fn controls(
        mut self,
        controls: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.controls = Some(controls.into());
        self
    }

    pub fn always_show_controls(mut self) -> Self {
        self.always_show_controls = true;
        self
    }
}


impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Snapshot<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height).pad(self.padding);
        let max_size = limits.max();
        let size = limits.resolve(Size::ZERO);
        let snapshot_layout = layout::Node::new(size);

        let mut node = if let Some(controls) = &self.controls {
            let mut controls_layout = controls
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let controls_size = controls_layout.size();
            let space_before_controls = max_size.width - controls_size.width;

            controls_layout.move_to(Point::new(space_before_controls, 0.0));

            layout::Node::with_children(
                Size::new(max_size.width, size.height),
                vec![snapshot_layout, controls_layout],
            )  
        } else {
            layout::Node::with_children(
                Size::new(max_size.width, size.height),
                vec![snapshot_layout],
            )
        };     
        
        node.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        layout::Node::with_children(node.size().pad(self.padding), vec![node])
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
        self.height.hash(state);
        self.padding.hash(state);

        if let Some(controls) = &self.controls {
            controls.hash_layout(state);
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let snapshot_layout = children.next().unwrap();

        let controls = if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();
            let show_controls = snapshot_layout.bounds().contains(cursor_position);

            if show_controls || self.always_show_controls {
                Some((controls, controls_layout))
            } else {
                None
            }
        } else {
            None
        };

        renderer.draw(
            layout.bounds(),
            self.pattern.clone(),
            self.selected,
            &self.style,
            controls,
            cursor_position,
            viewport
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        // snapshot layout
        children.next();

        if let Some(controls) = &mut self.controls {
            let controls_layout = children.next().unwrap();

            controls.on_event(
                event.clone(),
                controls_layout,
                cursor_position,
                renderer,
                clipboard,
                messages,
            )
        } else {
            event::Status::Ignored
        }
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        // snapshot layout
        children.next();

        let Self { controls, .. } = self;

        match controls {
            Some(ctr) => {
                let controls_layout = children.next()?;
                ctr.overlay(controls_layout)
            },
            None => {
                None
            },
        }
    }
}

pub trait Renderer: iced_native::Renderer {
    type Style: Default;

    fn draw<Message>(
        &mut self,
        bounds: Rectangle,
        pattern: GridPattern,
        selected: bool,
        style: &Self::Style,
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
        viewport: &Rectangle
    ) -> Self::Output;
}


impl<'a, Message, Renderer> From<Snapshot<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer,
{
    fn from(
        snapshot: Snapshot<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(snapshot)
    }
}