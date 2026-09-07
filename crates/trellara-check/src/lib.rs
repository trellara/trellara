mod args;
mod diagnostic;
mod error;
mod factors;
mod factors_postgres;
mod factors_slot_failover;
mod factors_slot_recommendation;
mod factors_slots;
mod factors_subscription;
mod factors_tables;
mod html_escape;
mod labels;
mod render;
mod render_html;
mod render_html_evidence;
mod render_html_queries;
mod render_text;
mod scoring;
mod slot_evidence;
mod url_redaction;

pub use args::{CheckArgs, CheckOutputFormat};
pub use diagnostic::{CheckFactor, CheckGrade, CheckSeverity, CheckStatus, CheckSummary};
pub use error::{CheckError, Result};
pub use render::render_check_summary;
pub use url_redaction::redact_database_url;

pub async fn run_check(args: &CheckArgs) -> Result<String> {
    let inspection =
        trellara_pg_capture::inspect_database_source_safety(&args.database_url).await?;
    let summary =
        CheckSummary::from_inspection(redact_database_url(&args.database_url), inspection);
    render_check_summary(&summary, args.format)
}
