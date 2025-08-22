//! Rich reporting and logging utilities for acceptance tests
//!
//! This module provides utilities for generating rich, condensed output during
//! journey execution with progress indicators, tables, and structured logging.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use comfy_table::{Attribute, Cell, ContentArrangement, Row, Table};
use console::{Color as ConsoleColor, Emoji, Term, style};

use crate::AcceptanceResult;

/// Emojis used throughout the reporting
pub struct ReportingEmojis;

impl ReportingEmojis {
    pub const ROCKET: Emoji<'_, '_> = Emoji("🚀", ">");
    pub const CHECK: Emoji<'_, '_> = Emoji("✅", "✓");
    pub const CROSS: Emoji<'_, '_> = Emoji("❌", "✗");
    pub const WARNING: Emoji<'_, '_> = Emoji("⚠️", "!");
    pub const INFO: Emoji<'_, '_> = Emoji("ℹ️", "i");
    pub const FOLDER: Emoji<'_, '_> = Emoji("📁", "+");
    pub const LIST: Emoji<'_, '_> = Emoji("📋", "-");
    pub const SEARCH: Emoji<'_, '_> = Emoji("🔍", "?");
    pub const TRASH: Emoji<'_, '_> = Emoji("🗑️", "x");
    pub const CLEAN: Emoji<'_, '_> = Emoji("🧹", "~");
    pub const CELEBRATE: Emoji<'_, '_> = Emoji("🎉", "*");
    pub const HOURGLASS: Emoji<'_, '_> = Emoji("⏳", "-");
}

/// Configuration for journey reporting
#[derive(Debug, Clone)]
pub struct ReportingConfig {
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Verbosity level (0=minimal, 1=normal, 2=verbose)
    pub verbosity: u8,
    /// Whether to show timing information
    pub show_timing: bool,
    /// Width for tables (None = auto-detect)
    pub table_width: Option<usize>,
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            use_colors: console::colors_enabled(),
            verbosity: 1,
            show_timing: true,
            table_width: None,
        }
    }
}

/// A structured step in a journey for reporting purposes
#[derive(Debug, Clone)]
pub struct JourneyStep {
    pub id: String,
    pub description: String,
    pub status: StepStatus,
    pub duration: Option<Duration>,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed(String),
    Skipped,
}

impl StepStatus {
    pub fn emoji(&self) -> Emoji<'_, '_> {
        match self {
            StepStatus::Pending => ReportingEmojis::HOURGLASS,
            StepStatus::Running => ReportingEmojis::HOURGLASS,
            StepStatus::Success => ReportingEmojis::CHECK,
            StepStatus::Failed(_) => ReportingEmojis::CROSS,
            StepStatus::Skipped => ReportingEmojis::WARNING,
        }
    }

    pub fn color(&self) -> ConsoleColor {
        match self {
            StepStatus::Pending => ConsoleColor::Yellow,
            StepStatus::Running => ConsoleColor::Cyan,
            StepStatus::Success => ConsoleColor::Green,
            StepStatus::Failed(_) => ConsoleColor::Red,
            StepStatus::Skipped => ConsoleColor::Yellow,
        }
    }
}

/// Main reporter for journey execution
pub struct JourneyReporter {
    config: ReportingConfig,
    term: Term,
    journey_name: String,
    steps: Vec<JourneyStep>,
    start_time: Instant,
    step_timings: HashMap<String, Instant>,
}

impl JourneyReporter {
    /// Create a new journey reporter
    pub fn new(journey_name: impl Into<String>, config: ReportingConfig) -> Self {
        let term = Term::stdout();

        Self {
            config,
            term,
            journey_name: journey_name.into(),
            steps: Vec::new(),
            start_time: Instant::now(),
            step_timings: HashMap::new(),
        }
    }

    /// Start the journey reporting
    pub fn start_journey(&mut self, description: &str) -> AcceptanceResult<()> {
        if self.config.verbosity > 0 {
            let header = if self.config.use_colors {
                format!(
                    "{} {} {}",
                    style(ReportingEmojis::ROCKET).bold(),
                    style(&self.journey_name).bold().cyan(),
                    style(description).dim()
                )
            } else {
                format!(
                    "{} {} {}",
                    ReportingEmojis::ROCKET,
                    self.journey_name,
                    description
                )
            };

            self.term.write_line(&header)?;

            if self.config.verbosity > 1 {
                self.term.write_line(&format!(
                    "Started at: {}",
                    chrono::Utc::now().format("%H:%M:%S UTC")
                ))?;
            }
        }

        Ok(())
    }

    /// Add a step to track
    pub fn add_step(&mut self, id: impl Into<String>, description: impl Into<String>) {
        let step = JourneyStep {
            id: id.into(),
            description: description.into(),
            status: StepStatus::Pending,
            duration: None,
            details: None,
        };
        self.steps.push(step);
    }

    /// Start executing a step
    pub fn start_step(&mut self, step_id: &str) -> AcceptanceResult<()> {
        self.step_timings
            .insert(step_id.to_string(), Instant::now());

        // Auto-register step if it doesn't exist
        if !self.steps.iter().any(|s| s.id == step_id) {
            self.add_step(step_id, step_id); // Use step_id as description if not pre-registered
        }

        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = StepStatus::Running;

            if self.config.verbosity > 0 {
                let msg = if self.config.use_colors {
                    format!(
                        "  {} {}",
                        style(step.status.emoji()).bold(),
                        style(&step.description).dim()
                    )
                } else {
                    format!("  {} {}", step.status.emoji(), step.description)
                };
                self.term.write_line(&msg)?;
            }
        }

        Ok(())
    }

    /// Complete a step successfully
    pub fn complete_step(
        &mut self,
        step_id: &str,
        details: Option<String>,
    ) -> AcceptanceResult<()> {
        let duration = self
            .step_timings
            .remove(step_id)
            .map(|start| start.elapsed());

        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = StepStatus::Success;
            step.duration = duration;
            step.details = details;

            if self.config.verbosity > 0 {
                let timing_info = if self.config.show_timing && step.duration.is_some() {
                    format!(" ({}ms)", step.duration.unwrap().as_millis())
                } else {
                    String::new()
                };

                let msg = if self.config.use_colors {
                    format!(
                        "  {} {}{}",
                        style(step.status.emoji()).bold().green(),
                        style(&step.description),
                        style(timing_info).dim()
                    )
                } else {
                    format!(
                        "  {} {}{}",
                        step.status.emoji(),
                        step.description,
                        timing_info
                    )
                };
                self.term.write_line(&msg)?;

                if let Some(details) = &step.details {
                    if self.config.verbosity > 1 {
                        self.term.write_line(&format!("    └─ {}", details))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Fail a step
    pub fn fail_step(&mut self, step_id: &str, error: &str) -> AcceptanceResult<()> {
        let duration = self
            .step_timings
            .remove(step_id)
            .map(|start| start.elapsed());

        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = StepStatus::Failed(error.to_string());
            step.duration = duration;

            let msg = if self.config.use_colors {
                format!(
                    "  {} {} - {}",
                    style(step.status.emoji()).bold().red(),
                    style(&step.description),
                    style(error).red()
                )
            } else {
                format!("  {} {} - {}", step.status.emoji(), step.description, error)
            };
            self.term.write_line(&msg)?;
        }

        Ok(())
    }

    /// Finish the journey and show summary
    pub fn finish_journey(&mut self, success: bool) -> AcceptanceResult<()> {
        let total_duration = self.start_time.elapsed();

        if self.config.verbosity > 0 {
            let icon = if success {
                ReportingEmojis::CELEBRATE
            } else {
                ReportingEmojis::CROSS
            };
            let status_text = if success { "COMPLETED" } else { "FAILED" };

            let summary = if self.config.use_colors {
                let color = if success {
                    ConsoleColor::Green
                } else {
                    ConsoleColor::Red
                };
                format!(
                    "{} Journey {} {} ({}ms)",
                    style(icon).bold(),
                    style(&self.journey_name).bold().fg(color),
                    style(status_text).bold().fg(color),
                    total_duration.as_millis()
                )
            } else {
                format!(
                    "{} Journey {} {} ({}ms)",
                    icon,
                    self.journey_name,
                    status_text,
                    total_duration.as_millis()
                )
            };

            self.term.write_line("")?;
            self.term.write_line(&summary)?;
        }

        Ok(())
    }

    /// Generate a summary table of all steps
    pub fn generate_summary_table(&self) -> AcceptanceResult<String> {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);

        if let Some(width) = self.config.table_width {
            table.set_width(width as u16);
        }

        // Header
        table.set_header(Row::from(vec![
            Cell::new("Step").add_attribute(Attribute::Bold),
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new("Duration").add_attribute(Attribute::Bold),
            Cell::new("Details").add_attribute(Attribute::Bold),
        ]));

        // Rows
        for step in &self.steps {
            let duration_str = step
                .duration
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "-".to_string());

            let (status_text, status_color) = match &step.status {
                StepStatus::Success => ("✓ Success", comfy_table::Color::Green),
                StepStatus::Failed(_err) => ("✗ Failed", comfy_table::Color::Red),
                StepStatus::Running => ("⏳ Running", comfy_table::Color::Yellow),
                StepStatus::Pending => ("⏳ Pending", comfy_table::Color::Yellow),
                StepStatus::Skipped => ("⚠ Skipped", comfy_table::Color::Yellow),
            };

            let details = match &step.status {
                StepStatus::Failed(err) => err.clone(),
                _ => step.details.clone().unwrap_or_else(|| "-".to_string()),
            };

            table.add_row(Row::from(vec![
                Cell::new(&step.description),
                Cell::new(status_text).fg(status_color),
                Cell::new(duration_str),
                Cell::new(details),
            ]));
        }

        Ok(table.to_string())
    }

    /// Log an info message
    pub fn info(&self, message: &str) -> AcceptanceResult<()> {
        if self.config.verbosity > 0 {
            let msg = if self.config.use_colors {
                format!(
                    "  {} {}",
                    style(ReportingEmojis::INFO).blue(),
                    style(message).dim()
                )
            } else {
                format!("  {} {}", ReportingEmojis::INFO, message)
            };
            self.term.write_line(&msg)?;
        }
        Ok(())
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) -> AcceptanceResult<()> {
        if self.config.verbosity > 0 {
            let msg = if self.config.use_colors {
                format!(
                    "  {} {}",
                    style(ReportingEmojis::WARNING).yellow(),
                    style(message).yellow()
                )
            } else {
                format!("  {} {}", ReportingEmojis::WARNING, message)
            };
            self.term.write_line(&msg)?;
        }
        Ok(())
    }
}

/// Generate a summary table for multiple journey results
pub fn generate_journeys_summary_table(
    results: &[crate::execution::JourneyExecutionResult],
) -> AcceptanceResult<String> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Header
    table.set_header(Row::from(vec![
        Cell::new("Journey").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("Duration").add_attribute(Attribute::Bold),
        Cell::new("Steps").add_attribute(Attribute::Bold),
        Cell::new("Error").add_attribute(Attribute::Bold),
    ]));

    // Rows
    for result in results {
        let (status_text, status_color) = if result.success {
            ("✓ Success", comfy_table::Color::Green)
        } else {
            ("✗ Failed", comfy_table::Color::Red)
        };

        let error_msg = result.error_message.as_deref().unwrap_or("-");

        table.add_row(Row::from(vec![
            Cell::new(&result.journey_name),
            Cell::new(status_text).fg(status_color),
            Cell::new(format!("{}ms", result.duration.as_millis())),
            Cell::new(result.steps_completed.to_string()),
            Cell::new(error_msg),
        ]));
    }

    Ok(table.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_status_colors() {
        assert_eq!(StepStatus::Success.color(), ConsoleColor::Green);
        assert_eq!(
            StepStatus::Failed("test".to_string()).color(),
            ConsoleColor::Red
        );
        assert_eq!(StepStatus::Pending.color(), ConsoleColor::Yellow);
    }

    #[test]
    fn test_reporting_config_default() {
        let config = ReportingConfig::default();
        assert_eq!(config.verbosity, 1);
        assert!(config.show_timing);
    }

    #[test]
    fn test_journey_reporter_creation() {
        let config = ReportingConfig::default();
        let reporter = JourneyReporter::new("test_journey", config);
        assert_eq!(reporter.journey_name, "test_journey");
        assert!(reporter.steps.is_empty());
    }
}
