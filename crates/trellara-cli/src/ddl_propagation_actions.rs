use crate::ddl_types::*;
use crate::{ddl_plan_change_kind_label, DatasetMode, TrellaraConfig};

pub(crate) fn ddl_propagation_actions(
    config: &TrellaraConfig,
    change: &DdlPlanChange,
    sinks: &[DdlPropagationSink],
) -> Vec<DdlPropagationAction> {
    sinks
        .iter()
        .map(|sink| {
            let (action, ack_condition) = match change.decision {
                DdlPlanDecision::Block => (
                    format!(
                        "do not propagate {}; keep later DML behind the schema barrier",
                        ddl_plan_change_kind_label(change.kind)
                    ),
                    "blocked change is reclassified or removed from the propagation set"
                        .to_string(),
                ),
                DdlPlanDecision::ManualReview => (
                    format!(
                        "prepare reviewed {} action for {}",
                        ddl_plan_change_kind_label(change.kind),
                        sink.name
                    ),
                    "operator approval records accepted schema version, relation mapping, and handoff LSN"
                        .to_string(),
                ),
                DdlPlanDecision::StageThenApply => (
                    format!(
                        "stage {} on {} before releasing buffered DML",
                        ddl_plan_change_kind_label(change.kind),
                        sink.name
                    ),
                    "staged rollout is validated with contract-test and a fresh snapshot handoff if required"
                        .to_string(),
                ),
                DdlPlanDecision::AutoApply => match sink.kind {
                    DdlPropagationSinkKind::TargetPostgres => (
                        format!(
                            "apply compatible {} to target Postgres in a schema transaction",
                            ddl_plan_change_kind_label(change.kind)
                        ),
                        "target preflight reports the expected schema fingerprint after DDL"
                            .to_string(),
                    ),
                    DdlPropagationSinkKind::RawCdcLake => (
                        "record schema version in raw CDC metadata while preserving append-only history"
                            .to_string(),
                        "lake writer plan can publish epoch metadata for the new schema version"
                            .to_string(),
                    ),
                    DdlPropagationSinkKind::SparkDerivedView => (
                        "regenerate current-state and SCD2 derived SQL for the new column set"
                            .to_string(),
                        "Spark template output is accepted for the same epoch/schema version"
                            .to_string(),
                    ),
                    DdlPropagationSinkKind::PartitionVisibility => (
                        "hold global partition visibility until every partition reaches the barrier"
                            .to_string(),
                        "partition-watermarks shows all partitions at or beyond the barrier LSN"
                            .to_string(),
                    ),
                },
            };
            let action = if sink.kind == DdlPropagationSinkKind::PartitionVisibility
                && config.dataset.mode != DatasetMode::PartitionedScaleMode
            {
                "no partition visibility action required for strict mode".to_string()
            } else {
                action
            };
            DdlPropagationAction {
                sink: sink.name.clone(),
                action,
                ack_condition,
            }
        })
        .collect()
}
