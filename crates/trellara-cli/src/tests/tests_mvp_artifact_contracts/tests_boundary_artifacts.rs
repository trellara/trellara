use super::*;

#[test]
fn strict_chunk_audit_artifacts_require_readme_and_rfc_boundary_modes() {
    let root = temp_root("strict-chunk-audit-ready");
    let docs = root.join("docs");
    fs::create_dir_all(&docs).expect("create strict chunk temp dir");
    fs::write(
        root.join("README.md"),
        "The strict-chunked contract is documented in docs/DESIGN.md.",
    )
    .expect("write readme");
    fs::write(
        docs.join("DESIGN.md"),
        "`boundary_mode` documents strict_chunked_transaction_order and partitioned_scale_mode with `ddl_events`, source_total, and dml_replay_after_ddl_barrier.",
    )
    .expect("write rfc");

    assert!(strict_chunk_audit_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove strict chunk temp dir");
}

#[test]
fn strict_chunk_audit_artifacts_reject_generic_manifest_docs() {
    let root = temp_root("strict-chunk-audit-stale");
    let docs = root.join("docs");
    fs::create_dir_all(&docs).expect("create strict chunk temp dir");
    fs::write(
        root.join("README.md"),
        "The strict-chunked contract is documented in docs/DESIGN.md.",
    )
    .expect("write readme");
    fs::write(
        docs.join("DESIGN.md"),
        "`boundary_mode` documents partitioned_scale_mode.",
    )
    .expect("write rfc");

    assert!(!strict_chunk_audit_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove strict chunk temp dir");
}

#[test]
fn partitioned_watermark_artifacts_require_marker_and_formatter_contract() {
    let root = temp_root("partition-watermark-ready");
    let cli_src = root.join("crates").join("trellara-cli").join("src");
    fs::create_dir_all(&cli_src).expect("create partition watermark temp dir");
    fs::create_dir_all(root.join("docs")).expect("create partition watermark docs dir");
    fs::write(
        root.join("docs").join("DESIGN.md"),
        "partition-watermarks --format text reports complete_partition_set, partition_scale_health, global_visibility_releasable, and computes the global low watermark.",
    )
    .expect("write readme");
    fs::write(
        root.join("Makefile"),
        "quickstart-proof-check verifies live-evidence partition-watermarks, partition_watermarks, partition-rebalance-plan.json, and trellara partition-rebalance-plan --config artifacts.",
    )
    .expect("write makefile");
    fs::write(cli_src.join("args.rs"), "struct PartitionWatermarksArgs;").expect("write cli args");
    fs::write(
        cli_src.join("status_partition_render.rs"),
        "fn render_partition_watermark_summary() { PartitionWatermarkReport::from_summary(); render_partition_watermark_text(); }",
    )
    .expect("write cli status");
    fs::write(
        cli_src.join("status_partition_text.rs"),
        "fn render_partition_watermark_text() { println!(\"complete_partition_set: {}\"); println!(\"partition_scale_health\"); println!(\"global_visibility_releasable\"); }",
    )
    .expect("write cli status text");
    fs::write(
        cli_src.join("pilot_evidence_partition_markers.rs"),
        "const MARKER: &str = \"complete_partition_set true partition_scale_health ready\"; expected_partition_count observed_partition_count global_durable_lsn global_applied_lsn missing_partitions blocks_global_applied_watermark",
    )
    .expect("write cli pilot");
    fs::write(
        cli_src.join("pilot_evidence_partition_health.rs"),
        "partition_scale_health global_visibility_releasable expected_partition_count observed_partition_count global_durable_lsn global_applied_lsn missing_partitions",
    )
    .expect("write partition health pilot");
    fs::write(
        cli_src.join("pilot_evidence_partition_json.rs"),
        "global_durable_lsn global_applied_lsn",
    )
    .expect("write partition json pilot");
    fs::write(
        cli_src.join("pilot_evidence_marker_catalog.rs"),
        "\"partition_rebalance_plan\" partition-rebalance-plan.json rebalance evidence complete runtime movement disabled visibility contract present skew metrics present move candidates reviewable",
    )
    .expect("write marker catalog");
    fs::write(
        cli_src.join("pilot_evidence_collection_requirements.rs"),
        "\"partition_rebalance_plan\" runtime_movement_allowed=false skew_ratio_basis_points reviewable recommended_moves",
    )
    .expect("write collection requirements");
    fs::write(
        cli_src.join("pilot_evidence_rebalance_markers.rs"),
        "PARTITION_REBALANCE_VISIBILITY_CONTRACT evidence_complete runtime_movement_allowed skew_ratio_basis_points recommended_moves estimated_event_delta",
    )
    .expect("write rebalance markers");

    assert!(partitioned_watermark_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove partition watermark temp dir");
}

#[test]
fn partitioned_watermark_artifacts_reject_stale_marker_docs() {
    let root = temp_root("partition-watermark-stale");
    let cli_src = root.join("crates").join("trellara-cli").join("src");
    fs::create_dir_all(&cli_src).expect("create partition watermark temp dir");
    fs::create_dir_all(root.join("docs")).expect("create partition watermark docs dir");
    fs::write(
        root.join("docs").join("DESIGN.md"),
        "partition-watermarks reports partition lag.",
    )
    .expect("write readme");
    fs::write(
        root.join("Makefile"),
        "quickstart-proof-check verifies partition-watermarks.",
    )
    .expect("write makefile");
    fs::write(cli_src.join("args.rs"), "struct PartitionWatermarksArgs;").expect("write cli args");
    fs::write(
        cli_src.join("status_partition_render.rs"),
        "fn render_partition_watermark_summary() {}",
    )
    .expect("write cli status");
    fs::write(cli_src.join("status_partition_text.rs"), "").expect("write cli status text");
    fs::write(cli_src.join("pilot.rs"), "").expect("write cli pilot");

    assert!(!partitioned_watermark_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove partition watermark temp dir");
}

#[test]
fn partitioned_watermark_artifacts_require_rebalance_surface() {
    let root = temp_root("partition-watermark-without-rebalance");
    let cli_src = root.join("crates").join("trellara-cli").join("src");
    fs::create_dir_all(&cli_src).expect("create partition watermark temp dir");
    fs::write(
        root.join("README.md"),
        "partition-watermarks --format text reports complete_partition_set, partition_scale_health, global_visibility_releasable, and computes the global low watermark.",
    )
    .expect("write readme");
    fs::write(
        root.join("Makefile"),
        "quickstart-proof-check verifies live-evidence partition-watermarks and partition_watermarks artifacts.",
    )
    .expect("write makefile");
    fs::write(cli_src.join("args.rs"), "struct PartitionWatermarksArgs;").expect("write cli args");
    fs::write(
        cli_src.join("status_partition_render.rs"),
        "fn render_partition_watermark_summary() { PartitionWatermarkReport::from_summary(); render_partition_watermark_text(); }",
    )
    .expect("write cli status");
    fs::write(
        cli_src.join("status_partition_text.rs"),
        "fn render_partition_watermark_text() { println!(\"complete_partition_set: {}\"); println!(\"partition_scale_health\"); println!(\"global_visibility_releasable\"); }",
    )
    .expect("write cli status text");
    fs::write(
        cli_src.join("pilot_evidence_partition_markers.rs"),
        "const MARKER: &str = \"complete_partition_set true partition_scale_health ready\"; expected_partition_count observed_partition_count global_durable_lsn global_applied_lsn global_visibility_releasable missing_partitions blocks_global_applied_watermark",
    )
    .expect("write cli pilot");

    assert!(!partitioned_watermark_artifacts_are_current(&root));

    fs::remove_dir_all(root).expect("remove partition watermark temp dir");
}
