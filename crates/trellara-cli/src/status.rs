use std::path::Path;

use trellara_checkpoint::{CheckpointLag, PostgresCheckpointStore};
use trellara_pg_capture::PgCapture;

use crate::{
    apply_schema_contracts, flow_recovery_actions, render_correctness_report_text,
    render_dashboard_text, render_diagnostics_text, render_flow_alerts_text,
    render_flow_status_text, render_prometheus_metrics,
    status_target_evidence::{load_target_status_evidence, TargetStatusEvidence},
    CorrectnessReportSummary, DashboardSummary, DiagnosticsBundleSummary, FlowAlertsSummary,
    FlowSchemaDriftSummary, FlowStatusParts, FlowStatusSummary, QuickstartOutputFormat, Result,
    StatusView, TrellaraConfig,
};

pub(crate) async fn flow_status(config: &TrellaraConfig) -> Result<FlowStatusSummary> {
    let source_store = PostgresCheckpointStore::connect(&config.source.database_url, true).await?;
    let source_checkpoint = trellara_relay::load_source_checkpoint(
        &source_store,
        &config.source.id,
        &config.dataset.id,
    )
    .await?;
    let source_lag = source_checkpoint
        .as_ref()
        .map(CheckpointLag::from_checkpoint);
    let source_capture = PgCapture::connect(config.to_capture_config(false)?).await?;
    let source_slot = source_capture
        .inspect_slot_status(config.source.capture.expected_plugin())
        .await?;
    let subscription_conflicts = source_capture.inspect_subscription_conflict_stats().await?;
    let source_tables = apply_schema_contracts(config, source_capture.inspect_tables().await?);
    let source_schema_drift = FlowSchemaDriftSummary::from_preflight(&source_tables);

    let target_evidence = if let Some(target) = &config.target {
        load_target_status_evidence(config, target).await?
    } else {
        TargetStatusEvidence::empty()
    };
    let recovery_actions = flow_recovery_actions(
        config,
        &source_slot,
        source_schema_drift.as_ref(),
        target_evidence.latest_quarantine.as_ref(),
    )?;

    Ok(FlowStatusSummary::new(FlowStatusParts {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        mode: config.status_mode(),
        source_slot,
        subscription_conflicts,
        source_wal_retention_warn_bytes: config.source.wal_retention_warn_bytes,
        source_stream_spill_threshold_changes: config
            .source
            .stream_spill_threshold_changes
            .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES),
        source_stream_spill_dir: config
            .source
            .stream_spill_dir
            .as_ref()
            .map(|path| path.display().to_string()),
        source: source_lag,
        target: target_evidence.target_lag,
        partition_watermarks: target_evidence.partition_watermarks,
        source_schema_drift,
        latest_quarantine: target_evidence.latest_quarantine,
        latest_reseed: target_evidence.latest_reseed,
        latest_snapshot_handoff: target_evidence.latest_snapshot_handoff,
        latest_snapshot_run: target_evidence.latest_snapshot_run,
        latest_validation: target_evidence.latest_validation,
        recovery_actions,
    }))
}

pub(crate) fn render_status_view(
    status: FlowStatusSummary,
    view: StatusView,
    format: QuickstartOutputFormat,
    config_path: &Path,
) -> Result<String> {
    match (view, format) {
        (StatusView::Flow, QuickstartOutputFormat::Json) => {
            Ok(serde_json::to_string_pretty(&status)?)
        }
        (StatusView::Flow, QuickstartOutputFormat::Text) => Ok(render_flow_status_text(&status)),
        (StatusView::Report, QuickstartOutputFormat::Json) => Ok(serde_json::to_string_pretty(
            &CorrectnessReportSummary::from_status(status),
        )?),
        (StatusView::Report, QuickstartOutputFormat::Text) => Ok(render_correctness_report_text(
            &CorrectnessReportSummary::from_status(status),
        )),
        (StatusView::Alerts, QuickstartOutputFormat::Json) => Ok(serde_json::to_string_pretty(
            &FlowAlertsSummary::from_status(status),
        )?),
        (StatusView::Alerts, QuickstartOutputFormat::Text) => Ok(render_flow_alerts_text(
            &FlowAlertsSummary::from_status(status),
        )),
        (StatusView::Dashboard, QuickstartOutputFormat::Json) => Ok(serde_json::to_string_pretty(
            &DashboardSummary::from_status(status),
        )?),
        (StatusView::Dashboard, QuickstartOutputFormat::Text) => Ok(render_dashboard_text(
            &DashboardSummary::from_status(status),
        )),
        (StatusView::Metrics, _) => Ok(render_prometheus_metrics(status)),
        (StatusView::Diagnostics, QuickstartOutputFormat::Json) => {
            Ok(serde_json::to_string_pretty(
                &DiagnosticsBundleSummary::from_status(status, config_path),
            )?)
        }
        (StatusView::Diagnostics, QuickstartOutputFormat::Text) => Ok(render_diagnostics_text(
            &DiagnosticsBundleSummary::from_status(status, config_path),
        )),
    }
}
