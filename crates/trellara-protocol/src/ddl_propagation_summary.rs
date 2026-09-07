use crate::ddl_propagation_validation::validate_policy_release_gate;
use crate::{classify_ddl_propagation, DdlEvent, DdlPropagationDisposition, ProtocolError};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DdlPropagationSummary {
    pub auto_apply: usize,
    pub manual_review: usize,
    pub unsupported: usize,
    pub target_ack_required: usize,
}

impl DdlPropagationSummary {
    pub fn evidence(&self) -> String {
        format!(
            "propagation_decisions=auto_apply:{},manual_review:{},unsupported:{},target_ack_required:{}",
            self.auto_apply, self.manual_review, self.unsupported, self.target_ack_required
        )
    }
}

pub fn summarize_ddl_propagation(
    events: &[DdlEvent],
) -> Result<DdlPropagationSummary, ProtocolError> {
    let mut summary = DdlPropagationSummary::default();
    for event in events {
        let decision = classify_ddl_propagation(event)?;
        validate_policy_release_gate(event, &decision)?;
        if decision.requires_target_ack {
            summary.target_ack_required += 1;
        }
        match decision.disposition {
            DdlPropagationDisposition::AutoApply => summary.auto_apply += 1,
            DdlPropagationDisposition::ManualReview => summary.manual_review += 1,
            DdlPropagationDisposition::Unsupported => summary.unsupported += 1,
        }
    }
    Ok(summary)
}
