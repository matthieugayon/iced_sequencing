// Import iced modules.
use iced::{
    Sandbox, Settings, container, 
    Element, Color, Column, Align,
    Container, Length
};

// Import iced_audio sequencing.
use iced_sequencing::{grid, snapshot, h_list};
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
    NewPattern(Pattern),
    Dragged(h_list::DragEvent)
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

pub struct App {
    grid_state: grid::State,
    panes: h_list::State<Option<Pattern>>
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        App {
            grid_state: grid::State::new(None),
            panes: h_list::State::new(vec![
                Some(Pattern::new_test()),
                Some(Pattern::new_test()),
                Some(Pattern::new_test()),
                None,
                None,
                None,
            ])
        }
    }

    fn title(&self) -> String {
        format!("Simple Example - Iced Sequencing")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::NewPattern(_pattern) => {
                // self.output_text = format!("new pattern: {}", pattern.data);
                // println!("{:?}", pattern.data);
            }
            Message::Dragged(h_list::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.panes.swap(pane, target);
            }
            Message::Dragged(_) => {}
        }
    }

    fn view(&mut self) -> Element<Message> {
        // let grid_widget = grid::Grid::new(
        //     &mut self.grid_state, 
        //     Message::NewPattern,
        //     Length::from(Length::Units(690)),
        //     Length::from(Length::Units(345))
        // );

        // let snapshot_test = snapshot::Snapshot::new(
        //     Some(Pattern::new_test()),
        //     Length::from(Length::Units(130)),
        //     Length::from(Length::Units(85))
        // );
        
        let list = h_list::HList::new(&mut self.panes, |pane| {
                let snapshot = snapshot::Snapshot::new(
                    *pane,
                    Length::Fill,
                    Length::Fill
                );
                
                h_list::Content::new(snapshot)
            })
            .width(Length::Fill)
            .height(Length::from(Length::Units(90)))
            .spacing(20)
            .on_drag(Message::Dragged);

        let content: Element<_> = Column::new()
            .max_width(690)
            .align_items(Align::Center)
            // .push(grid_widget)
            // .push(snapshot_test)
            .push(list)
            .into();

        Container::new(content)
            .width(Length::from(Length::Units(690)))
            .height(Length::Fill)
            .center_x()
            .into()
    }
}
