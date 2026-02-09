use std::process::exit;

use iced::widget::{column, container, scrollable, text, Button, Column, Container, Row};
use iced::{event, window, Alignment, Application, Command, Element, Length, Settings, Subscription};
use iced_core::keyboard::key::Named;
use iced_core::keyboard::Key;
use iced_core::widget::operation::scrollable::RelativeOffset;
use iced_core::window::settings::PlatformSpecific;
use iced_core::{Event, Font, Pixels, Size};
use iced_style::Theme;
use once_cell::sync::Lazy;
use tracing::debug;

use crate::app::style::rows::button::ButtonStyle;
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
    Click(usize),
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
            Message::Click(row_idx) => {
                self.state.selected_row = row_idx;
                self.on_confirm()
            }
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
        let num_cols = self.state.num_columns();

        // Build rows
        let mut rows_column: Vec<Element<'_, Self::Message>> = Vec::new();

        // Header row (if present)
        if let Some(headers) = &self.state.table.headers {
            let header_style = &THEME.app_container.rows.row_selected;
            let header_cells: Vec<Element<'_, Self::Message>> = headers
                .iter()
                .map(|h| {
                    Container::new(
                        text(h.as_str())
                            .size(header_style.title.font_size),
                    )
                    .style(iced::theme::Container::Custom(Box::new(
                        &header_style.title,
                    )))
                    .padding(header_style.title.padding.to_iced_padding())
                    .width(Length::FillPortion(1))
                    .into()
                })
                .collect();

            let header_row = Container::new(Row::with_children(header_cells).width(Length::Fill))
                .style(iced::theme::Container::Custom(Box::new(header_style)))
                .padding(header_style.padding.to_iced_padding())
                .width(header_style.width);

            rows_column.push(header_row.into());
        }

        // Data rows
        for (idx, row_data) in self.state.table.rows.iter().enumerate() {
            let is_selected = idx == self.state.selected_row;
            let row_style = if is_selected {
                &THEME.app_container.rows.row_selected
            } else {
                &THEME.app_container.rows.row
            };

            let cells: Vec<Element<'_, Self::Message>> = (0..num_cols)
                .map(|col| {
                    let cell_text = row_data.get(col).map(|s| s.as_str()).unwrap_or("");
                    Container::new(
                        text(cell_text).size(row_style.title.font_size),
                    )
                    .style(iced::theme::Container::Custom(Box::new(&row_style.title)))
                    .padding(row_style.title.padding.to_iced_padding())
                    .width(Length::FillPortion(1))
                    .into()
                })
                .collect();

            let row_content = Row::with_children(cells)
                .width(Length::Fill)
                .spacing(row_style.spacing)
                .align_items(Alignment::Start);

            let button = Button::new(row_content)
                .style(iced::theme::Button::Custom(Box::new(&ButtonStyle)))
                .on_press(Message::Click(idx));

            let row_container = Container::new(button)
                .style(iced::theme::Container::Custom(Box::new(row_style)))
                .padding(row_style.padding.to_iced_padding())
                .width(row_style.width);

            rows_column.push(row_container.into());
        }

        // Scrollable containing all rows
        let scrollable =
            scrollable(column(rows_column))
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
                .align_items(Alignment::Start),
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
            Key::Named(Named::ArrowUp) => return self.dec_selected(),
            Key::Named(Named::ArrowDown) => return self.inc_selected(),
            Key::Named(Named::Enter) => return self.on_confirm(),
            Key::Named(Named::Escape) => {
                exit(0);
            }
            _ => {}
        };

        Command::none()
    }

    fn on_confirm(&self) -> Command<Message> {
        // Stub: will output selection in Step 7
        exit(0);
    }

    fn inc_selected(&mut self) -> Command<Message> {
        let total = self.state.visible_rows();
        if total > 0 && self.state.selected_row < total - 1 {
            self.state.selected_row += 1;
        }
        self.snap()
    }

    fn dec_selected(&mut self) -> Command<Message> {
        if self.state.selected_row > 0 {
            self.state.selected_row -= 1;
        }
        self.snap()
    }

    fn snap(&self) -> Command<Message> {
        let total = self.state.visible_rows() as f32;
        if total == 0.0 {
            return scrollable::snap_to(SCROLL_ID.clone(), RelativeOffset::START);
        }
        let offset = self.state.selected_row as f32 / total;
        scrollable::snap_to(
            SCROLL_ID.clone(),
            RelativeOffset {
                x: 0.0,
                y: offset,
            },
        )
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
