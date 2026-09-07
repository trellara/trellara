use serde::{Deserialize, Serialize};

use crate::LakeCompletenessState;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LakeEpochRecoveryGuidance {
    pub code: &'static str,
    pub consumer_gate: &'static str,
    pub operator_action: &'static str,
    pub evidence_required: &'static str,
}

pub fn lake_epoch_recovery_guidance(state: LakeCompletenessState) -> LakeEpochRecoveryGuidance {
    match state {
        LakeCompletenessState::Open | LakeCompletenessState::Sealing => pending_epoch_guidance(),
        LakeCompletenessState::Complete => complete_epoch_guidance(),
        LakeCompletenessState::CompleteWithGaps => complete_with_gaps_guidance(),
        LakeCompletenessState::Quarantined => quarantined_epoch_guidance(),
        LakeCompletenessState::Reseeding => reseeding_epoch_guidance(),
        LakeCompletenessState::FailedRecoverable => failed_recoverable_guidance(),
    }
}

fn pending_epoch_guidance() -> LakeEpochRecoveryGuidance {
    LakeEpochRecoveryGuidance {
        code: "epoch_not_sealed",
        consumer_gate: "blocked until the epoch is sealed and verified",
        operator_action:
            "wait for the writer to seal epoch metadata or rerun the writer if progress has stalled",
        evidence_required: "sealed epoch row plus matching _trellara_epoch_verification row",
    }
}

fn complete_epoch_guidance() -> LakeEpochRecoveryGuidance {
    LakeEpochRecoveryGuidance {
        code: "epoch_complete",
        consumer_gate: "released after checksum verification matches",
        operator_action: "publish the verified epoch to downstream lake consumers",
        evidence_required: "checksum_status=match for the same epoch_id",
    }
}

fn complete_with_gaps_guidance() -> LakeEpochRecoveryGuidance {
    LakeEpochRecoveryGuidance {
        code: "explicit_gap_acceptance_required",
        consumer_gate: "blocked unless the consumer explicitly accepts recorded gaps",
        operator_action: "record customer acceptance for the listed missing, lagging, or quarantined sources before consumption",
        evidence_required: "gap list in _trellara_epoch_sources and --accept-complete-with-gaps in verification evidence",
    }
}

fn quarantined_epoch_guidance() -> LakeEpochRecoveryGuidance {
    LakeEpochRecoveryGuidance {
        code: "quarantine_repair_required",
        consumer_gate: "blocked until quarantine evidence is repaired or replayed",
        operator_action: "inspect _trellara_quarantine, replay the exact transaction boundary when possible, or reseed the affected source",
        evidence_required: "empty or superseded quarantine rows plus a fresh matching verification artifact",
    }
}

fn reseeding_epoch_guidance() -> LakeEpochRecoveryGuidance {
    LakeEpochRecoveryGuidance {
        code: "source_reseed_required",
        consumer_gate: "blocked until affected sources complete snapshot-to-stream handoff",
        operator_action:
            "finish source reseed, publish a replacement epoch, and rerun fan-in verification",
        evidence_required:
            "verified reseed handoff and replacement epoch metadata for affected sources",
    }
}

fn failed_recoverable_guidance() -> LakeEpochRecoveryGuidance {
    LakeEpochRecoveryGuidance {
        code: "writer_replay_required",
        consumer_gate: "blocked until durable stream replay completes",
        operator_action: "resume the lake writer from durable stream offsets and regenerate epoch metadata before Spark visibility",
        evidence_required: "writer replay evidence, replacement epoch row, and checksum_status=match",
    }
}
