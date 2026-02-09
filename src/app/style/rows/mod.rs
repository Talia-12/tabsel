use crate::app::style::Scale;
use crate::config::color::OnagreColor;
use crate::config::padding::OnagrePadding;
use generic::GenericContainerStyle;
use iced::alignment::{Horizontal, Vertical};
use iced::Length;
use iced_core::border::Radius;
use iced_core::{Background, Border};
use iced_style::container::{Appearance, StyleSheet};

pub mod button;
pub mod generic;

#[derive(Debug, PartialEq, Clone)]
pub struct RowStyles {
    // Layout
    pub padding: OnagrePadding,
    pub width: Length,
    pub height: Length,
    pub spacing: u16,
    pub align_x: Horizontal,
    pub align_y: Vertical,

    // Style
    pub background: OnagreColor,
    pub border_radius: f32,
    pub border_width: f32,
    pub color: OnagreColor,
    pub border_color: OnagreColor,
    pub hide_description: bool,

    // Children
    pub title: GenericContainerStyle,
    pub description: GenericContainerStyle,
}

impl Scale for RowStyles {
    fn scale(mut self, scale: f32) -> Self {
        self.height = self.height.scale(scale);
        self.width = self.width.scale(scale);
        self.spacing = self.spacing.scale(scale);
        self.border_width = self.border_width.scale(scale);
        self.title = self.title.scale(scale);
        self.description = self.description.scale(scale);
        self
    }
}
impl StyleSheet for &RowStyles {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            text_color: Some(self.color.into()),
            background: Some(Background::Color(self.background.into())),
            border: Border {
                color: self.border_color.into(),
                width: self.border_width,
                radius: Radius::from(self.border_radius),
            },
            shadow: Default::default(),
        }
    }
}

impl Default for RowStyles {
    fn default() -> Self {
        RowStyles {
            width: Length::Fill,
            height: Length::Shrink,
            background: OnagreColor::DEFAULT_BACKGROUND,
            color: OnagreColor::DEFAULT_TEXT,
            border_radius: 0.0,
            border_width: 0.0,
            padding: OnagrePadding::from(5),
            align_x: Horizontal::Right,
            align_y: Vertical::Bottom,
            border_color: OnagreColor::RED,
            hide_description: false,
            title: GenericContainerStyle::default(),
            description: GenericContainerStyle::description_default(),
            spacing: 2,
        }
    }
}

impl RowStyles {
    pub fn default_selected() -> Self {
        Self {
            color: OnagreColor::WHITE,
            title: GenericContainerStyle {
                color: OnagreColor::WHITE,
                ..Default::default()
            },
            description: GenericContainerStyle {
                color: OnagreColor::WHITE,
                ..GenericContainerStyle::description_default()
            },
            ..Default::default()
        }
    }
}
impl Eq for RowStyles {}

#[derive(Debug, PartialEq, Clone)]
pub struct HeaderRowStyle {
    // Layout
    pub padding: OnagrePadding,
    pub width: Length,
    pub height: Length,
    pub spacing: u16,
    pub align_x: Horizontal,
    pub align_y: Vertical,

    // Style
    pub background: OnagreColor,
    pub border_radius: f32,
    pub border_width: f32,
    pub color: OnagreColor,
    pub border_color: OnagreColor,
    pub font_size: u16,
    pub separator_color: OnagreColor,
    pub separator_width: f32,
}

impl Scale for HeaderRowStyle {
    fn scale(mut self, scale: f32) -> Self {
        self.height = self.height.scale(scale);
        self.width = self.width.scale(scale);
        self.spacing = self.spacing.scale(scale);
        self.border_width = self.border_width.scale(scale);
        self.font_size = self.font_size.scale(scale);
        self.separator_width = self.separator_width.scale(scale);
        self.padding = self.padding.scale(scale);
        self
    }
}

impl StyleSheet for &HeaderRowStyle {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            text_color: Some(self.color.into()),
            background: Some(Background::Color(self.background.into())),
            border: Border {
                color: self.border_color.into(),
                width: self.border_width,
                radius: Radius::from(self.border_radius),
            },
            shadow: Default::default(),
        }
    }
}

impl Default for HeaderRowStyle {
    fn default() -> Self {
        HeaderRowStyle {
            width: Length::Fill,
            height: Length::Shrink,
            background: OnagreColor::DEFAULT_BACKGROUND,
            color: OnagreColor::WHITE,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: OnagreColor::TRANSPARENT,
            padding: OnagrePadding::from(5),
            align_x: Horizontal::Left,
            align_y: Vertical::Center,
            spacing: 2,
            font_size: 14,
            separator_color: OnagreColor::DEFAULT_BORDER,
            separator_width: 1.0,
        }
    }
}

impl Eq for HeaderRowStyle {}
