use crate::app::style::app::AppContainerStyles;
use crate::app::style::scrollable::scroller::ScrollerStyles;
use crate::app::style::search::input::SearchInputStyles;
use crate::app::style::search::SearchContainerStyles;
use crate::config::color::OnagreColor;
use crate::config::padding::OnagrePadding;
use crate::THEME_PATH;
use crate::THEME_SCALE;
use iced::widget::container::Appearance;
use iced::Background;
use iced_core::border::Radius;
use iced_core::{Border, Length};
use tracing::{error, warn};

pub mod app;
pub mod rows;
pub mod scrollable;
pub mod search;

impl Theme {
    pub fn load() -> Self {
        let buf = THEME_PATH.lock().unwrap().clone();
        let theme = crate::config::parse_file(&buf);
        if let Err(err) = &theme {
            error!("Failed to parse theme {buf:?}: {err}");
            warn!("Failing back to default theme");
        };

        let mut theme = theme.unwrap_or_default();
        if let Some(scale) = THEME_SCALE.get() {
            theme = theme.scale(*scale)
        }

        theme
    }
}

pub(crate) trait Scale {
    fn scale(self, scale: f32) -> Self;
}

impl AsRef<Theme> for Theme {
    fn as_ref(&self) -> &Theme {
        self
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SizeUnit {
    Px,
    Percent,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SizeSpec {
    pub value: f32,
    pub unit: SizeUnit,
}

impl SizeSpec {
    pub fn px(value: f32) -> Self {
        SizeSpec {
            value,
            unit: SizeUnit::Px,
        }
    }

    pub fn percent(value: f32) -> Self {
        SizeSpec {
            value,
            unit: SizeUnit::Percent,
        }
    }

    /// Resolve to pixels given the corresponding screen dimension.
    pub fn resolve(&self, screen_dim: f32) -> f32 {
        match self.unit {
            SizeUnit::Px => self.value,
            SizeUnit::Percent => self.value / 100.0 * screen_dim,
        }
    }
}

impl Scale for SizeSpec {
    fn scale(mut self, scale: f32) -> Self {
        if self.unit == SizeUnit::Px {
            self.value *= scale;
        }
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct Theme {
    // Layout
    pub exit_unfocused: bool,
    pub min_width: SizeSpec,
    pub max_width: SizeSpec,
    pub min_height: SizeSpec,
    pub max_height: SizeSpec,
    pub font: Option<String>,
    pub font_size: u16,
    pub padding: OnagrePadding,

    // Style
    pub background: OnagreColor,
    pub color: OnagreColor,
    pub border_color: OnagreColor,
    pub border_radius: f32,
    pub border_width: f32,

    // Children
    pub app_container: AppContainerStyles,
}

impl Scale for Theme {
    fn scale(mut self, scale: f32) -> Self {
        self.app_container = self.app_container.scale(scale);
        self.min_width = self.min_width.scale(scale);
        self.max_width = self.max_width.scale(scale);
        self.min_height = self.min_height.scale(scale);
        self.max_height = self.max_height.scale(scale);
        self.padding = self.padding * scale;
        self.font_size = (self.font_size as f32 * scale) as u16;
        self
    }
}

impl Scale for Length {
    fn scale(self, scale: f32) -> Self {
        match self {
            Length::Fixed(size) => Length::Fixed(size * scale),
            _ => self,
        }
    }
}

impl Scale for u16 {
    fn scale(self, scale: f32) -> Self {
        (self as f32 * scale) as u16
    }
}

impl Scale for f32 {
    fn scale(self, scale: f32) -> Self {
        self * scale
    }
}

impl Theme {
    pub fn search(&self) -> &SearchContainerStyles {
        &self.app_container.search
    }

    pub fn search_input(&self) -> &SearchInputStyles {
        &self.app_container.search.input
    }

    pub fn scrollable(&self) -> &ScrollerStyles {
        &self.app_container.scrollable
    }

    pub fn app(&self) -> &AppContainerStyles {
        &self.app_container
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            exit_unfocused: false,
            min_width: SizeSpec::px(200.0),
            max_width: SizeSpec::percent(80.0),
            min_height: SizeSpec::px(150.0),
            max_height: SizeSpec::percent(70.0),
            font: None,
            font_size: 18,
            background: OnagreColor::DEFAULT_BACKGROUND,
            color: OnagreColor::DEFAULT_TEXT,
            border_color: OnagreColor::TRANSPARENT,
            border_radius: 0.0,
            border_width: 0.0,
            padding: OnagrePadding::ZERO,
            app_container: AppContainerStyles::default(),
        }
    }
}

impl iced::widget::container::StyleSheet for &Theme {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: Some(Background::Color(self.background.into())),
            border: Border {
                color: self.border_color.into(),
                width: self.border_width,
                radius: Radius::from(self.border_radius),
            },
            text_color: Some(self.color.into()),
            shadow: Default::default(),
        }
    }
}
