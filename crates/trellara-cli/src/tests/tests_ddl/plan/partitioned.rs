use super::*;

mod release_proof;
mod summary_validation;

#[test]
fn ddl_plan_stages_allowed_new_table_and_blocks_partition_key_change() {
    let yaml = STRICT_YAML.replace(
            "  mode: strict_transaction_order",
            "  mode: partitioned_scale_mode\n  unknown_table_policy: allow_compatible\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("partitioned.yml"),
            changes: vec![
                "add_table:public.refunds".to_string(),
                "change_partition_key:public.sales.region_id".to_string(),
            ],
            apply_mode: DdlPlanApplyMode::StagedRollout,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    assert_eq!(summary.verdict, DdlPlanVerdict::Blocked);
    assert_eq!(summary.staged_rollout_count, 1);
    assert_eq!(summary.blocked_count, 1);
    assert_eq!(summary.blockers.len(), 1);
    assert_eq!(summary.changes[0].decision, DdlPlanDecision::StageThenApply);
    assert_eq!(
        summary.changes[1].kind,
        DdlPlanChangeKind::ChangePartitionKey
    );
    assert_eq!(summary.changes[1].decision, DdlPlanDecision::Block);
    assert!(summary.changes[1]
        .boundary_rule
        .contains("partition-key contract"));
    assert_eq!(
        summary.blockers[0].kind,
        DdlPlanChangeKind::ChangePartitionKey
    );
    assert_eq!(
        summary.blockers[0].relation.as_deref(),
        Some("public.sales")
    );
    assert!(summary.blockers[0]
        .reason
        .contains("partitioned scale mode ordering"));
    assert!(summary.blockers[0]
        .release_impact
        .contains("blocks post-DDL DML release"));
    assert!(summary.propagation.release_blockers.iter().any(|blocker| {
        blocker.contains("change_partition_key on public.sales.region_id")
            && blocker.contains("partition-key changes alter partitioned scale mode ordering")
    }));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::StagedRollout
            && policy.active
            && policy.change_count == 1
            && policy.release_rule.contains("staged rollout validation")
            && policy.approval_evidence.contains("staged rollout approval")
            && policy.approval_evidence.contains("barrier_lsn")
    }));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::BlockUnsupported
            && policy.active
            && policy.change_count == 1
            && policy.release_rule.contains("blocks post-DDL DML")
    }));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::ShadowPlanOnly
            && policy.active
            && policy.change_count == 1
            && policy
                .release_rule
                .contains("no release barrier is recorded")
            && policy.approval_evidence.contains("shadow plan review")
    }));
    assert!(summary.propagation.requires_global_partition_pause);
    assert_eq!(
        summary.propagation.ack_quorum,
        "all_required_sinks_plus_every_partition_lane"
    );
    assert_eq!(
        summary.propagation.required_ack_count,
        summary.propagation.sink_count
    );
    assert!(summary
        .propagation
        .cdc_transaction_boundary
        .contains("every partition lane watermark is at or beyond barrier_lsn"));
    assert!(summary.propagation.cdc_transaction_boundary.contains(
        "propagation_boundary=source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks"
    ));
    assert!(summary.propagation.cdc_transaction_boundary.contains(
        "propagation_decisions=auto_apply:0,manual_review:1,unsupported:1,target_ack_required:1"
    ));
    assert!(summary
        .propagation
        .cdc_transaction_boundary
        .contains("propagation_policy_sha256="));
    assert!(summary
        .propagation
        .row_visibility_mode
        .contains("all partition lanes"));
    assert!(summary.propagation.release_gates.iter().any(|gate| {
        gate.name == "partition_visibility_watermark"
            && gate
                .required_evidence
                .contains("every partition lane at or beyond barrier_lsn")
            && gate.opens_when.contains("all partition lanes")
    }));
    assert!(summary.propagation.release_gates.iter().any(|gate| {
        gate.name == "required_sink_acknowledgements"
            && gate.required_evidence.contains("detail evidence")
            && gate.opens_when.contains("include audit detail")
    }));
    assert!(summary.propagation.release_gates.iter().any(|gate| {
        gate.name == "post_ddl_dml_release"
            && gate
                .opens_when
                .contains("before partitioned DML can leave quarantine")
    }));
    assert!(summary.propagation.sinks.iter().any(|sink| {
        sink.kind == DdlPropagationSinkKind::PartitionVisibility
            && sink.required_ack.contains("every partition lane")
            && sink
                .ack_evidence
                .contains("partition-watermarks output showing every partition")
    }));
    assert!(summary.changes[0]
        .propagation_actions
        .iter()
        .any(|action| action.sink == "partition_visibility"
            && action.action.contains("stage add_table")));
    assert!(summary.changes[1]
        .propagation_actions
        .iter()
        .all(|action| action.action.contains("do not propagate")));
    assert_eq!(summary.changes[1].target_postgres_sql, None);
    assert!(!summary
        .next_commands
        .iter()
        .any(|command| command.contains("ddl-barrier record")));
}

#[test]
fn partitioned_safe_ddl_ack_commands_include_partition_visibility_evidence() {
    let yaml = STRICT_YAML.replace(
            "  mode: strict_transaction_order",
            "  mode: partitioned_scale_mode\n  unknown_table_policy: allow_compatible\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("partitioned.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    assert_eq!(summary.verdict, DdlPlanVerdict::ReadyToApply);
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("--sink target_postgres")
            && command.contains("--plan-sha256 <target-ddl-plan-sha256>")
            && command.contains("--statement-sha256 <target-ddl-statement-sha256>")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("--sink partition_visibility")
            && command.contains("--barrier-lsn <ddl-lsn>")
            && command.contains("--expected-partition-count <partition-count>")
            && command.contains("--partition-durable-lsn <partition-id>=<durable-lsn>")
            && command.contains("--partition-applied-lsn <partition-id>=<applied-lsn>")
    }));
}
