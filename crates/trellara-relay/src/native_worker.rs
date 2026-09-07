use trellara_pg_extension::{
    native_worker_supervision_decision, NativeHandoffDrainBatch, NativeSourceFeedbackDecision,
    NativeWorkerSupervisionState,
};

use crate::{native_feedback_decision_from_relay_step, RelayError, RelayStep, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWorkerRunOutcome {
    Slept,
    SourceFeedbackReady(NativeSourceFeedbackDecision),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWorkerRunStatus {
    Slept,
    SourceFeedbackReady,
    FailedClosed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWorkerRunReport {
    pub status: NativeWorkerRunStatus,
    pub drained_frames: usize,
    pub source_feedback_lsn: Option<String>,
    pub reason: &'static str,
}

pub fn run_native_worker_once(
    state: &NativeWorkerSupervisionState,
    batch: &NativeHandoffDrainBatch,
    step: &RelayStep,
) -> Result<NativeWorkerRunOutcome> {
    let supervision = native_worker_supervision_decision(state)
        .map_err(RelayError::NativeWorkerSupervisionRejected)?;
    if supervision.should_sleep {
        return Ok(NativeWorkerRunOutcome::Slept);
    }

    native_feedback_decision_from_relay_step(batch, step)
        .map(NativeWorkerRunOutcome::SourceFeedbackReady)
}

pub fn run_native_worker_once_report(
    state: &NativeWorkerSupervisionState,
    batch: &NativeHandoffDrainBatch,
    step: &RelayStep,
) -> NativeWorkerRunReport {
    match run_native_worker_once(state, batch, step) {
        Ok(NativeWorkerRunOutcome::Slept) => report(
            NativeWorkerRunStatus::Slept,
            0,
            None,
            "queue_empty_after_supervision_ready",
        ),
        Ok(NativeWorkerRunOutcome::SourceFeedbackReady(decision)) => report(
            NativeWorkerRunStatus::SourceFeedbackReady,
            decision.frames_covered,
            Some(decision.source_feedback_lsn),
            "durable_relay_proof_allows_source_feedback",
        ),
        Err(RelayError::NativeWorkerSupervisionRejected(_)) => report(
            NativeWorkerRunStatus::FailedClosed,
            0,
            None,
            "supervision_rejected",
        ),
        Err(RelayError::NativeFeedbackProofNotDurable { .. }) => report(
            NativeWorkerRunStatus::FailedClosed,
            0,
            None,
            "relay_proof_not_durable",
        ),
        Err(RelayError::NativeFeedbackRejected(_)) => report(
            NativeWorkerRunStatus::FailedClosed,
            0,
            None,
            "native_feedback_rejected",
        ),
        Err(_) => report(
            NativeWorkerRunStatus::FailedClosed,
            0,
            None,
            "unexpected_worker_error",
        ),
    }
}

fn report(
    status: NativeWorkerRunStatus,
    drained_frames: usize,
    source_feedback_lsn: Option<String>,
    reason: &'static str,
) -> NativeWorkerRunReport {
    NativeWorkerRunReport {
        status,
        drained_frames,
        source_feedback_lsn,
        reason,
    }
}
