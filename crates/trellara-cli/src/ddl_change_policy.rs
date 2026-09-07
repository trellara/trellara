use crate::{
    DdlPlanApplyMode, DdlPlanChangeKind, DdlPlanCompatibility, DdlPlanDecision, TrellaraConfig,
    UnknownTablePolicy,
};

pub(crate) fn ddl_change_compatibility(
    config: &TrellaraConfig,
    kind: DdlPlanChangeKind,
    configured_relation: bool,
) -> (DdlPlanCompatibility, String) {
    match kind {
        DdlPlanChangeKind::AddNullableColumn
        | DdlPlanChangeKind::WidenType
        | DdlPlanChangeKind::IncreaseVarchar => {
            if configured_relation {
                (
                    DdlPlanCompatibility::Compatible,
                    "compatible column evolution on a configured dataset table".to_string(),
                )
            } else {
                (
                    DdlPlanCompatibility::BlockedByPolicy,
                    "relation is not configured in dataset.tables".to_string(),
                )
            }
        }
        DdlPlanChangeKind::AddTable => match config.dataset.unknown_table_policy {
            UnknownTablePolicy::AllowCompatible => (
                DdlPlanCompatibility::Compatible,
                "unknown_table_policy=allow_compatible permits controlled new-table adoption"
                    .to_string(),
            ),
            UnknownTablePolicy::Reject => (
                DdlPlanCompatibility::BlockedByPolicy,
                "unknown_table_policy=reject requires adding the table to config before adoption"
                    .to_string(),
            ),
        },
        DdlPlanChangeKind::RenameColumn | DdlPlanChangeKind::RenameTable => (
            DdlPlanCompatibility::RequiresMapping,
            "rename-like DDL is ambiguous without an explicit mapping".to_string(),
        ),
        DdlPlanChangeKind::DropColumn
        | DdlPlanChangeKind::NarrowType
        | DdlPlanChangeKind::AddNotNullColumn => (
            DdlPlanCompatibility::DestructiveOrAmbiguous,
            "destructive or stricter DDL can break replay, snapshots, or target apply".to_string(),
        ),
        DdlPlanChangeKind::ChangePrimaryKey => (
            DdlPlanCompatibility::DestructiveOrAmbiguous,
            "primary-key changes alter replica identity and idempotent apply semantics".to_string(),
        ),
        DdlPlanChangeKind::ChangePartitionKey => (
            DdlPlanCompatibility::DestructiveOrAmbiguous,
            "partition-key changes alter partitioned scale mode ordering and barrier semantics"
                .to_string(),
        ),
        DdlPlanChangeKind::Unknown => (
            DdlPlanCompatibility::RequiresMapping,
            "unknown DDL kind requires operator classification".to_string(),
        ),
    }
}

pub(crate) fn ddl_change_decision(
    apply_mode: DdlPlanApplyMode,
    compatibility: DdlPlanCompatibility,
) -> DdlPlanDecision {
    match (apply_mode, compatibility) {
        (_, DdlPlanCompatibility::BlockedByPolicy) => DdlPlanDecision::Block,
        (DdlPlanApplyMode::AutoSafe, DdlPlanCompatibility::Compatible) => {
            DdlPlanDecision::AutoApply
        }
        (DdlPlanApplyMode::StagedRollout, DdlPlanCompatibility::Compatible) => {
            DdlPlanDecision::StageThenApply
        }
        (_, DdlPlanCompatibility::Compatible) => DdlPlanDecision::ManualReview,
        (DdlPlanApplyMode::ManualReview, _) => DdlPlanDecision::ManualReview,
        (_, DdlPlanCompatibility::RequiresMapping) => DdlPlanDecision::ManualReview,
        (_, DdlPlanCompatibility::DestructiveOrAmbiguous) => DdlPlanDecision::Block,
    }
}
