// Import iced modules.
use iced::{
    Align, Column, Container, Element, Length, Sandbox, Settings, Text,
};
// Import iced_audio modules.
use iced_sequencing::{grid};

#[derive(Debug, Clone)]
pub enum Message {
    Grid()
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
            grid_state: grid::State::new()
        }
    }

    fn title(&self) -> String {
        format!("Simple Example - Iced Sequencing")
    }

    fn update(&mut self, event: Message) {
        match event {
            // Retrieve the value by mapping the normalized value of the parameter
            // to the corresponding range.
            //
            // Now do something useful with that value!
            Message::HSliderInt(normal) => {
                // Integer parameters must be snapped to make the widget "step" when moved.
                self.h_slider_state.snap_visible_to(&self.int_range);

                let value = self.int_range.unmap_to_value(normal);
                self.output_text = format!("HSliderInt: {}", value);
            }
            Message::VSliderDB(normal) => {
                let value = self.db_range.unmap_to_value(normal);
                self.output_text = format!("VSliderDB: {:.3}", value);
            }
            Message::KnobFreq(normal) => {
                let value = self.freq_range.unmap_to_value(normal);
                self.output_text = format!("KnobFreq: {:.2}", value);
            }
            Message::XYPadFloat(normal_x, normal_y) => {
                let value_x = self.float_range.unmap_to_value(normal_x);
                let value_y = self.float_range.unmap_to_value(normal_y);
                self.output_text =
                    format!("XYPadFloat: x: {:.2}, y: {:.2}", value_x, value_y);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let grid_widget = Grid::new(&mut self.grid_state, Message::HSliderInt)

        let content: Element<_> = Column::new()
            .align_items(Align::Center)
            .push(grid_widget)
            .into();

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
