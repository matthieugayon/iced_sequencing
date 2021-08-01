// Import iced modules.
use iced::{
    Sandbox, Settings, container, 
    Element, Color, Column, Align,
    Container, Length, button, Button,
    Text
};

// Import iced_audio sequencing.
use iced_sequencing::{grid, snapshot, h_list, multi_slider};
use iced_sequencing::style::color_utils::hex;
use ganic_no_std::pattern::Pattern;

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
    SetPattern(Pattern),
    FocusTrack(usize),
    Dragged(h_list::DragEvent),
    Clicked(usize),
    SetVelocities(Vec<f32>),
    AddSnapshotPressed
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

pub struct App {
    add_snapshot_button: button::State,
    grid_state: grid::State,
    multi_slider: multi_slider::State,
    snapshot_list: h_list::State<Option<Pattern>>,
    current_snapshot: usize,
    focused_track: usize
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        let test_pattern = Some(Pattern::new_test());
        App {
            add_snapshot_button: button::State::new(),
            grid_state: grid::State::new(test_pattern),
            multi_slider: multi_slider::State::new(),
            snapshot_list: h_list::State::new(vec![test_pattern]),
            current_snapshot: 0,
            focused_track: 0
        }
    }

    fn title(&self) -> String {
        format!("Simple Example - Iced Sequencing")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::SetPattern(pattern) => {
                // update snapshot list 
                self.snapshot_list.replace(self.current_snapshot, Some(pattern));
            }
            Message::SetVelocities(values) => {
                let mut current_snapshot = self.snapshot_list.get_mut(self.current_snapshot).unwrap().unwrap();                
                values.into_iter()
                    .enumerate()
                    .for_each(|(step, vel)| {
                        current_snapshot.data[step][self.focused_track][0] = vel;
                    });

                // update snapshot list state
                self.snapshot_list.replace(self.current_snapshot, Some(current_snapshot));

                // update grid state
                self.grid_state.new_pattern(current_snapshot);
            },
            Message::Dragged(h_list::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.snapshot_list.swap(pane, target);
            },
            Message::Dragged(_) => {},
            Message::Clicked(index) => {
                self.current_snapshot = index;

                // update grid state
                let current_snapshot = self.snapshot_list.get(self.current_snapshot).unwrap().unwrap();
                self.grid_state.new_pattern(current_snapshot);
            }
            Message::FocusTrack(track) => {
                self.focused_track = track;
            },
            Message::AddSnapshotPressed => {
                self.snapshot_list.push(Some(Pattern::new_test()))
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let grid = grid::Grid::new(
            &mut self.grid_state, 
            Message::SetPattern,
            Message::FocusTrack,
            Length::from(Length::Units(690)),
            Length::from(Length::Units(345))
        );

        let current_velocities = self.snapshot_list.get(self.current_snapshot)
            .unwrap()
            .unwrap()
            .velocities(self.focused_track)
            .to_vec();

        let multi_slider = multi_slider::MultiSlider::new(
                &mut self.multi_slider,
                (0 as f32)..=1.0,
                current_velocities,
                Message::SetVelocities,
                hex("ff7d00")
            )
            .spacing(2)
            .height(Length::from(Length::Units(120)))
            .step(0.01);
        
        let list = h_list::HList::new(&mut self.snapshot_list, |pane| {
                let snapshot = snapshot::Snapshot::new(
                    *pane,
                    Length::Fill,
                    Length::Fill
                );
                
                h_list::Content::new(snapshot)
            })
            .width(Length::Fill)
            .height(Length::from(Length::Units(50)))
            .spacing(0)
            .on_click(Message::Clicked)
            .on_drag(Message::Dragged);

        let content: Element<_> = Column::new()
            .max_width(690)
            .align_items(Align::Center)
            .push(
                Button::new(&mut self.add_snapshot_button, Text::new("Add new snapshot"))
                    .on_press(Message::AddSnapshotPressed),
            )
            .push(list)
            .push(grid)
            .push(multi_slider)
            .spacing(16)
            .into();

        Container::new(content)
            .width(Length::from(Length::Units(690)))
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
