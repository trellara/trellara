use super::*;

#[test]
fn schema_ddl_propagation_artifacts_require_packaged_barrier_contract() {
    let root = temp_root("ddl-propagation-ready");
    let crate_src = root.join("crates").join("trellara-cli").join("src");
    fs::create_dir_all(&crate_src).expect("create ddl propagation temp dir");
    fs::create_dir_all(root.join("docs")).expect("create ddl propagation docs dir");
    fs::write(
            root.join("docs").join("DESIGN.md"),
            "schema ddl-plan emits a schema-barrier transaction boundary with a structured propagation contract, per-sink acknowledgements, partitioned global-visibility pause, and DML held until targets accept the new schema version.",
        )
        .expect("write readme");
    fs::write(
            root.join("Makefile"),
            "quickstart-proof-check verifies schema-ddl-plan.json, schema-ddl-apply-plan.json, schema-ddl-envelope-plan.json, ddl-barrier-status.json, ddl-release-proof.json top-level cdc_transaction_boundary, partitioned-schema-ddl-plan.json, blocked-schema-ddl-plan.json, \"propagation\", \"dml_after_barrier_held\": true, \"release_gates\", \"post_ddl_dml_release\", \"release_gate_code\": \"post_ddl_dml_release\", \"requires_global_partition_pause\": false, \"requires_global_partition_pause\": true, \"kind\": \"target_postgres\", \"kind\": \"raw_cdc_lake\", \"kind\": \"spark_derived_view\", \"kind\": \"partition_visibility\", partitioned: all partition lanes, partition-watermarks output showing every partition, partition_visibility_watermark, \"required_ack\", \"cdc_transaction_boundary\", \"ack_commands\", \"ack_evidence\", \"release_dml\", \"release_blockers\", \"blockers\", \"release_impact\", change_partition_key:public.sales.region_id, partition-key changes alter partitioned scale mode ordering, \"release_blocker_codes\", \"rejection_code\", \"release_decision\", \"blocker_codes\", pending required sink acknowledgements, \"target_postgres_sql\", \"target_postgres_transaction_script\", schema_ddl_envelope_plan, \"ddl_event_count\", \"executable\", \"statement_count\", \"propagation_boundary\": \"source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks\", \"propagation_decisions\", \"propagation_policy_sha256\", ALTER TABLE, BEGIN, COMMIT, Do not record target_postgres ACK, trellara schema ddl-barrier record --config, trellara lake spark-template current-state --config, trellara lake spark-template scd2 --config, --format json, trellara schema ddl-barrier ack --config, --plan-sha256 <target-ddl-plan-sha256>, --statement-sha256 <target-ddl-statement-sha256>, --epoch-id <lake-epoch-id>, --metadata-table <raw-cdc-epoch-metadata-table>, --partition-metadata-table <raw-cdc-epoch-partitions-table>, --manifest-digest <lake-epoch-manifest-sha256>, --template-digest <spark-template-sha256-hex-from-template_sha256>, --accepted-by <reviewer-or-automation>, --view-count <derived-view-count>, plan_sha256=, statement_sha256=, partition_metadata_table=, manifest_digest=, template_digest=, accepted_by=, --expected-partition-count <partition-count>, --partition-durable-lsn <partition-id>=<durable-lsn>, --partition-applied-lsn <partition-id>=<applied-lsn>, and trellara schema ddl-barrier status --config.",
        )
        .expect("write makefile");
    fs::write(
        crate_src.join("ddl_types.rs"),
        "struct DdlPlanBlocker; struct DdlApplyPlanSummary; release_gate_code release_impact",
    )
    .expect("write ddl types");
    fs::write(
            crate_src.join("ddl_propagation_types.rs"),
            "struct DdlPropagationPlan; struct DdlPropagationSink; struct DdlPropagationPhase; struct DdlReleaseGate; release_gates dml_after_barrier_held requires_global_partition_pause ack_evidence cdc_transaction_boundary",
        )
        .expect("write ddl propagation types");
    fs::write(
        crate_src.join("ddl.rs"),
        "DdlPlanSummary::from_args ddl_plan_blockers",
    )
    .expect("write ddl");
    fs::write(crate_src.join("ddl_change.rs"), "DdlPlanChange::from_args")
        .expect("write ddl change");
    fs::write(
        crate_src.join("ddl_sql.rs"),
        "fn ddl_target_postgres_sql ADD COLUMN ALTER COLUMN",
    )
    .expect("write ddl sql");
    fs::write(
            crate_src.join("ddl_propagation.rs"),
            "DdlPropagationSinkKind::RawCdcLake DdlPropagationSinkKind::SparkDerivedView DdlPropagationSinkKind::PartitionVisibility partition-watermarks shows all partitions at or beyond the barrier LSN",
        )
        .expect("write ddl propagation");
    fs::write(
        crate_src.join("ddl_release_gates.rs"),
        "fn ddl_release_gates post_ddl_dml_release",
    )
    .expect("write ddl release gates");
    fs::write(
        crate_src.join("ddl_apply_plan.rs"),
        "DdlApplyPlanSummary post_ddl_dml_release release_gate_code target_postgres_transaction_script Do not record target_postgres ACK",
    )
    .expect("write ddl apply plan");
    fs::write(
        crate_src.join("ddl_envelope_plan.rs"),
        "DdlEnvelopePlanSummary propagation_boundary propagation_decisions propagation_policy_sha256 ddl_events stripped target_ddl_barrier_from_envelope_with_requirements",
    )
    .expect("write ddl envelope plan");
    fs::write(
        crate_src.join("ddl_envelope_replay.rs"),
        "DdlEnvelopeReplaySummary dml_replay_after_ddl_barrier",
    )
    .expect("write ddl envelope replay");
    fs::write(
        crate_src.join("ddl_envelope_target.rs"),
        "target_ddl_apply_plan_from_envelope",
    )
    .expect("write ddl envelope target");
    fs::write(
        crate_src.join("pilot_package_ddl_release.rs"),
        "struct DdlReleaseProof { cdc_transaction_boundary: String, ack_commands: Vec<String> }",
    )
    .expect("write pilot DDL release proof");
    fs::write(
        crate_src.join("pilot_evidence_ddl_markers.rs"),
        r#"post_ddl_release_gate_satisfied ack_commands_are_executable ack_evidence_has_valid_sink_lsns cdc_transaction_boundary "ack_commands" "ack_evidence""#,
    )
    .expect("write DDL evidence markers");
    fs::create_dir_all(crate_src.join("pilot_evidence_ddl_markers"))
        .expect("create DDL evidence marker module dir");
    fs::write(
        crate_src.join("pilot_evidence_ddl_markers").join("json.rs"),
        r#"fn post_ddl_release_gate_satisfied() {}
fn ack_commands_are_executable() {}
fn ack_evidence_has_valid_sink_lsns() { "cdc_transaction_boundary"; "ack_commands"; "ack_evidence"; }"#,
    )
    .expect("write DDL evidence marker JSON helpers");
    fs::write(
        crate_src.join("pilot_evidence_ddl_markers").join("ack.rs"),
        r#"fn command_is_executable() {}
fn command_collection_is_valid() {}
fn evidence_item_is_valid() {}
fn evidence_covers_required_sinks() {}"#,
    )
    .expect("write DDL evidence marker ACK helpers");
    fs::create_dir_all(crate_src.join("pilot_evidence_ddl_markers").join("ack"))
        .expect("create DDL evidence marker ACK module dir");
    fs::write(
        crate_src
            .join("pilot_evidence_ddl_markers")
            .join("ack")
            .join("command.rs"),
        r#"fn command_collection_is_valid() { "--partition-metadata-table" }"#,
    )
    .expect("write DDL evidence marker ACK command helpers");
    fs::write(
        crate_src
            .join("pilot_evidence_ddl_markers")
            .join("ack")
            .join("evidence.rs"),
        r#"fn evidence_item_is_valid() { "partition_metadata_table" }
fn evidence_covers_required_sinks() {}"#,
    )
    .expect("write DDL evidence marker ACK evidence helpers");
    fs::write(
        crate_src
            .join("pilot_evidence_ddl_markers")
            .join("ack")
            .join("sinks.rs"),
        r#"fn required_sink_set() {}"#,
    )
    .expect("write DDL evidence marker ACK sink helpers");

    assert!(schema_ddl_propagation_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove ddl propagation temp dir");
}

#[test]
fn schema_ddl_propagation_artifacts_reject_missing_sink_ack_contract() {
    let root = temp_root("ddl-propagation-stale");
    let crate_src = root.join("crates").join("trellara-cli").join("src");
    fs::create_dir_all(&crate_src).expect("create ddl propagation temp dir");
    fs::create_dir_all(root.join("docs")).expect("create ddl propagation docs dir");
    fs::write(
        root.join("docs").join("DESIGN.md"),
        "schema ddl-plan mentions schema barrier.",
    )
    .expect("write readme");
    fs::write(
        root.join("Makefile"),
        "quickstart-proof-check verifies schema-ddl-plan.json.",
    )
    .expect("write makefile");
    fs::write(crate_src.join("ddl.rs"), "struct DdlPropagationPlan;").expect("write cli");

    assert!(!schema_ddl_propagation_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove ddl propagation temp dir");
}
