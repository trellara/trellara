use super::*;

#[test]
fn lake_writer_evidence_accepts_partition_rows_with_source_evidence() {
    assert!(live_evidence_marker_present(
        "lake_writer_plan",
        "partition source evidence",
        r#"{
            "epoch_metadata": {
                "source_rows": [
                    {"source_id": "store-001"},
                    {"source_id": "store-002"}
                ],
                "partition_rows": [
                    {"source_id": "store-001", "partition_id": 0},
                    {"source_id": "store-002", "partition_id": 1}
                ]
            }
        }"#
    ));
}

#[test]
fn lake_writer_evidence_rejects_partition_rows_without_source_evidence() {
    assert!(!live_evidence_marker_present(
        "lake_writer_plan",
        "partition source evidence",
        r#"{
            "epoch_metadata": {
                "source_rows": [
                    {"source_id": "store-001"}
                ],
                "partition_rows": [
                    {"source_id": "store-001", "partition_id": 0},
                    {"source_id": "store-002", "partition_id": 1}
                ]
            }
        }"#
    ));
}
