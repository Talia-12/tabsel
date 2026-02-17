use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::anyhow;
use clap::Parser;
use once_cell::sync::{Lazy, OnceCell};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use app::style::Theme;
use data::{InputFormat, OutputFormat, SelectionMode, Table};
use iced_core::Length;

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
        long = "input-format",
        short = 'i',
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
        long = "hidden-column",
        short = 'H',
        help = "Column(s) to hide from display but include in output. Use header names with --header, or 0-based column numbers without. Repeatable."
    )]
    hidden_column: Vec<String>,

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
    // Reset SIGPIPE to default so writing to a broken pipe exits cleanly
    // instead of panicking.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

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

    let input_format = match cli.format.as_str() {
        "json" => InputFormat::Json,
        _ => InputFormat::Csv,
    };

    let table = data::parse::parse_stdin(input_format, cli.header).unwrap_or_else(|err| {
        eprintln!("Error parsing input: {err}");
        std::process::exit(1);
    });

    if table.rows.is_empty() {
        eprintln!("No data rows to display");
        std::process::exit(1);
    }

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

    // Resolve hidden columns to actual column indices
    let num_cols = table
        .headers
        .as_ref()
        .map_or_else(|| table.rows.first().map_or(0, |r| r.len()), |h| h.len());

    let hidden_columns: Vec<usize> = cli
        .hidden_column
        .iter()
        .map(|spec| {
            if let Some(headers) = &table.headers {
                headers
                    .iter()
                    .position(|h| h == spec)
                    .unwrap_or_else(|| {
                        eprintln!("Unknown header name: {spec}. Available headers: {}", headers.join(", "));
                        std::process::exit(1);
                    })
            } else {
                spec.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("Invalid column number: {spec}. Must be a 0-based integer when --header is false");
                    std::process::exit(1);
                })
            }
        })
        .collect();

    for &col in &hidden_columns {
        if col >= num_cols {
            eprintln!("Column index {col} is out of range (table has {num_cols} columns)");
            std::process::exit(1);
        }
    }

    let filter_enabled = !cli.no_filter;

    // Query screen dimensions for resolving percentage-based sizes
    let screen_size = get_screen_size();
    info!("Screen size: {:?}", screen_size);

    // Resolve min/max bounds to pixels
    let min_w = THEME.min_width.resolve(screen_size.0);
    let max_w = THEME.max_width.resolve(screen_size.0);
    let min_h = THEME.min_height.resolve(screen_size.1);
    let max_h = THEME.max_height.resolve(screen_size.1);

    // Calculate content-preferred size
    let (content_w, content_h) = calculate_content_size(&table, filter_enabled, &hidden_columns);
    info!(
        "Content size: ({}, {}), bounds: w=[{}, {}], h=[{}, {}]",
        content_w, content_h, min_w, max_w, min_h, max_h
    );

    // Clamp to bounds
    let width = content_w.max(min_w).min(max_w);
    let height = content_h.max(min_h).min(max_h);
    info!("Resolved window size: ({}, {})", width, height);

    app::run(
        table,
        available_modes,
        filter_enabled,
        output_format,
        hidden_columns,
        (width, height),
    )
}

fn get_screen_size() -> (f32, f32) {
    // Parse xrandr output to find the current screen resolution.
    // Falls back to 1920x1080 if xrandr is unavailable or parsing fails.
    std::process::Command::new("xrandr")
        .arg("--query")
        .output()
        .ok()
        .and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Active mode lines contain '*', e.g. "   1920x1080     60.00*+"
                if line.contains('*') {
                    let resolution = line.trim().split_whitespace().next()?;
                    let mut dims = resolution.split('x');
                    let w = dims.next()?.parse::<f32>().ok()?;
                    let h = dims.next()?.parse::<f32>().ok()?;
                    return Some((w, h));
                }
            }
            None
        })
        .unwrap_or((1920.0, 1080.0))
}

fn calculate_content_size(table: &Table, filter_enabled: bool, hidden_columns: &[usize]) -> (f32, f32) {
    let theme = &*THEME;
    let font_size = theme.font_size as f32;
    let char_width_estimate = font_size * 0.6;

    let total_cols = table
        .headers
        .as_ref()
        .map_or_else(|| table.rows.first().map_or(0, |r| r.len()), |h| h.len());
    let visible_cols: Vec<usize> = (0..total_cols)
        .filter(|c| !hidden_columns.contains(c))
        .collect();
    let num_cols = visible_cols.len();
    let num_rows = table.rows.len();

    // Row height estimate:
    // Each data row is: Container(Button(Row(cells)))
    //   - Container padding (row_style.padding)
    //   - Button default internal padding (5px each side in iced)
    //   - Cell Container padding (title.padding)
    //   - Text height (title.font_size)
    let button_padding_v: f32 = 5.0 + 5.0; // iced Button default padding top + bottom
    let row_padding = theme.app_container.rows.row.padding.top as f32
        + theme.app_container.rows.row.padding.bottom as f32;
    let title_padding = theme.app_container.rows.row.title.padding.top as f32
        + theme.app_container.rows.row.title.padding.bottom as f32;
    let row_font_size = theme.app_container.rows.row.title.font_size as f32;
    // Line height ~1.5x font size: accounts for iced's 1.3x default line height
    // plus font metric variations (ascent/descent) and sub-pixel rounding
    let row_line_height = row_font_size * 1.5;
    let row_height = row_line_height + title_padding + button_padding_v + row_padding;

    // Header height: Container(Row(cells)) with header.padding
    let header_height = if table.headers.is_some() {
        let h = &theme.app_container.rows.header;
        let h_padding = h.padding.top as f32 + h.padding.bottom as f32;
        let h_line_height = h.font_size as f32 * 1.5;
        h_line_height + h_padding + h.separator_width
    } else {
        0.0
    };

    // Total rows area content (inside the scrollable)
    let rows_content = (num_rows as f32 * row_height) + header_height;

    // Rows container padding
    let rows_padding_v = theme.app_container.rows.padding.top as f32
        + theme.app_container.rows.padding.bottom as f32;

    let rows_area_needed = rows_content + rows_padding_v;

    // Determine column height based on how iced's Column lays out children.
    // Fixed-height children get their exact size; FillPortion children share
    // whatever remains proportionally.
    let column_height = if filter_enabled {
        let search_len = theme.app_container.search.height;
        let rows_len = theme.app_container.rows.height;

        match (search_len, rows_len) {
            // Search has a fixed pixel height — rows get the remainder
            (Length::Fixed(search_fixed), _) => search_fixed + rows_area_needed,

            // Both use FillPortion — need enough so each portion >= its content
            (Length::FillPortion(sp), Length::FillPortion(rp)) => {
                let search = &theme.app_container.search;
                let input = &search.input;
                let input_padding =
                    input.padding.top as f32 + input.padding.bottom as f32;
                let search_padding =
                    search.padding.top as f32 + search.padding.bottom as f32;
                let search_content =
                    input.font_size as f32 * 1.5 + input_padding + search_padding + 12.0;

                let total = sp as f32 + rp as f32;
                let from_rows = rows_area_needed * total / rp as f32;
                let from_search = search_content * total / sp as f32;
                from_rows.max(from_search)
            }

            // Other combinations (Fill, Shrink, etc.) — just sum
            _ => {
                let search = &theme.app_container.search;
                let input = &search.input;
                let input_padding =
                    input.padding.top as f32 + input.padding.bottom as f32;
                let search_padding =
                    search.padding.top as f32 + search.padding.bottom as f32;
                let search_content =
                    input.font_size as f32 * 1.5 + input_padding + search_padding + 12.0;
                search_content + rows_area_needed
            }
        }
    } else {
        rows_area_needed
    };

    // Container paddings outside the column
    let app_padding_v = theme.app_container.padding.top as f32
        + theme.app_container.padding.bottom as f32;
    let outer_padding_v = theme.padding.top as f32 + theme.padding.bottom as f32;

    let height = column_height + app_padding_v + outer_padding_v;

    // Width estimate
    let column_spacing = theme.app_container.rows.column_spacing as f32;

    let col_widths: f32 = visible_cols
        .iter()
        .map(|&col| {
            let header_len = table
                .headers
                .as_ref()
                .and_then(|h| h.get(col))
                .map_or(0, |s| s.len());
            let max_cell_len = table
                .rows
                .iter()
                .map(|row| row.get(col).map_or(0, |s| s.len()))
                .max()
                .unwrap_or(0);
            let max_chars = header_len.max(max_cell_len) as f32;
            max_chars * char_width_estimate
        })
        .sum();

    // Column cell padding (left+right per column)
    let cell_h_padding = theme.app_container.rows.row.title.padding.left as f32
        + theme.app_container.rows.row.title.padding.right as f32;
    let total_cell_padding = cell_h_padding * num_cols as f32;

    // Button adds default 5px horizontal padding on each side
    let button_padding_h: f32 = 5.0 + 5.0;

    let row_h_padding = theme.app_container.rows.row.padding.left as f32
        + theme.app_container.rows.row.padding.right as f32;
    let rows_h_padding = theme.app_container.rows.padding.left as f32
        + theme.app_container.rows.padding.right as f32;
    let app_h_padding = theme.app_container.padding.left as f32
        + theme.app_container.padding.right as f32;
    let outer_h_padding = theme.padding.left as f32 + theme.padding.right as f32;

    let spacing = if num_cols > 1 {
        column_spacing * (num_cols - 1) as f32
    } else {
        0.0
    };

    let width = col_widths
        + total_cell_padding
        + spacing
        + button_padding_h
        + row_h_padding
        + rows_h_padding
        + app_h_padding
        + outer_h_padding;

    (width, height)
}
