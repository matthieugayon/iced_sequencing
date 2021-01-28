// Import iced modules.
use iced::{Align, Column, Container, Element, Length, Sandbox, Settings, Text, Color, Row};
// Import iced_audio modules.
use iced_sequencing::{grid, multi_slider};
use ganic_no_std::pattern::Pattern;

#[derive(Debug, Clone)]
pub enum Message {
    NewPattern(Pattern),
    NewSliders(Vec<f32>),
}

pub fn main() {
    App::run(Settings::default()).unwrap();
}

pub struct App {
    grid_state: grid::State,
    multi_slider_1_state: multi_slider::State,
    multi_slider_2_state: multi_slider::State,
    output_text: String
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> App {
        App {
            grid_state: grid::State::new(None),
            multi_slider_1_state: multi_slider::State::new(vec![]),
            multi_slider_2_state: multi_slider::State::new(vec![]),
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
            Message::NewSliders(values) => {
                println!("{:?}", values);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let multi_slider_widget_1 = multi_slider::MultiSlider::new(
            &mut self.multi_slider_1_state,
            Message::NewSliders,
            100,
            100,
            25,
            Color::from_rgba(0.7, 0.7, 0.0, 1.0)
        );

        let multi_slider_widget_2 = multi_slider::MultiSlider::new(
            &mut self.multi_slider_2_state,
            Message::NewSliders,
            200,
            75,
            10,
            Color::from_rgba(0.0, 0.7, 0.7, 1.0)
        );

        let sliders_row = Row::new()
            .padding(24)
            .align_items(Align::Center)
            .push(Column::new().padding(24).push(multi_slider_widget_1))
            .push(Column::new().padding(24).push(multi_slider_widget_2));

        let grid_widget = grid::Grid::new(
            &mut self.grid_state, 
            Message::NewPattern,
            Length::from(Length::Units(984)),
            Length::from(Length::Units(376))
        );

        let content: Element<_> = Column::new()
            .max_width(984)
            .align_items(Align::Center)
            .push(Text::new(&self.output_text))
            .push(grid_widget)
            .push(sliders_row)
            .into();


        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}
