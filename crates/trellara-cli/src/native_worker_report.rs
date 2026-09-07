use serde::Serialize;
#[cfg(test)]
use trellara_relay::NativeWorkerRunReport;
use trellara_relay::NativeWorkerRunStatus;

use crate::{NativeWorkerReportStatus, QuickstartOutputFormat, Result};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NativeWorkerReportView {
    pub(crate) status: String,
    pub(crate) drained_frames: usize,
    pub(crate) source_feedback_lsn: Option<String>,
    pub(crate) reason: String,
    pub(crate) operator_action: String,
}

#[cfg(test)]
pub(crate) fn native_worker_report_view(report: &NativeWorkerRunReport) -> NativeWorkerReportView {
    native_worker_report_view_from_parts(
        &report.status,
        report.drained_frames,
        report.source_feedback_lsn.clone(),
        report.reason.to_string(),
    )
}

pub(crate) fn native_worker_report_view_from_parts(
    status: &NativeWorkerRunStatus,
    drained_frames: usize,
    source_feedback_lsn: Option<String>,
    reason: String,
) -> NativeWorkerReportView {
    NativeWorkerReportView {
        status: native_worker_status_label(status).to_string(),
        drained_frames,
        source_feedback_lsn,
        reason,
        operator_action: operator_action(status).to_string(),
    }
}

pub(crate) fn render_native_worker_report_view(
    view: &NativeWorkerReportView,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(view)?),
        QuickstartOutputFormat::Text => Ok(render_native_worker_report_text(view)),
    }
}

#[cfg(test)]
pub(crate) fn render_native_worker_report(
    report: &NativeWorkerRunReport,
    format: QuickstartOutputFormat,
) -> Result<String> {
    render_native_worker_report_view(&native_worker_report_view(report), format)
}

pub(crate) fn native_status_from_args(status: NativeWorkerReportStatus) -> NativeWorkerRunStatus {
    match status {
        NativeWorkerReportStatus::Slept => NativeWorkerRunStatus::Slept,
        NativeWorkerReportStatus::SourceFeedbackReady => NativeWorkerRunStatus::SourceFeedbackReady,
        NativeWorkerReportStatus::FailedClosed => NativeWorkerRunStatus::FailedClosed,
    }
}

fn render_native_worker_report_text(view: &NativeWorkerReportView) -> String {
    let source_feedback_lsn = view.source_feedback_lsn.as_deref().unwrap_or("none");
    format!(
        "native worker: {}\ndrained frames: {}\nsource feedback lsn: {}\nreason: {}\noperator action: {}\n",
        view.status,
        view.drained_frames,
        source_feedback_lsn,
        view.reason,
        view.operator_action
    )
}

fn native_worker_status_label(status: &NativeWorkerRunStatus) -> &'static str {
    match status {
        NativeWorkerRunStatus::Slept => "slept",
        NativeWorkerRunStatus::SourceFeedbackReady => "source_feedback_ready",
        NativeWorkerRunStatus::FailedClosed => "failed_closed",
    }
}

fn operator_action(status: &NativeWorkerRunStatus) -> &'static str {
    match status {
        NativeWorkerRunStatus::Slept => "leave source feedback unchanged",
        NativeWorkerRunStatus::SourceFeedbackReady => "advance source feedback after durable proof",
        NativeWorkerRunStatus::FailedClosed => "hold source feedback and inspect rejection reason",
    }
}
