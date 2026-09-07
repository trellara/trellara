pub(crate) use crate::pilot_evidence_marker_catalog::{
    live_evidence_artifact_name, live_evidence_collection_requirements,
    live_evidence_expected_markers,
};
use crate::{
    contract_preflight_marker_present, ddl_release_marker_present, failure_drill_marker_present,
    lake_spark_marker_present, lake_writer_marker_present, partition_rebalance_marker_present,
    partition_watermark_marker_present, snapshot_handoff_marker_present,
    source_safety_marker_present, transaction_boundary_marker_present,
    verified_apply_marker_present,
};

pub(crate) fn live_evidence_markers(code: &str, contents: &str) -> (Vec<String>, Vec<String>) {
    let expected = live_evidence_expected_markers(code);
    let mut observed = Vec::new();
    let mut missing = Vec::new();

    for marker in expected {
        if live_evidence_marker_present(code, &marker, contents) {
            observed.push(marker);
        } else {
            missing.push(marker);
        }
    }

    (observed, missing)
}

pub(crate) fn live_evidence_marker_present(code: &str, marker: &str, contents: &str) -> bool {
    if code == "ddl_release_proof" {
        return ddl_release_marker_present(marker, contents);
    }
    if code == "lake_spark_consumption" {
        return lake_spark_marker_present(marker, contents);
    }
    if code == "lake_writer_plan" {
        return lake_writer_marker_present(marker, contents);
    }
    if code == "partition_watermarks" {
        return partition_watermark_marker_present(marker, contents);
    }
    if code == "partition_rebalance_plan" {
        return partition_rebalance_marker_present(marker, contents);
    }
    if code == "transaction_boundary" {
        return transaction_boundary_marker_present(marker, contents);
    }
    if code == "verified_apply" {
        return verified_apply_marker_present(marker, contents);
    }
    if code == "source_safety" {
        return source_safety_marker_present(marker, contents);
    }
    if code == "contract_preflight" {
        return contract_preflight_marker_present(marker, contents);
    }
    if code == "snapshot_handoff" {
        return snapshot_handoff_marker_present(marker, contents);
    }
    if code == "failure_drill" {
        return failure_drill_marker_present(marker, contents);
    }

    contents
        .to_ascii_lowercase()
        .contains(&marker.to_ascii_lowercase())
}
