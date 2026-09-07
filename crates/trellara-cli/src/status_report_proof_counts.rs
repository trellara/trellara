use crate::{CorrectnessProofCheck, CorrectnessProofStatus};

pub(crate) struct CorrectnessProofCounts {
    pub(crate) total: usize,
    pub(crate) verified: usize,
    pub(crate) at_risk: usize,
    pub(crate) missing_evidence: usize,
}

impl CorrectnessProofCounts {
    pub(crate) fn from_checks(proof_checks: &[CorrectnessProofCheck]) -> Self {
        Self {
            total: proof_checks.len(),
            verified: count_status(proof_checks, CorrectnessProofStatus::Verified),
            at_risk: count_status(proof_checks, CorrectnessProofStatus::AtRisk),
            missing_evidence: count_status(proof_checks, CorrectnessProofStatus::MissingEvidence),
        }
    }
}

fn count_status(proof_checks: &[CorrectnessProofCheck], status: CorrectnessProofStatus) -> usize {
    proof_checks
        .iter()
        .filter(|check| check.status == status)
        .count()
}
