// Import iced modules.
use iced::{
    Align, Column, Container, Element, Length, Sandbox,
    Settings, Color, container
};
// Import iced_audio modules.
use iced_sequencing::grid;
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
    NewPattern(Pattern)
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

pub struct App {
    grid_state: grid::State
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        App {
            grid_state: grid::State::new(None)
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
        }
    }

    fn view(&mut self) -> Element<Message> {
        let grid_widget = grid::Grid::new(
            &mut self.grid_state, 
            Message::NewPattern,
            Length::from(Length::Units(690)),
            Length::from(Length::Units(345))
        );

        let content: Element<_> = Column::new()
            .max_width(690)
            .align_items(Align::Center)
            .push(grid_widget)
            .into();

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(MainContainerStyle {}) 
            .into()
    }
}
