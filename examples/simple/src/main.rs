// Import iced modules.
use iced::{
    Button, Color, Column, Container,
    Element, Length, Sandbox, Settings, Text, button, 
    container, Alignment
};

use iced_native::Padding;

// Import iced_audio sequencing.
use iced_sequencing::{grid, snapshot, h_list, multi_slider};
use iced_sequencing::core::grid::{
    GridEvent,
    GridPattern,
    GridMessage
};
use iced_sequencing::style::color_utils::hex;
use ganic_no_std::{
    pattern::Pattern,
    NUM_PERCS
};

const WINDOW_BG_COLOR: Color = Color::from_rgb(
    0x25 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x2A as f32 / 255.0, //180b28
);

pub struct MainContainerStyle;

impl container::StyleSheet for MainContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: WINDOW_BG_COLOR.into(),
            ..container::Style::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GridEvent(GridMessage),
    Dragged(h_list::DragEvent),
    Clicked(usize),
    SetVelocities(Vec<f32>),
    AddSnapshotPressed,
    Delete(usize)
}

pub fn main() {
    App::run(Settings {
        antialiasing: true,
        ..Settings::default()
    }).unwrap();
}

pub struct App {
    add_snapshot_button: button::State,
    grid_state: grid::State,
    live_pattern: GridPattern,
    multi_slider: multi_slider::State,
    snapshot_list: h_list::State<Item>,
    current_snapshot: usize,
    focused_track: usize
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        let test = Item::new(None);
        let initial_pattern = GridPattern::from(test.data.unwrap());

        App {
            add_snapshot_button: button::State::new(),
            grid_state: grid::State::new(initial_pattern.clone()),
            live_pattern: initial_pattern,
            multi_slider: multi_slider::State::new(),
            snapshot_list: h_list::State::new(vec![test]),
            current_snapshot: 0,
            focused_track: 0
        }
    }

    fn title(&self) -> String {
        format!("Simple Example - Iced Sequencing")
    }

    fn update(&mut self, event: Message) {
        println!("--- update {:?}", event);

        match event {
            // Message::SetPatternState(pattern) => {
            //     // update widget state
            //     self.live_pattern = pattern;
            //     self.grid_state.set_pattern(pattern);
            //     self.snapshot_list.replace(self.current_snapshot, Item::new(Some(pattern)));
            // },
            // Message::SetPatternUI(pattern) => {
            //     // update widget temporary state
            //     self.live_pattern = pattern;
            //     self.grid_state.temporary_pattern(pattern);
            //     self.snapshot_list.replace(self.current_snapshot, Item::new(Some(pattern)));
            // },
            Message::GridEvent(grid_message) => {
                let mut next_grid = self.grid_state.clone_base_pattern();

                match grid_message {
                    GridMessage::EmptySelection() => {
                        self.live_pattern.empty_selection();
                        self.grid_state.set_pattern(self.live_pattern.clone());
                    },
                    GridMessage::Add((step, track, offset)) => {
                        next_grid.data.insert((step, track), GridEvent {
                            offset,
                            ..GridEvent::default()
                        });
                        self.live_pattern = next_grid;
                        self.grid_state.set_pattern(self.live_pattern.clone());
                    },
                    GridMessage::Delete(grid_id) => {
                        next_grid.data.remove(&grid_id);
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::ToggleOne(grid_id) => {
                        next_grid.toggle_select(grid_id);
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::SelectOne(grid_id) => {
                        next_grid.select_one(grid_id);
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::SelectArea(selection, bounds) => {
                        next_grid.select_area(selection, bounds);
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::SelectAll() => {
                        next_grid.select_all();
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::ToggleArea(selection, bounds) => {
                        next_grid.toggle_area(selection, bounds);
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::MoveSelection(next_movement, relative) => {
                        self.grid_state.set_movement(next_movement, relative);
                        
                        match self.grid_state.get_movement() {
                            Some(movement) => {
                                next_grid.move_selection(movement.0, movement.1);
                            },
                            None => {},
                        }

                        self.live_pattern = next_grid;
                    },
                    GridMessage::DeleteSelection() => {
                        next_grid.remove_selection();
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::SetVelocity(ratio) => {
                        next_grid.set_velocity(ratio);
                        self.live_pattern = next_grid.clone();
                        self.grid_state.set_pattern(next_grid);
                    },
                    GridMessage::TrackSelected(track) => {
                        self.focused_track = NUM_PERCS - track - 1;
                    },
                    GridMessage::DiscardState() => {
                        self.live_pattern = self.grid_state.clone_base_pattern();
                    },
                    _ => {}
                }

                self.snapshot_list.replace(
                    self.current_snapshot, 
                    Item::new(Some(Pattern::from(self.live_pattern.clone())))
                );
            },
            Message::SetVelocities(values) => {
                let mut current_snapshot = self.snapshot_list.get_mut(self.current_snapshot).unwrap().data.unwrap();                
                values.into_iter()
                    .enumerate()
                    .for_each(|(step, vel)| {
                        current_snapshot.data[step][self.focused_track][0] = vel;
                    });

                // update snapshot list state
                self.snapshot_list.replace(self.current_snapshot, Item::new(Some(current_snapshot)));

                // update grid state
                self.grid_state.set_pattern(GridPattern::from(current_snapshot));
            },
            Message::Dragged(h_list::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.snapshot_list.swap(pane, target);
                self.current_snapshot = target;
            },
            Message::Dragged(_) => {},
            Message::Clicked(index) => {
                self.current_snapshot = index;

                // update grid state
                let current_snapshot = self.snapshot_list.get(self.current_snapshot).unwrap().data.unwrap();
                self.grid_state.set_pattern(GridPattern::from(current_snapshot));
            },
            Message::AddSnapshotPressed => {
                self.snapshot_list.push(Item::new(Some(Pattern::new_test())))
            },
            Message::Delete(delete_index) => {
                // we first need to correct current snappshot index
                let new_index = match self.current_snapshot {
                    index if index >= delete_index && index > 0 => {
                        self.current_snapshot - 1
                    }
                    _ => {
                        self.current_snapshot
                    }
                };

                if self.current_snapshot != new_index {
                    self.current_snapshot = new_index;

                    // update grid state
                    let current_snapshot = self.snapshot_list.get(self.current_snapshot).unwrap().data.unwrap();
                    self.grid_state.set_pattern(GridPattern::from(current_snapshot));
                }
                
                self.snapshot_list.remove(delete_index);
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let current_velocities = self.snapshot_list.get(self.current_snapshot)
            .unwrap()
            .data
            .unwrap()
            .velocities(self.focused_track)
            .to_vec();

        let number_of_snapshots = self.snapshot_list.len(); 
        let current_snapshot = self.current_snapshot;
        
        let list = h_list::HList::new(&mut self.snapshot_list, |pane_index, pane| {
                let controls = Column::new()
                    .padding(5)
                    .push(pane.controls.view(pane_index, number_of_snapshots));
                    
                let is_focused = current_snapshot == pane_index;
                let snapshot = snapshot::Snapshot::new(
                        pane.data,
                        Length::Fill,
                        Length::Fill
                    )
                    .select(is_focused)
                    .controls(controls);
                
                h_list::Content::new(snapshot)
            })
            .width(Length::Fill)
            .height(Length::from(Length::Units(60)))
            .spacing(2)
            .padding(Padding::from(4))
            .on_click(Message::Clicked)
            .on_drag(Message::Dragged);

        let grid = grid::Grid::new(
            &mut self.grid_state, 
            self.live_pattern.clone(),
            Message::GridEvent,
            Length::from(Length::Units(690)),
            Length::from(Length::Units(345))
        );

        let multi_slider = multi_slider::MultiSlider::new(
                &mut self.multi_slider,
                (0 as f32)..=1.0,
                current_velocities,
                Message::SetVelocities,
                hex("ff7d00")
            )
            .spacing(2)
            .padding(Padding::from([6, 4]))
            .height(Length::from(Length::Units(120)))
            .step(0.01);

        let content: Element<_> = Column::new()
            .align_items(Alignment::Center)
            .push(
                Button::new(&mut self.add_snapshot_button, Text::new("Add new snapshot"))
                    .on_press(Message::AddSnapshotPressed),
            )
            .push(list)
            .push(grid)
            .push(multi_slider)
            .spacing(16)
            .width(Length::Units(690))
            .into();

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone)]
struct Item {
    pub data: Option<Pattern>,
    pub controls: Controls,
}

#[derive(Debug, Clone)]
struct Controls {
    close: button::State,
}

impl Item {
    fn new(pattern: Option<Pattern>) -> Self {
        let data = match pattern {
            Some(patt) => {
                Some(patt)
            },
            None => {
                Some(Pattern::new_test())
            },
        };

        Self {
            data,
            controls: Controls::new()
        }
    }
}

impl Controls {
    fn new() -> Self {
        Self {
            close: button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        snapshot_index: usize,
        number_of_items: usize
    ) -> Element<Message> {
        let mut button =
            Button::new(&mut self.close, Text::new("Del").size(10))
                .padding(2);

        if number_of_items > 1 {
            button = button.on_press(Message::Delete(snapshot_index));
        }
        button.into()
    }
}