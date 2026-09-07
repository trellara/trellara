use serde::Serialize;
use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};

use crate::{FlowAlertSeverity, FlowHealthStatus, SourceSafetyInitRecommendation};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SourceSafetySummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) score: u8,
    pub(crate) grade: SourceSafetyGrade,
    pub(crate) status: FlowHealthStatus,
    pub(crate) slot: ReplicationSlotStatus,
    pub(crate) subscription_conflicts: Vec<SubscriptionConflictStats>,
    pub(crate) factor_count: usize,
    pub(crate) critical_factor_count: usize,
    pub(crate) warning_factor_count: usize,
    pub(crate) factors: Vec<SourceSafetyFactor>,
    pub(crate) recommended_actions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DirectSourceSafetySummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) read_only: bool,
    pub(crate) score: u8,
    pub(crate) grade: SourceSafetyGrade,
    pub(crate) status: FlowHealthStatus,
    pub(crate) table_count: usize,
    pub(crate) unsafe_table_count: usize,
    pub(crate) slot: ReplicationSlotStatus,
    pub(crate) subscription_conflicts: Vec<SubscriptionConflictStats>,
    pub(crate) factor_count: usize,
    pub(crate) critical_factor_count: usize,
    pub(crate) warning_factor_count: usize,
    pub(crate) factors: Vec<SourceSafetyFactor>,
    pub(crate) recommended_actions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) init_recommendation: Option<SourceSafetyInitRecommendation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) init_config_written: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceSafetyGrade {
    A,
    B,
    C,
    D,
    F,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SourceSafetyFactor {
    pub(crate) code: String,
    pub(crate) severity: FlowAlertSeverity,
    pub(crate) points_lost: u8,
    pub(crate) evidence: String,
    pub(crate) recommendation: String,
}

impl SourceSafetyFactor {
    pub(crate) fn warning(
        code: impl Into<String>,
        points_lost: u8,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: FlowAlertSeverity::Warning,
            points_lost,
            evidence: evidence.into(),
            recommendation: recommendation.into(),
        }
    }

    pub(crate) fn critical(
        code: impl Into<String>,
        points_lost: u8,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: FlowAlertSeverity::Critical,
            points_lost,
            evidence: evidence.into(),
            recommendation: recommendation.into(),
        }
    }
}
