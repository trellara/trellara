use super::*;

#[test]
fn ddl_plan_auto_safe_allows_configured_additive_change() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    assert_eq!(summary.verdict, DdlPlanVerdict::ReadyToApply);
    assert_eq!(summary.auto_apply_count, 1);
    assert_eq!(summary.blocked_count, 0);
    assert_eq!(
        summary.changes[0].kind,
        DdlPlanChangeKind::AddNullableColumn
    );
    assert_eq!(summary.changes[0].decision, DdlPlanDecision::AutoApply);
    assert!(summary.changes[0].boundary_rule.contains("schema barrier"));
    assert_eq!(
        summary.changes[0].target_postgres_sql.as_deref(),
        Some("ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;")
    );
    assert!(summary.changes[0]
        .target_postgres_sql
        .as_ref()
        .expect("target sql")
        .contains("IF NOT EXISTS"));
    assert!(summary.transaction_boundary_rule.contains("handoff LSN"));
    assert!(summary.propagation.barrier_id.starts_with("ddl-barrier-"));
    assert_eq!(summary.propagation.sink_count, 3);
    assert_eq!(summary.propagation.required_ack_count, 3);
    assert_eq!(summary.propagation.ack_quorum, "all_required_sinks");
    assert!(summary
        .propagation
        .cdc_transaction_boundary
        .contains("source commit LSN is the DDL barrier"));
    assert!(summary
        .propagation
        .cdc_transaction_boundary
        .contains("DDL and DML share source transaction order"));
    assert!(summary.propagation.cdc_transaction_boundary.contains(
        "propagation_boundary=source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks"
    ));
    assert!(summary.propagation.cdc_transaction_boundary.contains(
        "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1"
    ));
    assert!(summary
        .propagation
        .cdc_transaction_boundary
        .contains("propagation_policy_sha256="));
    assert!(summary
        .propagation
        .cdc_transaction_boundary
        .contains("required sink ACK is at or beyond barrier_lsn"));
    assert!(summary.propagation.release_blockers.is_empty());
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::AutoApply
            && policy.active
            && policy.change_count == 1
            && policy.release_rule.contains("required sink ACK evidence")
            && policy
                .approval_evidence
                .contains("target DDL apply-plan digest")
            && policy.approval_evidence.contains("barrier_lsn")
    }));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::ManualApprovalRequired && !policy.active
    }));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::ShadowPlanOnly && !policy.active
    }));
    let text = render_ddl_plan_text(&summary);
    assert!(text.contains("approval_evidence="));
    assert!(text.contains("target DDL apply-plan digest"));
    assert!(!summary.propagation.requires_global_partition_pause);
    assert!(summary.propagation.dml_after_barrier_held);
    assert!(summary
        .propagation
        .row_visibility_mode
        .contains("strict: later transactions remain invisible"));
    assert!(summary.propagation.release_gates.iter().any(|gate| {
        gate.name == "schema_barrier_recorded"
            && gate.required_evidence.contains("barrier_lsn")
            && gate.opens_when.contains("before any later DML")
    }));
    assert!(summary.propagation.release_gates.iter().any(|gate| {
        gate.name == "required_sink_acknowledgements"
            && gate.required_evidence.contains("detail evidence")
            && gate.opens_when.contains("include audit detail")
    }));
    assert!(summary.propagation.release_gates.iter().any(|gate| {
        gate.name == "post_ddl_dml_release"
            && gate.required_evidence.contains("release_blockers")
            && gate
                .opens_when
                .contains("after the DDL transaction boundary")
    }));
    assert!(summary
        .propagation
        .sinks
        .iter()
        .any(|sink| sink.kind == DdlPropagationSinkKind::TargetPostgres
            && sink.ack_evidence.contains("target preflight JSON")));
    assert!(summary
        .propagation
        .sinks
        .iter()
        .any(|sink| sink.kind == DdlPropagationSinkKind::SparkDerivedView
            && sink.ack_evidence.contains("template_sha256")));
    assert!(summary.changes[0]
        .propagation_actions
        .iter()
        .any(|action| action.sink == "raw_cdc_lake" && action.action.contains("schema version")));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("schema ddl-barrier record")
            && command.contains("--change add_nullable_column:public.sales.discount_code:text")
            && command.contains("--apply-mode auto-safe")
            && command.contains("--barrier-lsn <ddl-lsn>")
            && command.contains("--schema-version <schema-version>")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("schema ddl-barrier ack")
            && command.contains(&summary.propagation.barrier_id)
            && command.contains("--sink target_postgres")
            && command.contains("--plan-sha256 <target-ddl-plan-sha256>")
            && command.contains("--statement-sha256 <target-ddl-statement-sha256>")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("--sink raw_cdc_lake")
            && command.contains("--epoch-id <lake-epoch-id>")
            && command.contains("--metadata-table <raw-cdc-epoch-metadata-table>")
            && command.contains("--partition-metadata-table <raw-cdc-epoch-partitions-table>")
            && command.contains("--manifest-digest <lake-epoch-manifest-sha256>")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("--sink spark_derived_views")
            && command
                .contains("--template-digest <spark-template-sha256-hex-from-template_sha256>")
            && command.contains("--accepted-by <reviewer-or-automation>")
            && command.contains("--view-count <derived-view-count>")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("lake spark-template current-state")
            && command.contains("--epoch-id <lake-epoch-id>")
            && command.contains("--format json")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("lake spark-template scd2")
            && command.contains("--epoch-id <lake-epoch-id>")
            && command.contains("--format json")
    }));
    assert!(summary.next_commands.iter().any(|command| {
        command.contains("schema ddl-barrier status")
            && command.contains(&summary.propagation.barrier_id)
    }));
}
