use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use console::{Emoji, style};
use indicatif::{ProgressBar, ProgressBarIter, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::time::Duration;

// Emojis for visual feedback
static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "✓ ");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "✗ ");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", ">> ");
static GEAR: Emoji<'_, '_> = Emoji("⚙️ ", "[*] ");
static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "(*) ");
static WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "[!] ");
static INFO: Emoji<'_, '_> = Emoji("ℹ️ ", "[i] ");

/// Professional output formatter for Unity Catalog CLI
pub struct OutputFormatter;

impl OutputFormatter {
    /// Print a main section header
    pub fn section_header(title: &str) {
        println!();
        println!("{}", style(format!("{} {}", ROCKET, title)).bold().cyan());
        println!("{}", style("=".repeat(title.len() + 4)).dim());
    }

    /// Print a subsection header
    pub fn subsection_header(title: &str) {
        println!();
        println!("{}", style(format!("{} {}", GEAR, title)).bold().yellow());
        println!("{}", style("-".repeat(title.len() + 4)).dim());
    }

    /// Print a success message
    pub fn success(message: &str) {
        println!("{}{}", CHECKMARK, style(message).green());
    }

    /// Print an error message
    pub fn error(message: &str) {
        println!("{}{}", CROSS_MARK, style(message).red());
    }

    /// Print a warning message
    pub fn warning(message: &str) {
        println!("{}{}", WARNING, style(message).yellow());
    }

    /// Print an info message
    pub fn info(message: &str) {
        println!("{}{}", INFO, style(message).blue());
    }

    /// Print a step in progress
    pub fn step(message: &str) {
        println!("  {} {}", style("→").dim(), message);
    }

    /// Print a completed step
    pub fn step_complete(message: &str) {
        println!("  {} {}", style("✓").green(), style(message).dim());
    }

    /// Print a failed step
    pub fn step_failed(message: &str) {
        println!("  {} {}", style("✗").red(), style(message).dim());
    }

    /// Create a progress bar for operations
    pub fn progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}",
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create a spinner for indeterminate operations
    pub fn spinner(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Print test results summary
    pub fn test_summary(passed: usize, failed: usize, total: usize) {
        println!();
        println!("{}", style("Test Results Summary").bold().underlined());

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Metric").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Count").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Percentage").add_attribute(comfy_table::Attribute::Bold),
            ]);

        // Add rows with colored content
        table.add_row(vec![
            Cell::new("Passed").fg(Color::Green),
            Cell::new(passed.to_string()).fg(Color::Green),
            Cell::new(format!("{:.1}%", (passed as f64 / total as f64) * 100.0)).fg(Color::Green),
        ]);

        if failed > 0 {
            table.add_row(vec![
                Cell::new("Failed").fg(Color::Red),
                Cell::new(failed.to_string()).fg(Color::Red),
                Cell::new(format!("{:.1}%", (failed as f64 / total as f64) * 100.0)).fg(Color::Red),
            ]);
        }

        table.add_row(vec![
            Cell::new("Total").add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total.to_string()).add_attribute(comfy_table::Attribute::Bold),
            Cell::new("100.0%").add_attribute(comfy_table::Attribute::Bold),
        ]);

        println!("{}", table);

        // Overall result
        if failed == 0 {
            println!();
            println!("{}{}", SPARKLES, style("All tests passed!").bold().green());
        } else {
            println!();
            println!(
                "{}{}",
                CROSS_MARK,
                style(format!("{} test(s) failed", failed)).bold().red()
            );
        }
    }

    /// Print catalog information in a formatted table
    pub fn catalog_table(catalogs: &[unitycatalog_common::CatalogInfo]) {
        if catalogs.is_empty() {
            Self::info("No catalogs found");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Name").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("ID").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Comment").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Storage Root").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Properties").add_attribute(comfy_table::Attribute::Bold),
            ]);

        for catalog in catalogs {
            let properties_str = if catalog.properties.is_empty() {
                style("None").dim().to_string()
            } else {
                catalog.properties.len().to_string() + " properties"
            };

            table.add_row(vec![
                Cell::new(&catalog.name).fg(Color::Cyan),
                Cell::new(catalog.id.as_deref().unwrap_or("N/A")).fg(Color::Yellow),
                Cell::new(catalog.comment.as_deref().unwrap_or("No comment")).fg(Color::Green),
                Cell::new(catalog.storage_root.as_deref().unwrap_or("N/A")).fg(Color::Blue),
                Cell::new(properties_str).fg(Color::Magenta),
            ]);
        }

        println!("{}", table);
    }

    /// Print schema information in a formatted table
    pub fn schema_table(schemas: &[unitycatalog_common::SchemaInfo]) {
        if schemas.is_empty() {
            Self::info("No schemas found");
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Name").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Full Name").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Catalog").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Comment").add_attribute(comfy_table::Attribute::Bold),
            ]);

        for schema in schemas {
            table.add_row(vec![
                Cell::new(&schema.name).fg(Color::Cyan),
                Cell::new(schema.full_name.as_deref().unwrap_or("N/A")).fg(Color::Yellow),
                Cell::new(&schema.catalog_name).fg(Color::Blue),
                Cell::new(schema.comment.as_deref().unwrap_or("No comment")).fg(Color::Green),
            ]);
        }

        println!("{}", table);
    }

    /// Print a fancy banner
    pub fn banner(title: &str) {
        let border = "═".repeat(title.len() + 4);
        println!();
        println!("{}", style(format!("╔{}╗", border)).bold().blue());
        println!(
            "{}",
            style(format!("║ {} ║", style(title).bold().white()))
                .bold()
                .blue()
        );
        println!("{}", style(format!("╚{}╝", border)).bold().blue());
        println!();
    }

    /// Print test category start
    pub fn test_category_start(category: &str) {
        println!();
        println!("{}", style(format!("┌─ {}", category)).bold().cyan());
    }

    /// Print test category end with result
    pub fn test_category_end(category: &str, success: bool) {
        let status = if success {
            style("PASSED").green()
        } else {
            style("FAILED").red()
        };
        println!(
            "{}",
            style(format!("└─ {} [{}]", category, status)).bold().cyan()
        );
    }

    /// Print operation start (with optional spinner)
    pub fn operation_start(operation: &str) -> ProgressBar {
        let spinner = Self::spinner(operation);
        spinner
    }

    /// Finish operation with success
    pub fn operation_success(spinner: &ProgressBar, message: &str) {
        spinner.finish_with_message(format!("{} {}", CHECKMARK, style(message).green()));
    }

    /// Finish operation with failure
    pub fn operation_failed(spinner: &ProgressBar, message: &str) {
        spinner.finish_with_message(format!("{} {}", CROSS_MARK, style(message).red()));
    }

    /// Print a separator line
    pub fn separator() {
        println!("{}", style("─".repeat(80)).dim());
    }

    /// Print key-value pairs in a nice format
    pub fn key_value_pairs(pairs: &[(&str, &str)]) {
        for (key, value) in pairs {
            println!(
                "  {} {}",
                style(format!("{}:", key)).bold().blue(),
                style(value).white()
            );
        }
    }

    /// Print validation result
    pub fn validation_result(field: &str, expected: &str, actual: &str, success: bool) {
        let icon = if success { CHECKMARK } else { CROSS_MARK };
        println!(
            "    {}{}: {} {} {}",
            icon,
            style(field).bold(),
            style("expected").dim(),
            if success {
                style(expected).green()
            } else {
                style(expected).red()
            },
            if success {
                format!("{} {}", style("✓").green(), style("matches").green())
            } else {
                format!(
                    "{} {} {}",
                    style("✗").red(),
                    style("got").red(),
                    style(actual).red()
                )
            }
        );
    }
}

/// Trait to add progress bar functionality to iterators
pub trait ProgressBarExt {
    type Item;
    fn with_progress(self, message: &str) -> ProgressBarIter<Self>
    where
        Self: Iterator + ExactSizeIterator + Sized;
}

impl<T> ProgressBarExt for T
where
    T: Iterator + ExactSizeIterator,
{
    type Item = T::Item;

    fn with_progress(self, message: &str) -> ProgressBarIter<Self>
    where
        Self: Iterator + ExactSizeIterator + Sized,
    {
        let pb = OutputFormatter::progress_bar(self.len() as u64, message);
        pb.wrap_iter(self)
    }
}
