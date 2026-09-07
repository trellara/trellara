use crate::{PilotLiveEvidenceGate, PilotLiveEvidenceStatus};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PilotLiveEvidenceGateCounts {
    pub(crate) accepted: usize,
    pub(crate) missing: usize,
    pub(crate) insufficient: usize,
    pub(crate) not_required: usize,
    pub(crate) blocked: usize,
}

impl PilotLiveEvidenceGateCounts {
    pub(crate) fn from_gates(gates: &[PilotLiveEvidenceGate]) -> Self {
        Self {
            accepted: count_status(gates, PilotLiveEvidenceStatus::Accepted),
            missing: count_status(gates, PilotLiveEvidenceStatus::Missing),
            insufficient: count_status(gates, PilotLiveEvidenceStatus::Insufficient),
            not_required: count_status(gates, PilotLiveEvidenceStatus::NotRequired),
            blocked: count_status(gates, PilotLiveEvidenceStatus::Blocked),
        }
    }

    pub(crate) fn verdict(&self) -> &'static str {
        if self.blocked > 0 {
            "blocked_until_scorecard_fixed"
        } else if self.missing > 0 {
            "missing_live_evidence"
        } else if self.insufficient > 0 {
            "live_evidence_insufficient"
        } else {
            "live_evidence_accepted"
        }
    }
}

pub(crate) fn next_evidence_commands(
    gates: &[PilotLiveEvidenceGate],
    rerun_command: String,
) -> Vec<String> {
    let mut next_commands = gates
        .iter()
        .filter(|gate| {
            matches!(
                gate.evidence_status,
                PilotLiveEvidenceStatus::Missing | PilotLiveEvidenceStatus::Insufficient
            )
        })
        .map(|gate| gate.proof_command.clone())
        .collect::<Vec<_>>();
    next_commands.sort();
    next_commands.dedup();
    next_commands.push(rerun_command);
    next_commands
}

fn count_status(gates: &[PilotLiveEvidenceGate], status: PilotLiveEvidenceStatus) -> usize {
    gates
        .iter()
        .filter(|gate| gate.evidence_status == status)
        .count()
}
