use std::collections::HashSet;

use crate::ddl_next_commands::ddl_plan_next_commands;
use crate::ddl_propagation::ddl_propagation_plan;
use crate::ddl_propagation_actions::ddl_propagation_actions;
use crate::ddl_types::*;
use crate::{ddl_plan_change, ddl_plan_compatibility_label, DdlPlanArgs, Result, TrellaraConfig};

impl DdlPlanSummary {
    pub(crate) fn from_args(config: &TrellaraConfig, args: &DdlPlanArgs) -> Result<Self> {
        let configured_relations = config
            .dataset
            .tables
            .iter()
            .map(|table| format!("{}.{}", table.schema, table.name))
            .collect::<HashSet<_>>();
        let mut changes = args
            .changes
            .iter()
            .map(|change| ddl_plan_change(config, &configured_relations, args.apply_mode, change))
            .collect::<Result<Vec<_>>>()?;
        let auto_apply_count = changes
            .iter()
            .filter(|change| change.decision == DdlPlanDecision::AutoApply)
            .count();
        let staged_rollout_count = changes
            .iter()
            .filter(|change| change.decision == DdlPlanDecision::StageThenApply)
            .count();
        let manual_review_count = changes
            .iter()
            .filter(|change| change.decision == DdlPlanDecision::ManualReview)
            .count();
        let blocked_count = changes
            .iter()
            .filter(|change| change.decision == DdlPlanDecision::Block)
            .count();
        let verdict = if blocked_count > 0 {
            DdlPlanVerdict::Blocked
        } else if manual_review_count > 0 {
            DdlPlanVerdict::RequiresManualReview
        } else {
            DdlPlanVerdict::ReadyToApply
        };
        let blockers = ddl_plan_blockers(&changes);
        let propagation = ddl_propagation_plan(config, verdict, &changes);
        for change in &mut changes {
            change.propagation_actions =
                ddl_propagation_actions(config, change, &propagation.sinks);
        }
        let next_commands = ddl_plan_next_commands(
            verdict,
            &args.config,
            &args.changes,
            args.apply_mode,
            &propagation,
        );

        Ok(Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            apply_mode: args.apply_mode,
            unknown_table_policy: config.dataset.unknown_table_policy.to_string(),
            proposed_change_count: changes.len(),
            auto_apply_count,
            staged_rollout_count,
            manual_review_count,
            blocked_count,
            blockers,
            verdict,
            transaction_boundary_rule:
                "DDL is a schema barrier: target visibility for later row changes waits until every affected sink records the accepted source schema version and handoff LSN"
                    .to_string(),
            propagation,
            changes,
            next_commands,
        })
    }
}

fn ddl_plan_blockers(changes: &[DdlPlanChange]) -> Vec<DdlPlanBlocker> {
    changes
        .iter()
        .filter(|change| change.decision == DdlPlanDecision::Block)
        .map(|change| DdlPlanBlocker {
            change: change.input.clone(),
            kind: change.kind,
            object: change.object.clone(),
            relation: change.relation.clone(),
            compatibility: change.compatibility,
            reason: change.reason.clone(),
            release_impact: format!(
                "{} blocks post-DDL DML release until operator remediation",
                ddl_plan_compatibility_label(change.compatibility)
            ),
        })
        .collect()
}
