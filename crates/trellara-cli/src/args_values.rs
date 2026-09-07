use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum DdlPlanApplyMode {
    ManualReview,
    AutoSafe,
    StagedRollout,
    BlockDestructive,
}

impl std::fmt::Display for DdlPlanApplyMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdlPlanApplyMode::ManualReview => formatter.write_str("manual_review"),
            DdlPlanApplyMode::AutoSafe => formatter.write_str("auto_safe"),
            DdlPlanApplyMode::StagedRollout => formatter.write_str("staged_rollout"),
            DdlPlanApplyMode::BlockDestructive => formatter.write_str("block_destructive"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StatusView {
    Flow,
    Report,
    Alerts,
    Dashboard,
    Metrics,
    Diagnostics,
}

impl std::fmt::Display for StatusView {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Flow => "flow",
            Self::Report => "report",
            Self::Alerts => "alerts",
            Self::Dashboard => "dashboard",
            Self::Metrics => "metrics",
            Self::Diagnostics => "diagnostics",
        };
        formatter.write_str(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum QuickstartOutputFormat {
    Json,
    Text,
}

impl std::fmt::Display for QuickstartOutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Json => "json",
            Self::Text => "text",
        };
        formatter.write_str(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PilotGuideOutputFormat {
    Json,
    Text,
}

impl std::fmt::Display for PilotGuideOutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Json => "json",
            Self::Text => "text",
        };
        formatter.write_str(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SourceSafetyOutputFormat {
    Html,
    Json,
    Text,
}

impl std::fmt::Display for SourceSafetyOutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Html => "html",
            Self::Json => "json",
            Self::Text => "text",
        };
        formatter.write_str(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum TransactionInspectOutputFormat {
    Json,
    Text,
}

impl std::fmt::Display for TransactionInspectOutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Json => "json",
            Self::Text => "text",
        };
        formatter.write_str(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum NativeWorkerReportStatus {
    Slept,
    SourceFeedbackReady,
    FailedClosed,
}

impl std::fmt::Display for NativeWorkerReportStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Slept => "slept",
            Self::SourceFeedbackReady => "source_feedback_ready",
            Self::FailedClosed => "failed_closed",
        };
        formatter.write_str(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum LakeEpochScenario {
    OfflineStoresPublishWithGaps,
    LateStoreRecoveryCompletesEpoch,
    DuplicateStoreTransactionReplay,
    ConflictingDuplicateQuarantine,
}
