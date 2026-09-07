use super::*;

#[test]
fn product_positioning_requires_readme_proof_language() {
    let root = temp_root("positioning-ready");
    fs::create_dir_all(&root).expect("create positioning temp dir");
    fs::write(
            root.join("README.md"),
            "Trellara is source-safety and verified replication for PostgreSQL fleets. See docs/DESIGN.md and docs/ROADMAP.md plus the correctness report.",
        )
        .expect("write readme");

    assert!(product_positioning_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove positioning temp dir");
}

#[test]
fn product_positioning_rejects_stale_readme_copy() {
    let root = temp_root("positioning-stale");
    fs::create_dir_all(&root).expect("create positioning temp dir");
    fs::write(
        root.join("README.md"),
        "A generic data movement platform for every destination.",
    )
    .expect("write readme");

    assert!(!product_positioning_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove positioning temp dir");
}

#[test]
fn product_positioning_rejects_readme_missing_fleet_positioning() {
    let root = temp_root("positioning-no-fleet");
    fs::create_dir_all(&root).expect("create positioning temp dir");
    fs::write(
            root.join("README.md"),
            "Trellara is source-safety and verified replication for one Postgres. See docs/DESIGN.md and docs/ROADMAP.md plus the correctness report.",
        )
        .expect("write readme");

    assert!(!product_positioning_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove positioning temp dir");
}

#[test]
fn source_safety_enterprise_artifacts_require_read_only_and_failover_proofs() {
    let root = temp_root("source-safety-enterprise-ready");
    write_enterprise_source_safety_files(&root, None);

    assert!(source_safety_enterprise_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove source safety temp dir");
}

#[test]
fn source_safety_enterprise_artifacts_reject_missing_read_only_output() {
    let root = temp_root("source-safety-enterprise-no-read-only");
    write_enterprise_source_safety_files(&root, Some(("source_safety/text.rs", "status only")));

    assert!(!source_safety_enterprise_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove source safety temp dir");
}

#[test]
fn source_safety_enterprise_artifacts_reject_missing_failover_slot_evidence() {
    let root = temp_root("source-safety-enterprise-no-failover");
    write_enterprise_source_safety_files(
        &root,
        Some(("source_safety/slot/evidence.rs", "wal_status")),
    );

    assert!(!source_safety_enterprise_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove source safety temp dir");
}

#[test]
fn source_safety_enterprise_artifacts_reject_package_without_proof_gate() {
    let root = temp_root("source-safety-enterprise-no-proof-gate");
    write_enterprise_source_safety_files(
        &root,
        Some((
            "Makefile",
            "quickstart-proof-check verifies source-safety-checklist.md",
        )),
    );

    assert!(!source_safety_enterprise_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove source safety temp dir");
}

#[test]
fn local_design_partner_artifacts_require_run_proof_surface() {
    let root = temp_root("local-proof-ready");
    fs::create_dir_all(&root).expect("create local proof temp dir");
    fs::write(
            root.join("README.md"),
            "Run make quickstart-local for a brokerless pilot package, then review docs/DESIGN.md and docs/ROADMAP.md with trellara-check evidence.",
        )
        .expect("write readme");
    fs::write(
            root.join("Makefile"),
            concat!(
                "quickstart-proof-check verifies enterprise-evaluation.txt, quickstart-readiness.txt, trellara check --config $(QUICKSTART_CONFIG) --format text, local-run-proof.md, source_ack_lsn, source_ack_publish_destinations_match, every Trellara publish ack is durable, selected_table_count, snapshot_handoff_blocker_codes, snapshot_handoff_recovery_actions, barrier_pending_blockers, barrier_pending_blocker_codes, barrier_pending_recovery_actions, source-safety-checklist.md, deployment-guide.md, operational-burden-notes.md, feature-pull-list.md, fleet-report.txt, fleet-scorecard.txt, fleet-control-plane.json, read_only_fleet_topology, deployment_orchestration, diagnostics.txt, diagnostics.json, correctness-report.html, package/correctness-report.html, Trellara correctness report, consistency-contract.json, source_ack_contract, snapshot_handoff_contract, target_checkpoint_contract, performance-envelope.json, stream_spill_threshold_changes, transaction_boundary_cost, measurement_note, identity-audit.json, schema-ddl-plan.json, schema-ddl-apply-plan.json, schema-ddl-envelope-plan.json, ddl-barrier-status.json, ddl-release-proof.json, cdc_transaction_boundary, ack_evidence, schema_ddl_plan, schema_ddl_apply_plan, schema_ddl_envelope_plan, ddl_barrier_status, release_gates, post_ddl_dml_release, release_blockers, release_blocker_codes, release_decision, blocker_codes, rejection_code, trellara schema ddl-plan --config, trellara schema ddl-barrier record --config, trellara schema ddl-barrier ack --config, trellara schema ddl-barrier status --config, ordinary_pk_tables_do_not_require_full, plans_delete_with_key_predicate, update_omits_absent_non_key_columns_for_unchanged_toast, update_omits_explicit_unchanged_toast_marker, evidence-registry.txt, verified_artifacts: 50, review_surfaces:, 16 required, 0 missing_required, package_manifest_sha256, fleet-identity-audit.txt, blocked_by_identity_collision, assign unique source.id and dataset.id values, fleet-evidence-plan.txt, ready_to_collect_live_evidence, required_live_artifacts:, transaction_boundary, transaction-boundary.txt, trellara partition-watermarks --config, live-evidence/README.md, live-evidence/collect.sh, ddl_release_proof, release_dml true, propagation_boundary, propagation_decisions, propagation_policy_sha256, ack_commands, trellara pilot evidence-template --config, trellara pilot evidence-check --config, consumer-semantics.json, \"mode\": \"exact_transaction\", \"mode\": \"partition_local\", ",
                "lake-ddl.json, lake-epoch.json, lake-verify.json, lake-completeness.json, \"completeness_state\": \"complete_with_gaps\", \"verification_status\": \"match\", \"spark_consumption_gate\": \"released: stream and lake proofs match, complete_with_gaps was explicitly accepted, and verification_status=match\", lake-writer-plan.json, sample-envelope.pb, lake-fanin-run.json, recovery_guidance, explicit_gap_acceptance_required, operator_action, \"committer_topology\", \"strategy\": \"single_table_committer_epoch_batched_append_only_raw_cdc\", source acknowledgement advances after durable Trellara stream publish, source WAL remains protected, \"commit_steps\", \"phase\": \"raw_cdc_data\", \"phase\": \"epoch_row_metadata\", \"phase\": \"verification_metadata\", \"recovery_scenarios\", \"code\": \"writer_crash_after_epoch_metadata\", \"replay_policy\": \"discover_existing_epoch_metadata_before_publish\", \"code\": \"verification_mismatch_after_commit\", \"replay_policy\": \"hold_spark_consumption_until_fanin_verify_matches\", before source acknowledgement, ",
                "spark-current-state.sql, spark-current-state.py, spark-scd2.sql, spark-scd2.py, spark-maintenance.sql, spark-maintenance.py, spark-completeness-dashboard.sql, spark-completeness-dashboard.py, spark-golden-fixture.json, trellara lake epoch --config, trellara lake fanin verify --config, --accept-complete-with-gaps, trellara lake writer-plan --config, trellara lake spark-template current-state --config, trellara lake spark-template scd2 --config, trellara lake spark-template maintenance --config, trellara lake spark-template dashboard --config, retail_sales__public__sales__raw_cdc, _trellara_epochs, _trellara_epoch_sources, _trellara_verification, retail_sales__spark__derived__current_state, retail_sales__spark__derived__scd2_history, Trellara Local Run Proof, Trellara Source Safety Checklist, Trellara fleet report, Trellara fleet scorecard, and \"artifact_count\": 51.",
            ),
        )
        .expect("write makefile");

    assert!(local_design_partner_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove local proof temp dir");
}

#[test]
fn local_design_partner_artifacts_reject_missing_run_proof_manifest_check() {
    let root = temp_root("local-proof-stale");
    fs::create_dir_all(&root).expect("create local proof temp dir");
    fs::write(
            root.join("README.md"),
            "Run trellara pilot-package, review enterprise-evaluation.txt and local-run-proof.md, and keep trellara run --local --verify --format text evidence.",
        )
        .expect("write readme");
    fs::write(
        root.join("Makefile"),
        "quickstart-proof-check verifies the package manifest.",
    )
    .expect("write makefile");

    assert!(!local_design_partner_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove local proof temp dir");
}

fn write_enterprise_source_safety_files(root: &std::path::Path, replacement: Option<(&str, &str)>) {
    let src = root.join("crates").join("trellara-cli").join("src");
    fs::create_dir_all(&src).expect("create source safety temp src dir");
    fs::create_dir_all(root.join("docs")).expect("create source safety temp docs dir");

    write_root_file(
        root,
        "docs/DESIGN.md",
        "read-only source-safety checklist trellara-check failover-slot evidence trellara-check <source-database-url> source-safety.html source-safety-checklist.md",
        replacement,
    );
    write_root_file(
        root,
        "Makefile",
        "source-safety-checklist.md trellara-check <source-database-url> trellara check --config trellara-check failover slot wal_status replica identity",
        replacement,
    );
    write_src_file(
        &src,
        "pilot_package_source_safety.rs",
        "## Read-Only Commands trellara-check <source-database-url> trellara check --config trellara check --database-url <source-database-url> --table <schema.table> --format html source-safety.html --write-init slot `wal_status` `safe_wal_size` `failover`, `synced` `idle_replication_slot_timeout` transaction ID wraparound budget xmin horizon pinners Do not start capture against a customer source",
        replacement,
    );
    write_src_file(
        &src,
        "source_safety/text.rs",
        "read_only: {read_only}",
        replacement,
    );
    write_src_file(
        &src,
        "source_safety/render.rs",
        "read_only: Some(summary.read_only) read_only: None",
        replacement,
    );
    write_src_file(
        &src,
        "source_safety/html/queries.rs",
        "Exact Read-Only Queries source_safety_read_only_queries",
        replacement,
    );
    write_src_file(
        &src,
        "source_safety/direct.rs",
        "read_only: true",
        replacement,
    );
    write_src_file(
        &src,
        "source_safety/slot/evidence.rs",
        "restart_lsn confirmed_flush_lsn wal_status safe_wal_size_bytes retained_wal_bytes invalidation_reason \"failover\" \"synced\" inactive_since idle_replication_slot_timeout",
        replacement,
    );
}

fn write_root_file(
    root: &std::path::Path,
    file_name: &str,
    contents: &str,
    replacement: Option<(&str, &str)>,
) {
    let contents = replacement_contents(file_name, contents, replacement);
    fs::write(root.join(file_name), contents).expect("write source safety root file");
}

fn write_src_file(
    src: &std::path::Path,
    file_name: &str,
    contents: &str,
    replacement: Option<(&str, &str)>,
) {
    let contents = replacement_contents(file_name, contents, replacement);
    let path = src.join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create source safety src parent dir");
    }
    fs::write(path, contents).expect("write source safety src file");
}

fn replacement_contents<'a>(
    file_name: &str,
    contents: &'a str,
    replacement: Option<(&'a str, &'a str)>,
) -> &'a str {
    match replacement {
        Some((replacement_file, replacement_contents)) if replacement_file == file_name => {
            replacement_contents
        }
        _ => contents,
    }
}
