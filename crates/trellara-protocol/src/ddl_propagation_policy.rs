use crate::{DdlEvent, DdlPropagationDecision, DdlPropagationDisposition};

pub(crate) fn ddl_policy_row(event: &DdlEvent, decision: &DdlPropagationDecision) -> String {
    let relation = event
        .relation
        .as_ref()
        .map(|relation| relation.display_name())
        .unwrap_or_default();
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        event.total_order,
        relation,
        event.operation,
        disposition_label(decision.disposition),
        event.target_auto_apply,
        decision.requires_target_ack,
        event.release_gate,
        decision.release_gate,
        decision.cdc_boundary,
        event.schema_fingerprint_before,
        event.schema_fingerprint_after,
        event.statement
    )
}

fn disposition_label(disposition: DdlPropagationDisposition) -> &'static str {
    match disposition {
        DdlPropagationDisposition::AutoApply => "auto_apply",
        DdlPropagationDisposition::ManualReview => "manual_review",
        DdlPropagationDisposition::Unsupported => "unsupported",
    }
}
