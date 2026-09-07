use super::*;

#[test]
fn partitioned_ddl_release_proof_requires_partition_visibility_ack() {
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
    let apply_plan = DdlApplyPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("partitioned.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl apply plan");

    let proof =
        pilot_package_ddl_release_proof(&config, &summary, &apply_plan).expect("release proof");

    assert!(proof
        .required_sinks
        .iter()
        .any(|sink| sink == "raw_cdc_lake"));
    assert!(proof
        .required_sinks
        .iter()
        .any(|sink| sink == "spark_derived_views"));
    assert!(proof
        .required_sinks
        .iter()
        .any(|sink| sink == "partition_visibility"));
    assert_eq!(proof.release_gate, "post_ddl_dml_release");
    assert_eq!(
        proof.cdc_transaction_boundary,
        summary.propagation.cdc_transaction_boundary
    );
    assert!(proof
        .cdc_transaction_boundary
        .contains("every partition lane watermark"));
    assert_eq!(
        proof.release_summary.cdc_transaction_boundary,
        proof.cdc_transaction_boundary
    );
    assert!(proof.ack_commands.iter().any(|command| {
        command.contains("trellara schema ddl-barrier ack --config trellara.yml")
            && command.contains("--sink partition_visibility")
            && command.contains("--barrier-lsn 0/16B8000")
            && command.contains("--expected-partition-count 4")
            && command.contains("--partition-durable-lsn 0=0/16B9000")
            && command.contains("--partition-durable-lsn 3=0/16B9000")
            && command.contains("--partition-applied-lsn 0=0/16B9000")
            && command.contains("--partition-applied-lsn 3=0/16B9000")
    }));
    assert!(proof.ack_commands.iter().any(|command| {
        command.contains("--sink raw_cdc_lake")
            && command.contains("--epoch-id epoch-schema-ddl-sample")
            && command
                .contains("--metadata-table retail_sales__trellara__fanin___trellara_epoch_sources")
            && command.contains(
                "--partition-metadata-table retail_sales__trellara__fanin___trellara_epoch_partitions"
            )
            && command.contains(
                "--manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            )
    }));
    assert!(proof.release_decision.release_dml);
    assert!(proof.release_decision.blocker_codes.is_empty());
    assert!(proof.release_decision.blocker_details.is_empty());
    assert!(proof.release_evidence.iter().any(|evidence| {
        evidence.contains("barrier")
            && evidence.contains(&proof.barrier_id)
            && evidence.contains(&proof.barrier_lsn)
            && evidence.contains(&proof.schema_version)
    }));
    assert!(proof
        .release_evidence
        .iter()
        .any(|evidence| evidence.contains("4/4 required sink ACKs accepted")));
    assert!(proof.release_evidence.iter().any(|evidence| {
        evidence.contains("release_dml=true") && evidence.contains("blocker_codes=none")
    }));
    assert!(proof
        .release_evidence
        .iter()
        .any(|evidence| evidence.contains(&proof.cdc_transaction_boundary)));
    assert!(proof.release_evidence.iter().any(|evidence| {
        evidence.contains("partition_visibility release evidence")
            && evidence.contains("4/4 partitions")
            && evidence.contains("global_durable_lsn 0/16B9000")
            && evidence.contains("global_applied_lsn 0/16B9000")
            && evidence.contains("partition_watermark_sha256=")
    }));
    assert!(proof.release_summary.release_dml);
    assert_eq!(proof.release_summary.pending_sink_count, 0);
    assert_eq!(
        proof.release_summary.acked_sink_count,
        proof.required_sinks.len()
    );
    assert_eq!(
        proof.ddl_dml_replay_proof.contract,
        "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary"
    );
    assert_eq!(proof.ddl_dml_replay_proof.barrier_lsn, proof.barrier_lsn);
    assert_eq!(proof.ddl_dml_replay_proof.target_ack_lsn, proof.barrier_lsn);
    assert_eq!(proof.ddl_dml_replay_proof.dml_commit_lsn, proof.barrier_lsn);
    assert_eq!(
        proof.ddl_dml_replay_proof.ddl_applied_statements,
        apply_plan.statement_count
    );
    assert_eq!(proof.ddl_dml_replay_proof.dml_applied_changes, 1);
    let serialized_proof = serde_json::to_string(&proof).expect("serialize release proof");
    assert!(crate::pilot_evidence_markers::live_evidence_marker_present(
        "ddl_release_proof",
        "ddl_dml_replay_proof",
        &serialized_proof
    ));
    assert!(proof.ack_evidence.iter().any(|ack| {
        ack.sink == "partition_visibility"
            && ack.accepted
            && ack.ack_lsn == "0/16B9000"
            && ack.detail.contains("4/4 partitions")
            && ack.detail.contains("partition_watermark_sha256=")
            && ack.detail.contains("post_ddl_dml_release")
    }));
    assert!(proof.ack_evidence.iter().any(|ack| {
        ack.sink == "target_postgres"
            && ack.accepted
            && ack
                .detail
                .contains("target Postgres applied 1 DDL statements")
            && ack.detail.contains(&apply_plan.plan_sha256)
            && ack
                .detail
                .contains(&apply_plan.statements[0].statement_sha256)
            && ack.detail.contains("barrier_lsn=0/16B8000")
            && ack.detail.contains("post_ddl_dml_release")
    }));
    assert!(proof.ack_commands.iter().any(|command| {
        command.contains("--sink target_postgres")
            && command.contains("--barrier-lsn 0/16B8000")
            && command.contains("--plan-sha256")
            && command.contains(&apply_plan.plan_sha256)
            && command.contains("--statement-sha256")
            && command.contains(&apply_plan.statements[0].statement_sha256)
    }));
}
