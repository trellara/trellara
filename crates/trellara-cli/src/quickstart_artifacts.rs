use std::fs;
use std::path::Path;

pub(crate) fn strict_chunk_audit_artifacts_are_current(repository_root: &Path) -> bool {
    let readme_current = fs::read_to_string(repository_root.join("README.md"))
        .is_ok_and(|readme| readme.contains("docs/DESIGN.md") && readme.contains("strict-chunked"));
    let rfc_current =
        fs::read_to_string(repository_root.join("docs").join("DESIGN.md")).is_ok_and(|rfc| {
            rfc.contains("`boundary_mode`")
                && rfc.contains("strict_chunked_transaction_order")
                && rfc.contains("partitioned_scale_mode")
                && rfc.contains("`ddl_events`")
                && rfc.contains("source_total")
                && rfc.contains("dml_replay_after_ddl_barrier")
        });

    readme_current && rfc_current
}

pub(crate) fn partitioned_watermark_artifacts_are_current(repository_root: &Path) -> bool {
    partition_watermark_artifacts_are_current(repository_root)
        && partition_rebalance_artifacts_are_current(repository_root)
}

fn partition_watermark_artifacts_are_current(repository_root: &Path) -> bool {
    let readme_current = fs::read_to_string(repository_root.join("docs").join("DESIGN.md"))
        .is_ok_and(|readme| {
            readme.contains("partition-watermarks")
                && readme.contains("global low watermark")
                && readme.contains("complete_partition_set")
                && readme.contains("partition_scale_health")
        });
    let makefile_current =
        fs::read_to_string(repository_root.join("Makefile")).is_ok_and(|makefile| {
            makefile.contains("partition-watermarks")
                && makefile.contains("partition_watermarks")
                && makefile.contains("live-evidence")
        });
    let cli_src = repository_root
        .join("crates")
        .join("trellara-cli")
        .join("src");
    let args_current = fs::read_to_string(cli_src.join("args.rs"))
        .is_ok_and(|args| args.contains("PartitionWatermarksArgs"));
    let status_render_current = fs::read_to_string(cli_src.join("status_partition_render.rs"))
        .is_ok_and(|status| {
            status.contains("render_partition_watermark_summary")
                && status.contains("PartitionWatermarkReport::from_summary")
                && status.contains("render_partition_watermark_text")
        });
    let status_text_current = fs::read_to_string(cli_src.join("status_partition_text.rs"))
        .is_ok_and(|status| {
            status.contains("complete_partition_set: {}")
                && status.contains("partition_scale_health")
                && status.contains("global_visibility_releasable")
        });
    let pilot_current = partition_marker_sources(&cli_src).is_some_and(|markers| {
        markers.contains("complete_partition_set true")
            && markers.contains("partition_scale_health ready")
            && markers.contains("expected_partition_count")
            && markers.contains("observed_partition_count")
            && markers.contains("global_durable_lsn")
            && markers.contains("global_applied_lsn")
            && markers.contains("global_visibility_releasable")
            && markers.contains("missing_partitions")
            && markers.contains("blocks_global_applied_watermark")
    });

    readme_current
        && makefile_current
        && args_current
        && status_render_current
        && status_text_current
        && pilot_current
}

fn partition_marker_sources(cli_src: &Path) -> Option<String> {
    [
        "pilot_evidence_partition_markers.rs",
        "pilot_evidence_partition_health.rs",
        "pilot_evidence_partition_json.rs",
    ]
    .into_iter()
    .map(|file| fs::read_to_string(cli_src.join(file)).ok())
    .collect::<Option<Vec<_>>>()
    .map(|sources| sources.join("\n"))
}

fn partition_rebalance_artifacts_are_current(repository_root: &Path) -> bool {
    let cli_src = repository_root
        .join("crates")
        .join("trellara-cli")
        .join("src");
    let makefile_current =
        fs::read_to_string(repository_root.join("Makefile")).is_ok_and(|makefile| {
            makefile.contains("partition-rebalance-plan.json")
                && makefile.contains("trellara partition-rebalance-plan --config")
        });
    let catalog_current = fs::read_to_string(cli_src.join("pilot_evidence_marker_catalog.rs"))
        .is_ok_and(|catalog| {
            catalog.contains("\"partition_rebalance_plan\"")
                && catalog.contains("partition-rebalance-plan.json")
                && catalog.contains("rebalance evidence complete")
                && catalog.contains("runtime movement disabled")
                && catalog.contains("visibility contract present")
                && catalog.contains("skew metrics present")
                && catalog.contains("move candidates reviewable")
        });
    let requirements_current = fs::read_to_string(
        cli_src.join("pilot_evidence_collection_requirements.rs"),
    )
    .is_ok_and(|requirements| {
        requirements.contains("\"partition_rebalance_plan\"")
            && requirements.contains("runtime_movement_allowed=false")
            && requirements.contains("skew_ratio_basis_points")
            && requirements.contains("reviewable recommended_moves")
    });
    let marker_current = fs::read_to_string(cli_src.join("pilot_evidence_rebalance_markers.rs"))
        .is_ok_and(|markers| {
            markers.contains("PARTITION_REBALANCE_VISIBILITY_CONTRACT")
                && markers.contains("evidence_complete")
                && markers.contains("runtime_movement_allowed")
                && markers.contains("skew_ratio_basis_points")
                && markers.contains("recommended_moves")
                && markers.contains("estimated_event_delta")
        });

    makefile_current && catalog_current && requirements_current && marker_current
}

pub(crate) fn correctness_report_workflow_publishes_tested_report(repository_root: &Path) -> bool {
    fs::read_to_string(repository_root.join(".github/workflows/correctness-report.yml")).is_ok_and(
        |workflow| {
            workflow.contains("cron:")
                && workflow.contains("cargo test --workspace")
                && workflow.contains("make correctness-site")
                && workflow.contains("actions/upload-artifact")
                && workflow.contains("actions/deploy-pages")
        },
    )
}
