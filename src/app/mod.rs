use std::process::exit;

use iced::widget::{
    column, container, horizontal_rule, scrollable, text, text_input, Button, Column, Container,
    Row, TextInput,
};
use iced::{event, window, Alignment, Application, Command, Element, Length, Settings, Subscription};
use iced_core::keyboard::key::Named;
use iced_core::keyboard::{Key, Modifiers};
use iced_core::widget::operation::scrollable::RelativeOffset;
use iced_core::window::settings::PlatformSpecific;
use iced_core::{Event, Font, Pixels, Size};
use iced_style::Theme;
use once_cell::sync::Lazy;
use tracing::debug;

use crate::app::style::rows::button::ButtonStyle;
use crate::data::output;
use crate::data::{OutputFormat, SelectionMode, Table};
use crate::THEME;

pub mod entries;
pub mod state;
pub mod style;

pub fn run(
    table: Table,
    available_modes: Vec<SelectionMode>,
    filter_enabled: bool,
    output_format: OutputFormat,
    window_size: (f32, f32),
) -> iced::Result {
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
                width: window_size.0,
                height: window_size.1,
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
        flags: TabselFlags {
            table,
            available_modes,
            filter_enabled,
            output_format,
        },
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
    InputChanged(String),
    KeyboardEvent(Key, Modifiers),
    Unfocused,
}

static SCROLL_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub struct TabselFlags {
    pub table: Table,
    pub available_modes: Vec<SelectionMode>,
    pub filter_enabled: bool,
    pub output_format: OutputFormat,
}

impl Application for Tabsel {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = TabselFlags;

    fn new(flags: TabselFlags) -> (Self, Command<Self::Message>) {
        let active_mode = flags.available_modes[0];
        let mut state = state::State {
            table: flags.table,
            active_mode,
            available_modes: flags.available_modes,
            filter_enabled: flags.filter_enabled,
            output_format: flags.output_format,
            ..Default::default()
        };
        state.init_filtered_indices();

        let tabsel = Tabsel { state };

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
            Message::Loading => {
                if self.state.filter_enabled {
                    text_input::focus(INPUT_ID.clone())
                } else {
                    Command::none()
                }
            }
            Message::InputChanged(value) => {
                self.state.filter_text = value;
                self.state.update_filtered_indices();
                self.state.selected_row = 0;
                self.snap()
            }
            Message::KeyboardEvent(key, modifiers) => self.handle_input(key, modifiers),
            Message::Click(filtered_pos) => {
                self.state.selected_row = filtered_pos;
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

        let mut app_column: Vec<Element<'_, Self::Message>> = Vec::new();

        // Filter bar (if enabled)
        if self.state.filter_enabled {
            let search_style = THEME.search();
            let input_style = THEME.search_input();

            let input: TextInput<'_, Message> =
                text_input("Filter...", &self.state.filter_text)
                    .id(INPUT_ID.clone())
                    .on_input(Message::InputChanged)
                    .size(input_style.font_size)
                    .width(input_style.width)
                    .style(iced::theme::TextInput::Custom(Box::new(input_style)));

            let search_container = Container::new(input)
                .style(iced::theme::Container::Custom(Box::new(search_style)))
                .padding(search_style.padding.to_iced_padding())
                .width(search_style.width);

            app_column.push(search_container.into());
        }

        // Build rows
        let column_spacing = THEME.app_container.rows.column_spacing;
        let mut rows_column: Vec<Element<'_, Self::Message>> = Vec::new();

        // Header row (if present)
        if let Some(headers) = &self.state.table.headers {
            let header_style = &THEME.app_container.rows.header;
            let header_cells: Vec<Element<'_, Self::Message>> = headers
                .iter()
                .map(|h| {
                    Container::new(
                        text(h.as_str()).size(header_style.font_size),
                    )
                    .width(Length::FillPortion(1))
                    .into()
                })
                .collect();

            let header_row = Container::new(
                Row::with_children(header_cells)
                    .width(Length::Fill)
                    .spacing(column_spacing),
            )
            .style(iced::theme::Container::Custom(Box::new(header_style)))
            .padding(header_style.padding.to_iced_padding())
            .width(header_style.width);

            rows_column.push(header_row.into());

            // Separator line between header and data
            if header_style.separator_width > 0.0 {
                rows_column.push(horizontal_rule(header_style.separator_width as u16).into());
            }
        }

        // Data rows (filtered)
        for (filtered_pos, &actual_idx) in self.state.filtered_indices.iter().enumerate() {
            let row_data = &self.state.table.rows[actual_idx];
            let cells: Vec<Element<'_, Self::Message>> = (0..num_cols)
                .map(|col| {
                    let selected = self.state.cell_is_selected(filtered_pos, col);
                    let cell_style = if selected {
                        &THEME.app_container.rows.row_selected
                    } else {
                        &THEME.app_container.rows.row
                    };

                    let cell_text = row_data.get(col).map(|s| s.as_str()).unwrap_or("");
                    Container::new(
                        text(cell_text).size(cell_style.title.font_size),
                    )
                    .style(iced::theme::Container::Custom(Box::new(&cell_style.title)))
                    .padding(cell_style.title.padding.to_iced_padding())
                    .width(Length::FillPortion(1))
                    .into()
                })
                .collect();

            // Row container uses selected style if any cell in the row is selected
            let row_has_selection =
                (0..num_cols).any(|c| self.state.cell_is_selected(filtered_pos, c));
            let row_style = if row_has_selection {
                &THEME.app_container.rows.row_selected
            } else {
                &THEME.app_container.rows.row
            };

            let row_content = Row::with_children(cells)
                .width(Length::Fill)
                .spacing(column_spacing)
                .align_items(Alignment::Start);

            let button = Button::new(row_content)
                .style(iced::theme::Button::Custom(Box::new(&ButtonStyle)))
                .on_press(Message::Click(filtered_pos));

            let row_container = Container::new(button)
                .style(iced::theme::Container::Custom(Box::new(row_style)))
                .padding(row_style.padding.to_iced_padding())
                .width(row_style.width);

            rows_column.push(row_container.into());
        }

        // Scrollable containing all rows
        let scrollable = scrollable(column(rows_column))
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

        app_column.push(scrollable.into());

        let app_container = Container::new(
            Column::with_children(app_column).align_items(Alignment::Start),
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
    fn handle_input(&mut self, key_code: Key, modifiers: Modifiers) -> Command<Message> {
        // Shift+Tab cycles selection mode
        if key_code == Key::Named(Named::Tab) && modifiers.shift() {
            self.state.cycle_mode();
            return Command::none();
        }

        match key_code {
            Key::Named(Named::ArrowUp) => {
                match self.state.active_mode {
                    SelectionMode::Row | SelectionMode::Cell => return self.dec_selected_row(),
                    SelectionMode::Column => {}
                }
            }
            Key::Named(Named::ArrowDown) => {
                match self.state.active_mode {
                    SelectionMode::Row | SelectionMode::Cell => return self.inc_selected_row(),
                    SelectionMode::Column => {}
                }
            }
            Key::Named(Named::ArrowLeft) => {
                match self.state.active_mode {
                    SelectionMode::Column | SelectionMode::Cell => return self.dec_selected_col(),
                    SelectionMode::Row => {}
                }
            }
            Key::Named(Named::ArrowRight) => {
                match self.state.active_mode {
                    SelectionMode::Column | SelectionMode::Cell => return self.inc_selected_col(),
                    SelectionMode::Row => {}
                }
            }
            Key::Named(Named::Enter) => return self.on_confirm(),
            Key::Named(Named::Escape) => {
                exit(1);
            }
            _ => {}
        };

        Command::none()
    }

    fn on_confirm(&self) -> Command<Message> {
        let fmt = self.state.output_format;
        let table = &self.state.table;

        if self.state.visible_rows() == 0 {
            exit(1);
        }

        let result = match self.state.active_mode {
            SelectionMode::Row => {
                let actual_idx = self.state.actual_row_index(self.state.selected_row);
                output::format_row(table, fmt, actual_idx)
            }
            SelectionMode::Column => {
                output::format_column(table, fmt, self.state.selected_col)
            }
            SelectionMode::Cell => {
                let actual_idx = self.state.actual_row_index(self.state.selected_row);
                output::format_cell(table, fmt, actual_idx, self.state.selected_col)
            }
        };

        println!("{result}");
        exit(0);
    }

    fn inc_selected_row(&mut self) -> Command<Message> {
        let total = self.state.visible_rows();
        if total > 0 && self.state.selected_row < total - 1 {
            self.state.selected_row += 1;
        }
        self.snap()
    }

    fn dec_selected_row(&mut self) -> Command<Message> {
        if self.state.selected_row > 0 {
            self.state.selected_row -= 1;
        }
        self.snap()
    }

    fn inc_selected_col(&mut self) -> Command<Message> {
        let num_cols = self.state.num_columns();
        if num_cols > 0 && self.state.selected_col < num_cols - 1 {
            self.state.selected_col += 1;
        }
        Command::none()
    }

    fn dec_selected_col(&mut self) -> Command<Message> {
        if self.state.selected_col > 0 {
            self.state.selected_col -= 1;
        }
        Command::none()
    }

    fn snap(&self) -> Command<Message> {
        let total = self.state.visible_rows();
        if total <= 1 {
            return scrollable::snap_to(SCROLL_ID.clone(), RelativeOffset::START);
        }
        let offset = self.state.selected_row as f32 / (total - 1) as f32;
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
                modifiers,
                text: _,
                key,
                location: _,
            }) => Some(Message::KeyboardEvent(key, modifiers)),
            _ => None,
        })
    }
}
