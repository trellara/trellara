use super::*;

#[test]
fn live_evidence_catalog_maps_partition_watermark_artifact_and_markers() {
    let complete = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-east",
        "expected_partition_count": 2,
        "observed_partition_count": 2,
        "complete_partition_set": true,
        "global_durable_lsn": "0/16B9000",
        "global_applied_lsn": "0/16B9000",
        "missing_partitions": [],
        "partition_scale_health": {
            "source_id": "local-source",
            "dataset_id": "retail-east",
            "expected_partition_count": 2,
            "observed_partition_count": 2,
            "status": "Ready",
            "global_watermark_available": true,
            "global_visibility_releasable": true,
            "global_durable_lsn": "0/16B9000",
            "global_applied_lsn": "0/16B9000",
            "blocking_partition_ids": [],
            "lagging_partition_ids": [],
            "straggler_partition_ids": [],
            "missing_partitions": []
        },
        "partitions": [
            {
                "partition_id": 0,
                "last_durable_lsn": "0/16B9000",
                "last_applied_lsn": "0/16B9000",
                "blocks_global_applied_watermark": false
            },
            {
                "partition_id": 1,
                "last_durable_lsn": "0/16B9000",
                "last_applied_lsn": "0/16B9000",
                "blocks_global_applied_watermark": false
            }
        ]
    }"#;

    assert_eq!(
        live_evidence_artifact_name("partition_watermarks"),
        "partition-watermarks.json"
    );
    assert_eq!(
        live_evidence_expected_markers("partition_watermarks"),
        vec![
            "source dataset identity".to_string(),
            "complete_partition_set true".to_string(),
            "partition_scale_health ready".to_string()
        ]
    );
    assert!(live_evidence_marker_present(
        "partition_watermarks",
        "source dataset identity",
        complete
    ));
    assert!(live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        complete
    ));
    assert!(live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        complete
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        r#"{"complete_partition_set":true}"#
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "source dataset identity",
        &complete.replace(
            "\"dataset_id\": \"retail-east\",",
            "\"dataset_id\": \"unknown\","
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "source dataset identity",
        &complete.replace("\"source_id\": \"local-source\",", "\"source_id\": \"\",")
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "source dataset identity",
        &complete.replace("\"dataset_id\": \"retail-east\",", "\"dataset_id\": \"\",")
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        &complete.replacen(
            "\"global_durable_lsn\": \"0/16B9000\"",
            "\"global_durable_lsn\": \"not-a-lsn\"",
            1
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        &complete.replace(
            "\"last_durable_lsn\": \"0/16B9000\"",
            "\"last_durable_lsn\": \"0/16B8000\""
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        &complete.replace("\"partition_id\": 1", "\"partition_id\": 0")
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        &complete.replace("\"partition_id\": 1", "\"partition_id\": 2")
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace("\"status\": \"Ready\"", "\"status\": \"LaggingPartitions\"")
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"global_visibility_releasable\": true",
            "\"global_visibility_releasable\": false"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"blocking_partition_ids\": []",
            "\"blocking_partition_ids\": [1]"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"source_id\": \"local-source\",\n            \"dataset_id\": \"retail-east\",\n            \"expected_partition_count\": 2,",
            "\"source_id\": \"shadow-source\",\n            \"dataset_id\": \"retail-east\",\n            \"expected_partition_count\": 2,"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"expected_partition_count\": 2,\n            \"observed_partition_count\": 2,",
            "\"expected_partition_count\": 3,\n            \"observed_partition_count\": 2,"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"observed_partition_count\": 2,\n            \"status\": \"Ready\",",
            "\"observed_partition_count\": 1,\n            \"status\": \"Ready\","
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"lagging_partition_ids\": []",
            "\"lagging_partition_ids\": [0]"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"straggler_partition_ids\": []",
            "\"straggler_partition_ids\": [1]"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"global_visibility_releasable\": true,\n            \"global_durable_lsn\": \"0/16B9000\"",
            "\"global_visibility_releasable\": true,\n            \"global_durable_lsn\": \"0/16B8000\""
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &complete.replace(
            "\"global_durable_lsn\": \"0/16B9000\",\n            \"global_applied_lsn\": \"0/16B9000\"",
            "\"global_durable_lsn\": \"0/16B9000\",\n            \"global_applied_lsn\": \"0/16B8000\""
        )
    ));
}
