use std::process::exit;

use iced::widget::{column, container, scrollable, Column, Container};
use iced::{event, window, Application, Command, Element, Length, Settings, Subscription};
use iced_core::keyboard::key::Named;
use iced_core::keyboard::Key;
use iced_core::widget::operation::scrollable::RelativeOffset;
use iced_core::window::settings::PlatformSpecific;
use iced_core::{Event, Font, Pixels, Size};
use iced_style::Theme;
use once_cell::sync::Lazy;
use tracing::debug;

use crate::data::Table;
use crate::THEME;

pub mod entries;
pub mod state;
pub mod style;

pub fn run(table: Table) -> iced::Result {
    debug!("Starting Tabsel in debug mode");

    let default_font = THEME
        .font
        .as_ref()
        .map(|font| Font::with_name(font))
        .unwrap_or_default();

    Tabsel::run(Settings {
        id: Some("tabsel".to_string()),
        window: window::Settings {
            transparent: true,
            size: Size {
                width: THEME.size.0 as f32,
                height: THEME.size.1 as f32,
            },
            decorations: false,
            resizable: false,
            position: window::Position::Centered,
            min_size: None,
            max_size: None,
            icon: None,
            visible: true,
            platform_specific: PlatformSpecific {
                application_id: "tabsel".to_string(),
            },
            level: Default::default(),
            exit_on_close_request: false,
        },
        default_text_size: Pixels::from(THEME.font_size),
        antialiasing: true,
        default_font,
        flags: TabselFlags { table },
        fonts: vec![],
    })
}

#[derive(Debug)]
pub struct Tabsel {
    state: state::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loading,
    KeyboardEvent(Key),
    Unfocused,
}

static SCROLL_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub struct TabselFlags {
    pub table: Table,
}

impl Application for Tabsel {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = TabselFlags;

    fn new(flags: TabselFlags) -> (Self, Command<Self::Message>) {
        let tabsel = Tabsel {
            state: state::State {
                table: flags.table,
                ..Default::default()
            },
        };

        (
            tabsel,
            Command::perform(async {}, move |()| Message::Loading),
        )
    }

    fn title(&self) -> String {
        "Tabsel".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Loading => Command::none(),
            Message::KeyboardEvent(event) => self.handle_input(event),
            Message::Unfocused => {
                if THEME.exit_unfocused {
                    exit(0);
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Empty scrollable container (no rows yet)
        let scrollable =
            scrollable(column([]))
                .id(SCROLL_ID.clone())
                .style(iced::theme::Scrollable::Custom(Box::new(
                    THEME.scrollable(),
                )));

        let scrollable = container(scrollable)
            .style(iced::theme::Container::Custom(Box::new(
                &THEME.app_container.rows,
            )))
            .padding(THEME.app_container.rows.padding.to_iced_padding())
            .width(THEME.app_container.rows.width)
            .height(THEME.app_container.rows.height);

        let app_container = Container::new(
            Column::new()
                .push(scrollable)
                .align_items(iced_core::Alignment::Start),
        )
        .padding(THEME.app().padding.to_iced_padding())
        .style(iced::theme::Container::Custom(Box::new(THEME.app())))
        .center_y()
        .center_x();

        let app_wrapper = Container::new(app_container)
            .center_y()
            .center_x()
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(THEME.padding.to_iced_padding())
            .style(iced::theme::Container::Custom(Box::new(&*THEME)));

        app_wrapper.into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Tabsel::keyboard_event()
    }
}

impl Tabsel {
    fn handle_input(&mut self, key_code: Key) -> Command<Message> {
        match key_code {
            Key::Named(Named::Escape) => {
                exit(0);
            }
            _ => {}
        };

        Command::none()
    }

    fn snap(&mut self) -> Command<Message> {
        let selected = self.state.selected_row;
        let offset = RelativeOffset {
            x: 0.0,
            y: selected as f32,
        };
        scrollable::snap_to(SCROLL_ID.clone(), offset)
    }

    fn keyboard_event() -> Subscription<Message> {
        event::listen_with(|event, _status| match event {
            Event::Window(_, window::Event::Unfocused) => Some(Message::Unfocused),
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                modifiers: _,
                text: _,
                key,
                location: _,
            }) => Some(Message::KeyboardEvent(key)),
            _ => None,
        })
    }
}
