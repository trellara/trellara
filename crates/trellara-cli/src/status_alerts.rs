use serde::Serialize;

use crate::{
    source_alerts, status_alerts_failure::failure_alerts,
    status_alerts_partition::partition_alerts, status_alerts_target::target_alerts,
    FlowHealthStatus, FlowStatusSummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowAlertsSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) status: FlowHealthStatus,
    pub(crate) alert_count: usize,
    pub(crate) highest_severity: Option<FlowAlertSeverity>,
    pub(crate) alerts: Vec<FlowAlert>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FlowAlertSeverity {
    Warning,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowAlert {
    pub(crate) code: String,
    pub(crate) severity: FlowAlertSeverity,
    pub(crate) message: String,
    pub(crate) recommendation: String,
}

impl FlowAlertsSummary {
    pub(crate) fn from_status(status: FlowStatusSummary) -> Self {
        let mut alerts = Vec::new();

        alerts.extend(source_alerts(&status));
        alerts.extend(target_alerts(&status));
        alerts.extend(partition_alerts(&status));
        alerts.extend(failure_alerts(&status));

        let highest_severity = alerts.iter().map(|alert| alert.severity).max();

        Self {
            source_id: status.source_id,
            dataset_id: status.dataset_id,
            status: status.health.status,
            alert_count: alerts.len(),
            highest_severity,
            alerts,
        }
    }
}

impl FlowAlert {
    pub(crate) fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: FlowAlertSeverity::Warning,
            message: message.into(),
            recommendation: recommendation.into(),
        }
    }

    pub(crate) fn critical(
        code: impl Into<String>,
        message: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: FlowAlertSeverity::Critical,
            message: message.into(),
            recommendation: recommendation.into(),
        }
    }
}
