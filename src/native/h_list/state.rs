use iced_native::Point;

#[derive(Debug, Clone)]
pub struct State<T> {
    pub(super) panes: Vec<T>,
    pub(super) internal: Internal,
}

impl<T> State<T>
where
    T: Clone,
{
    pub fn new(panes: Vec<T>) -> Self {
        Self {
            panes: panes.clone(),
            internal: Internal {
                action: Action::Idle,
            },
        }
    }

    pub fn len(&self) -> usize {
        self.panes.len()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.panes.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.panes.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.panes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.panes.iter_mut()
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.panes.swap(a, b);
    }

    pub fn replace(&mut self, index: usize, pattern: T) {
        self.panes[index] = pattern;
    }

    pub fn push(&mut self, pattern: T) {
        self.panes.push(pattern);
    }

    pub fn remove(&mut self, index: usize) {
        self.panes.remove(index);
    }
}
#[derive(Debug, Clone)]
pub struct Internal {
    // last_id: usize,
    action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Idle,
    Dragging { index: usize, origin: Point },
}

impl Internal {
    pub fn picked_pane(&self) -> Option<(usize, Point)> {
        match self.action {
            Action::Dragging { index, origin, .. } => Some((index, origin)),
            _ => None,
        }
    }

    pub fn pick_pane(&mut self, pick_index: &usize, origin: Point) {
        self.action = Action::Dragging {
            index: *pick_index,
            origin,
        };
    }

    pub fn idle(&mut self) {
        self.action = Action::Idle;
    }
}
