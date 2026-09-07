use trellara_checkpoint::SnapshotRunState;

use crate::FlowStatusSummary;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotHandoffProofStatus {
    Verified,
    MissingRun,
    MissingHandoffEvent,
    FailedRecoverable,
    IncompleteRunState,
    HandoffRecordedAwaitingStreamReplay,
    MissingConsistentLsn,
    IdentityMismatch,
    WatermarkMismatch,
}

impl SnapshotHandoffProofStatus {
    pub(crate) fn from_status(status: &FlowStatusSummary) -> Self {
        let Some(run) = status.latest_snapshot_run.as_ref() else {
            return Self::MissingRun;
        };
        if run.state == SnapshotRunState::FailedRecoverable {
            return Self::FailedRecoverable;
        }
        let Some(handoff) = status.latest_snapshot_handoff.as_ref() else {
            return Self::MissingHandoffEvent;
        };
        if !matches!(
            run.state,
            SnapshotRunState::Streaming | SnapshotRunState::Verified
        ) {
            if run.state == SnapshotRunState::StreamHandoffReady {
                return Self::HandoffRecordedAwaitingStreamReplay;
            }
            return Self::IncompleteRunState;
        }
        let Some(consistent_lsn) = run.consistent_lsn.as_ref() else {
            return Self::MissingConsistentLsn;
        };
        if run.source_id != status.source_id
            || run.dataset_id != status.dataset_id
            || handoff.source_id != status.source_id
            || handoff.dataset_id != status.dataset_id
        {
            return Self::IdentityMismatch;
        }
        if consistent_lsn != &handoff.watermark_lsn {
            return Self::WatermarkMismatch;
        }
        Self::Verified
    }

    pub(crate) fn is_verified(self) -> bool {
        self == Self::Verified
    }

    pub(crate) fn recommendation(self) -> &'static str {
        match self {
            Self::Verified => "",
            Self::MissingRun => {
                "run trellara snapshot --config <flow> before starting relay/apply for an initial copy"
            }
            Self::MissingHandoffEvent => {
                "resume trellara snapshot --config <flow> until stream_handoff_ready"
            }
            Self::FailedRecoverable => {
                "fix the snapshot copy failure, then resume trellara snapshot --config <flow> --run-id <run> until stream_handoff_ready"
            }
            Self::IncompleteRunState => {
                "resume trellara snapshot --config <flow> until the run reaches stream_handoff_ready"
            }
            Self::HandoffRecordedAwaitingStreamReplay => {
                "start trellara relay/apply or trellara run so CDC replays from the recorded snapshot handoff boundary"
            }
            Self::MissingConsistentLsn | Self::IdentityMismatch | Self::WatermarkMismatch => {
                "rerun trellara snapshot --config <flow> --force to create a fresh audited handoff"
            }
        }
    }
}
