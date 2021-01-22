// Import iced modules.
use iced::{
    Align, Column, Container, Element, Length, Sandbox, Settings, Text,
};
// Import iced_audio modules.
use iced_sequencing::grid;
use ganic_no_std::pattern::Pattern;

#[derive(Debug, Clone)]
pub enum Message {
    NewPattern(Pattern)
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

pub struct App {
    grid_state: grid::State,
    output_text: String
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        App {
            grid_state: grid::State::new(None),
            output_text: "Edit the grid!".into(),
        }
    }

    fn title(&self) -> String {
        format!("Simple Example - Iced Sequencing")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::NewPattern(pattern) => {
                // self.output_text = format!("new pattern: {}", pattern.data);
                // println!("{:?}", pattern.data);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let grid_widget = grid::Grid::new(
            &mut self.grid_state, 
            Message::NewPattern,
            Length::from(Length::Units(1000)),
            Length::from(Length::Units(400))
        );

        let content: Element<_> = Column::new()
            .max_width(1200)
            .align_items(Align::Center)
            .push(grid_widget)
            .push(Text::new(&self.output_text))
            .into();

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}
