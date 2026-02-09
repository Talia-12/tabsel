use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::anyhow;
use clap::Parser;
use once_cell::sync::{Lazy, OnceCell};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use app::style::Theme;
use data::{InputFormat, OutputFormat, SelectionMode};

pub mod app;
pub mod config;
pub mod data;

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

    #[arg(
        long = "format",
        short = 'f',
        default_value = "csv",
        help = "Input format: csv or json"
    )]
    format: String,

    #[arg(
        long = "header",
        default_value = "true",
        help = "Whether the CSV input has a header row"
    )]
    header: bool,

    #[arg(
        long = "mode",
        short = 'm',
        default_value = "row",
        help = "Selection mode(s): row, column, cell. Repeat for multiple (e.g. --mode row --mode cell)"
    )]
    mode: Vec<String>,

    #[arg(
        long = "no-filter",
        default_value = "false",
        help = "Disable the filter bar"
    )]
    no_filter: bool,

    #[arg(
        long = "output-format",
        short = 'o',
        default_value = "plain",
        help = "Output format: plain, json, or csv"
    )]
    output_format: String,
}

pub fn main() -> iced::Result {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "tabsel=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
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

    let format = match cli.format.as_str() {
        "json" => InputFormat::Json,
        _ => InputFormat::Csv,
    };

    let table = data::parse::parse_stdin(format, cli.header).unwrap_or_else(|err| {
        eprintln!("Error parsing input: {err}");
        std::process::exit(1);
    });

    info!(
        "Parsed table: {} rows, {} columns",
        table.rows.len(),
        table.headers.as_ref().map_or_else(
            || table.rows.first().map_or(0, |r| r.len()),
            |h| h.len()
        )
    );

    let available_modes: Vec<SelectionMode> = cli
        .mode
        .iter()
        .map(|m| match m.as_str() {
            "row" => SelectionMode::Row,
            "column" => SelectionMode::Column,
            "cell" => SelectionMode::Cell,
            other => {
                eprintln!("Unknown mode: {other}. Valid modes: row, column, cell");
                std::process::exit(1);
            }
        })
        .collect();

    let output_format = match cli.output_format.as_str() {
        "json" => OutputFormat::Json,
        "csv" => OutputFormat::Csv,
        "plain" => OutputFormat::Plain,
        other => {
            eprintln!("Unknown output format: {other}. Valid formats: plain, json, csv");
            std::process::exit(1);
        }
    };

    app::run(table, available_modes, !cli.no_filter, output_format)
}
