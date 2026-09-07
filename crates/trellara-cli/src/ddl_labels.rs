use crate::{
    DdlPlanApplyMode, DdlPlanChangeKind, DdlPlanCompatibility, DdlPlanDecision, DdlPlanVerdict,
    DdlPropagationPolicyMode, DdlPropagationSinkKind,
};

pub(crate) fn ddl_apply_mode_cli_value(apply_mode: DdlPlanApplyMode) -> &'static str {
    match apply_mode {
        DdlPlanApplyMode::ManualReview => "manual-review",
        DdlPlanApplyMode::AutoSafe => "auto-safe",
        DdlPlanApplyMode::StagedRollout => "staged-rollout",
        DdlPlanApplyMode::BlockDestructive => "block-destructive",
    }
}

pub(crate) fn ddl_plan_verdict_label(verdict: DdlPlanVerdict) -> &'static str {
    match verdict {
        DdlPlanVerdict::ReadyToApply => "ready_to_apply",
        DdlPlanVerdict::RequiresManualReview => "requires_manual_review",
        DdlPlanVerdict::Blocked => "blocked",
    }
}

pub(crate) fn ddl_plan_change_kind_label(kind: DdlPlanChangeKind) -> &'static str {
    match kind {
        DdlPlanChangeKind::AddNullableColumn => "add_nullable_column",
        DdlPlanChangeKind::AddTable => "add_table",
        DdlPlanChangeKind::WidenType => "widen_type",
        DdlPlanChangeKind::IncreaseVarchar => "increase_varchar",
        DdlPlanChangeKind::DropColumn => "drop_column",
        DdlPlanChangeKind::RenameColumn => "rename_column",
        DdlPlanChangeKind::RenameTable => "rename_table",
        DdlPlanChangeKind::NarrowType => "narrow_type",
        DdlPlanChangeKind::ChangePrimaryKey => "change_primary_key",
        DdlPlanChangeKind::ChangePartitionKey => "change_partition_key",
        DdlPlanChangeKind::AddNotNullColumn => "add_not_null_column",
        DdlPlanChangeKind::Unknown => "unknown",
    }
}

pub(crate) fn ddl_plan_compatibility_label(compatibility: DdlPlanCompatibility) -> &'static str {
    match compatibility {
        DdlPlanCompatibility::Compatible => "compatible",
        DdlPlanCompatibility::RequiresMapping => "requires_mapping",
        DdlPlanCompatibility::DestructiveOrAmbiguous => "destructive_or_ambiguous",
        DdlPlanCompatibility::BlockedByPolicy => "blocked_by_policy",
    }
}

pub(crate) fn ddl_plan_decision_label(decision: DdlPlanDecision) -> &'static str {
    match decision {
        DdlPlanDecision::AutoApply => "auto_apply",
        DdlPlanDecision::StageThenApply => "stage_then_apply",
        DdlPlanDecision::ManualReview => "manual_review",
        DdlPlanDecision::Block => "block",
    }
}

pub(crate) fn ddl_propagation_sink_kind_label(kind: DdlPropagationSinkKind) -> &'static str {
    match kind {
        DdlPropagationSinkKind::TargetPostgres => "target_postgres",
        DdlPropagationSinkKind::RawCdcLake => "raw_cdc_lake",
        DdlPropagationSinkKind::SparkDerivedView => "spark_derived_view",
        DdlPropagationSinkKind::PartitionVisibility => "partition_visibility",
    }
}

pub(crate) fn ddl_propagation_policy_mode_label(mode: DdlPropagationPolicyMode) -> &'static str {
    match mode {
        DdlPropagationPolicyMode::AutoApply => "auto_apply",
        DdlPropagationPolicyMode::StagedRollout => "staged_rollout",
        DdlPropagationPolicyMode::ManualApprovalRequired => "manual_approval_required",
        DdlPropagationPolicyMode::BlockUnsupported => "block_unsupported",
        DdlPropagationPolicyMode::ShadowPlanOnly => "shadow_plan_only",
    }
}
