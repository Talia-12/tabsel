use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::anyhow;
use clap::Parser;
use once_cell::sync::{Lazy, OnceCell};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use app::style::Theme;

pub mod app;
pub mod config;

pub static THEME_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(|| {
    Mutex::new(
        dirs::config_dir()
            .ok_or_else(|| anyhow!("Theme config not found"))
            .map(|path| path.join("tabsel").join("theme.scss"))
            .unwrap(),
    )
});

static THEME_SCALE: OnceCell<f32> = OnceCell::new();

pub static THEME: Lazy<Theme> = Lazy::new(Theme::load);

#[derive(Parser)]
#[command(name = "tabsel")]
struct Cli {
    #[arg(
        long = "theme",
        short = 't',
        help = "Path to an alternate tabsel theme file"
    )]
    theme: Option<PathBuf>,

    #[arg(long = "scale", short = 's', help = "Change the scale of tabsel theme")]
    scale: Option<f32>,
}

pub fn main() -> iced::Result {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "tabsel=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting tabsel");
    let cli = Cli::parse();

    if let Some(theme_path) = cli.theme {
        let path = theme_path.canonicalize();
        if let Ok(path) = path {
            *THEME_PATH.lock().unwrap() = path;
        }

        info!("Using alternate theme : {:?}", THEME_PATH.lock().unwrap());
    }

    if let Some(scale) = cli.scale {
        THEME_SCALE.get_or_init(|| scale);
        info!("Using scale value : {:?}", scale);
    }

    app::run()
}
