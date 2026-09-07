use crate::{ddl_plan_change_kind_label, DdlPlanChange, DdlPlanDecision, DdlPlanVerdict};

pub(crate) fn ddl_release_blockers(
    verdict: DdlPlanVerdict,
    changes: &[DdlPlanChange],
) -> Vec<String> {
    let mut blockers = match verdict {
        DdlPlanVerdict::ReadyToApply => Vec::new(),
        DdlPlanVerdict::RequiresManualReview => {
            vec!["manual review must accept the schema change before DML release".to_string()]
        }
        DdlPlanVerdict::Blocked => {
            vec!["schema policy blockers must be resolved before DML release".to_string()]
        }
    };
    blockers.extend(change_release_blockers(changes));
    blockers
}

fn change_release_blockers(changes: &[DdlPlanChange]) -> Vec<String> {
    changes
        .iter()
        .filter_map(|change| match change.decision {
            DdlPlanDecision::ManualReview => Some(format!(
                "{} on {} requires operator approval before post-DDL DML release: {}",
                ddl_plan_change_kind_label(change.kind),
                change.object,
                change.reason
            )),
            DdlPlanDecision::Block => Some(format!(
                "{} on {} blocks post-DDL DML release: {}",
                ddl_plan_change_kind_label(change.kind),
                change.object,
                change.reason
            )),
            DdlPlanDecision::AutoApply | DdlPlanDecision::StageThenApply => None,
        })
        .collect()
}
