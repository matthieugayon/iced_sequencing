use std::collections::HashMap;

// Import iced modules.
use iced::{
    Color, Column, Container, scrollable, Scrollable,
    Element, Length, Sandbox, Settings, Text,
    container, Alignment, TextInput, text_input, Row
};

use iced_sequencing::grid::{self, GridColor};
use iced_sequencing::style::grid::{
    Style, StyleSheet, 
    Grid, Event, Stroke
};
use iced_sequencing::style::color_utils::*;
use iced_sequencing::core::grid::{
    GridPattern,
    GridMessage,
    manage_state_update
};


// orange = ff7e53
// green = 48fce4

pub struct MainContainerStyle;

impl container::StyleSheet for MainContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: hex("1A2122").into(),
            ..container::Style::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GridEvent(GridMessage),
    SetColor(ColorTarget, String)
}


pub fn main() {
    App::run(Settings {
        antialiasing: true,
        ..Settings::default()
    }).unwrap();
}

pub struct App {
    scroll: scrollable::State,
    hex_data: HashMap<ColorTarget, Input>,
    style_sheet: CustomStyleSheet,
    grid_state: grid::State,
    live_pattern: GridPattern,
    focused_track: usize
}

impl<'a> Sandbox for App {
    type Message = Message;

    fn new() -> App  {
        let initial_pattern = GridPattern::new();
        let mut h = HashMap::new();

        ColorTarget::all().iter().cloned().for_each(|c| {
            h.insert(c, Input::new(c));
        });

        App {
            scroll: scrollable::State::new(),
            hex_data: h,
            style_sheet: CustomStyleSheet {
                basic: Style {
                    event: Event::default(),
                    grid: Grid::default(),
                    background: None,
                    selection_stroke: Stroke { color: hex("#8ea5a8"), line_width: 0.7 },
                    selected_track_bg_color: lighten(Color::BLACK, 0.7),
                    current_step_bg_color: lighten(hex("374140"), 0.1)
                }
            },
            grid_state: grid::State::new(initial_pattern.clone()),
            live_pattern: initial_pattern,
            focused_track: 0
        }
    }

    fn title(&self) -> String {
        format!("Grid designer - Iced Sequencing")
    }

    fn update(&mut self, event: Message) {
        // println!("--- update {:?}", event);

        match event {
            Message::GridEvent(grid_message) => {
                manage_state_update(grid_message, &mut self.grid_state, &mut self.live_pattern, &mut self.focused_track);
            },
            Message::SetColor(color_target, color) => {
                println!("{:?}", color_target);
                match from_hex(color.as_str()) {
                    Some(color_tuple) => {
                        let iced_color = Color::from_rgb(
                            color_tuple.0 / 255.,
                            color_tuple.1 / 255.,
                            color_tuple.2 / 255.,
                        );

                        match color_target {
                            ColorTarget::EventBg => {
                                println!("set color to EventBg in stylesheet");

                                self.style_sheet.basic.event.bg_color = GridColor::Simple(iced_color);
                            },
                            ColorTarget::EventContourBg => {},
                            ColorTarget::EventStroke => {},
                            ColorTarget::EventSliderBg => {
                                println!("set color to EventSliderBg in stylesheet");

                                self.style_sheet.basic.event.slider_bg_color = GridColor::Simple(iced_color);
                            },
                            ColorTarget::EventSliderHighlightedBg => {},
                            ColorTarget::EventNegativeOffsetMarkerBg => {},
                            ColorTarget::EventPositiveOffsetMarkerBg => {},
                            ColorTarget::GridEvenBeatBg => {},
                            ColorTarget::GridOddBeatBg => {},
                            ColorTarget::GridEdgeBeatBg => {},
                            ColorTarget::GridEvenBeatLine => {},
                            ColorTarget::GridOddBeatLine => {},
                            ColorTarget::GridEdgeBeatLine => {},
                            ColorTarget::GridTrackMargin => {},
                            ColorTarget::Background => {},
                            ColorTarget::SelectionStroke => {},
                            ColorTarget::SelectedTrackBg => {},
                            ColorTarget::CurrentStepBg => {},
                        }
                    },
                    None => {},
                }

                let input = self.hex_data.get_mut(&color_target).unwrap();
                input.update(color);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let grid = grid::Grid::new(
                &mut self.grid_state, 
                self.live_pattern.clone(),
                Message::GridEvent,
                Length::from(Length::Units(690)),
                Length::from(Length::Units(345))
            )
            .style(self.style_sheet.clone());

        let inputs = self.hex_data.iter_mut().fold(
            Column::new(),
            |column, (_target, input)| {
                // let (value, state) = self.hex_data.get_mut(&target);
                column
                    .push(input.view())
            }
        );

        let content = Column::new()
            .align_items(Alignment::Center)
            .push(grid)
            .push(inputs)
            .width(Length::Units(690));

        let scrollable = Scrollable::new(&mut self.scroll)
            .push(Container::new(content).width(Length::Fill).center_x());

        Container::new(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(MainContainerStyle {})
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorTarget { 
    EventBg,
    EventContourBg,
    EventStroke,
    EventSliderBg,
    EventSliderHighlightedBg,
    EventNegativeOffsetMarkerBg,
    EventPositiveOffsetMarkerBg,

    GridEvenBeatBg,
    GridOddBeatBg,
    GridEdgeBeatBg,
    GridEvenBeatLine,
    GridOddBeatLine,
    GridEdgeBeatLine,
    GridTrackMargin,

    Background,
    SelectionStroke,
    SelectedTrackBg,
    CurrentStepBg
}

impl ColorTarget {
    fn all() -> [ColorTarget; 18] {
        [
            ColorTarget::EventBg,
            ColorTarget::EventContourBg,
            ColorTarget::EventStroke,
            ColorTarget::EventSliderBg,
            ColorTarget::EventSliderHighlightedBg,
            ColorTarget::EventNegativeOffsetMarkerBg,
            ColorTarget::EventPositiveOffsetMarkerBg,
            ColorTarget::GridEvenBeatBg,
            ColorTarget::GridOddBeatBg,
            ColorTarget::GridEdgeBeatBg,
            ColorTarget::GridEvenBeatLine,
            ColorTarget::GridOddBeatLine,
            ColorTarget::GridEdgeBeatLine,
            ColorTarget::GridTrackMargin,
            ColorTarget::Background,
            ColorTarget::SelectionStroke,
            ColorTarget::SelectedTrackBg,
            ColorTarget::CurrentStepBg
        ]
    }
}

impl From<ColorTarget> for String {
    fn from(color_target: ColorTarget) -> String {
        String::from(match color_target {
            ColorTarget::EventBg => "EventBg",
            ColorTarget::EventContourBg => "EventContourBg",
            ColorTarget::EventStroke => "EventStroke",
            ColorTarget::EventSliderBg => "EventSliderBg",
            ColorTarget::EventSliderHighlightedBg => "EventSliderHighlightedBg",
            ColorTarget::EventNegativeOffsetMarkerBg => "EventNegativeOffsetMarkerBg",
            ColorTarget::EventPositiveOffsetMarkerBg => "EventPositiveOffsetMarkerBg",
            ColorTarget::GridEvenBeatBg => "GridEvenBeatBg",
            ColorTarget::GridOddBeatBg => "GridOddBeatBg",
            ColorTarget::GridEdgeBeatBg => "GridEdgeBeatBg",
            ColorTarget::GridEvenBeatLine => "GridEvenBeatLine",
            ColorTarget::GridOddBeatLine => "GridOddBeatLine",
            ColorTarget::GridEdgeBeatLine => "GridEdgeBeatLine",
            ColorTarget::GridTrackMargin => "GridTrackMargin",
            ColorTarget::Background => "Background",
            ColorTarget::SelectionStroke => "SelectionStroke",
            ColorTarget::SelectedTrackBg => "SelectedTrackBg",
            ColorTarget::CurrentStepBg => "CurrentStepBg"
        })
    }
}

#[derive(Debug, Clone)]
struct CustomStyleSheet {
    pub basic: Style
}

impl StyleSheet for CustomStyleSheet {
    fn default(&self) -> Style {
        Style { 
            event: self.basic.event.clone(), 
            grid: self.basic.grid.clone(), 
            background: self.basic.background, 
            selection_stroke: self.basic.selection_stroke, 
            selected_track_bg_color: self.basic.selected_track_bg_color, 
            current_step_bg_color: self.basic.current_step_bg_color
        }
    }

    fn dragging_selection(&self) -> Style {
        self.default()
    }
}

#[derive(Debug)]
struct Input {
    target: ColorTarget,
    input: text_input::State,
    value: String
}

impl Input {
    pub fn new(target: ColorTarget) -> Self {
        Self {
            target,
            input: text_input::State::new(),
            value: String::from("")
        }
    }

    pub fn update(&mut self, color: String) {
        self.value = color;
    }

    pub fn view(&mut self) -> Element<Message> {
        let target = self.target.clone();

        Row::new()
            .padding(2)
            .spacing(10)
            .push(
                TextInput::new(
                    &mut self.input,
                    "Hex color",
                    &mut self.value,
                    move |val| Message::SetColor(target, val),
                )
                .size(14)
                .padding(2)
                .width(Length::Units(100)) ,
            )
            .push(Text::new(String::from(self.target)).size(14))
            .align_items(Alignment::Center)
            .into()
    }   
}
