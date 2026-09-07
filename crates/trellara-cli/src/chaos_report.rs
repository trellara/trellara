use std::fs;

use serde::Serialize;

use crate::{
    render_chaos_report_html, ChaosEnterpriseReviewGate, ChaosReportArgs, ChaosRunSummary,
    CliError, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosReportWriteSummary {
    pub(crate) output: String,
    pub(crate) passed: bool,
    pub(crate) report_version: String,
    pub(crate) source_revision: String,
    pub(crate) source_repository: String,
    pub(crate) workflow_run_url: String,
    pub(crate) deterministic_seed: u64,
    pub(crate) simulation_count: usize,
    pub(crate) scenario_count: usize,
    pub(crate) enterprise_review_gate_count: usize,
    pub(crate) enterprise_review_gates: Vec<ChaosEnterpriseReviewGate>,
    pub(crate) verification_command: String,
}

pub(crate) fn write_chaos_report(args: &ChaosReportArgs) -> Result<ChaosReportWriteSummary> {
    let mut summary = ChaosRunSummary::default();
    summary.metadata.source_revision = args.source_revision.clone();
    summary.metadata.source_repository = args.source_repository.clone();
    summary.metadata.workflow_run_url = args.workflow_run_url.clone();
    if let Some(parent) = args
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| CliError::WriteOutput {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(&args.output, render_chaos_report_html(&summary)).map_err(|source| {
        CliError::WriteOutput {
            path: args.output.display().to_string(),
            source,
        }
    })?;

    Ok(ChaosReportWriteSummary {
        output: args.output.display().to_string(),
        passed: summary.passed,
        report_version: summary.metadata.report_version,
        source_revision: summary.metadata.source_revision,
        source_repository: summary.metadata.source_repository,
        workflow_run_url: summary.metadata.workflow_run_url,
        deterministic_seed: summary.deterministic_seed,
        simulation_count: summary.simulation_count,
        scenario_count: summary.scenario_count,
        enterprise_review_gate_count: summary.enterprise_review_gates.len(),
        enterprise_review_gates: summary.enterprise_review_gates,
        verification_command: summary.verification_command,
    })
}
