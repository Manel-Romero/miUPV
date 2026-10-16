//! UI helpers for terminal output: spinners, tables, colors, and formatting.

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use console::{style, StyledObject};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Returns an indicatif spinner with a clean UPV-styled spinner message.
pub fn create_spinner(message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(message);
    pb
}

/// Returns an indicatif spinner, or a hidden progress bar if not enabled (e.g. for JSON output).
pub fn create_spinner_opt(message: &'static str, enabled: bool) -> ProgressBar {
    if !enabled {
        return ProgressBar::hidden();
    }
    create_spinner(message)
}

/// Creates a new pre-configured comfy-table with rounded corners and nice spacing.
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let header_cells = headers.iter().map(|h| {
        Cell::new(h)
            .fg(Color::Cyan)
            .add_attribute(comfy_table::Attribute::Bold)
    });

    table.set_header(header_cells);
    table
}

/// Formats a success message with a green checkmark.
pub fn success(msg: &str) -> StyledObject<String> {
    style(format!("✔ {msg}")).green().bold()
}

/// Formats an error message with a red cross.
pub fn error(msg: &str) -> StyledObject<String> {
    style(format!("✘ {msg}")).red().bold()
}

/// Formats an informational message with a blue indicator.
pub fn info(msg: &str) -> StyledObject<String> {
    style(format!("ℹ {msg}")).cyan()
}

/// Formats a warning message with a yellow warning indicator.
pub fn warning(msg: &str) -> StyledObject<String> {
    style(format!("⚠ {msg}")).yellow().bold()
}
