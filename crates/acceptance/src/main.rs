use std::time::Instant;

use console::{Term, style};
use unitycatalog_acceptance::journeys::all_journeys;
use unitycatalog_acceptance::reporting::generate_journeys_summary_table;
use unitycatalog_acceptance::{AcceptanceResult, JourneyConfig};

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    dotenv::dotenv().ok();

    let term = Term::stdout();
    let start_time = Instant::now();

    // Show startup banner
    term.write_line(&format!(
        "{} {} {}",
        style("🚀").bold(),
        style("Unity Catalog Acceptance Tests").bold().cyan(),
        style("Starting journey execution...").dim()
    ))?;
    term.write_line("")?;

    let config = JourneyConfig::default();
    let mut results = Vec::new();
    let total_journeys = all_journeys().len();

    for (index, journey) in all_journeys().iter_mut().enumerate() {
        term.write_line(&format!(
            "{} Executing journey {}/{}: {}",
            style("▶️").bold(),
            index + 1,
            total_journeys,
            style(journey.name()).bold()
        ))?;

        let result = config.execute_journey(journey.as_mut()).await?;
        results.push(result);

        term.write_line("")?;
    }

    // Generate and display summary
    let total_duration = start_time.elapsed();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    term.write_line(&style("📊 Journey Execution Summary").bold().to_string())?;
    term.write_line(&style("═".repeat(50)).dim().to_string())?;

    // Summary table
    let summary_table = generate_journeys_summary_table(&results)?;
    term.write_line(&summary_table)?;

    // Overall statistics
    term.write_line("")?;
    term.write_line(&format!(
        "{} Total Duration: {}ms",
        style("⏱️").bold(),
        style(total_duration.as_millis()).bold()
    ))?;

    let status_color = if failed == 0 {
        console::Color::Green
    } else {
        console::Color::Red
    };
    term.write_line(&format!(
        "{} Results: {} successful, {} failed",
        style("📋").bold(),
        style(successful).bold().fg(status_color),
        style(failed).bold().fg(status_color)
    ))?;

    // Performance insights
    if results.len() > 1 {
        let avg_duration = total_duration.as_millis() / results.len() as u128;
        let fastest = results.iter().min_by_key(|r| r.duration).unwrap();
        let slowest = results.iter().max_by_key(|r| r.duration).unwrap();

        term.write_line("")?;
        term.write_line(&style("🔍 Performance Insights").bold().to_string())?;
        term.write_line(&format!("  • Average journey time: {}ms", avg_duration))?;
        term.write_line(&format!(
            "  • Fastest: {} ({}ms)",
            fastest.journey_name,
            fastest.duration.as_millis()
        ))?;
        term.write_line(&format!(
            "  • Slowest: {} ({}ms)",
            slowest.journey_name,
            slowest.duration.as_millis()
        ))?;
    }

    term.write_line("")?;

    // Final status
    if failed == 0 {
        term.write_line(&format!(
            "{} {} All journeys completed successfully!",
            style("🎉").bold(),
            style("SUCCESS:").bold().green()
        ))?;
    } else {
        term.write_line(&format!(
            "{} {} {} journey(s) failed. Check the logs above for details.",
            style("❌").bold(),
            style("FAILURE:").bold().red(),
            failed
        ))?;
        std::process::exit(1);
    }

    Ok(())
}
