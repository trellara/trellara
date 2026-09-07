use super::*;

fn complete_partition_watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 2,
        observed_partition_count: 2,
        complete_partition_set: true,
        global_durable_lsn: Some("0/16B9000".to_string()),
        global_applied_lsn: Some("0/16B9000".to_string()),
        global_durable_to_applied_bytes: Some(0),
        missing_partitions: Vec::new(),
        partitions: vec![
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 0,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B9000".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 1,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B9000".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
        ],
    }
}

fn lagging_partition_watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 3,
        observed_partition_count: 3,
        complete_partition_set: true,
        global_durable_lsn: Some("0/16B8800".to_string()),
        global_applied_lsn: Some("0/16B7000".to_string()),
        global_durable_to_applied_bytes: Some(8192),
        missing_partitions: Vec::new(),
        partitions: vec![
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 0,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B9000".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 1,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B7000".to_string(),
                durable_to_applied_bytes: 8192,
                blocks_global_applied_watermark: true,
            },
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 2,
                last_durable_lsn: "0/16B8800".to_string(),
                last_applied_lsn: "0/16B8800".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
        ],
    }
}

fn uniform_lag_partition_watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 2,
        observed_partition_count: 2,
        complete_partition_set: true,
        global_durable_lsn: Some("0/16B9000".to_string()),
        global_applied_lsn: Some("0/16B8000".to_string()),
        global_durable_to_applied_bytes: Some(4096),
        missing_partitions: Vec::new(),
        partitions: vec![
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 0,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B8000".to_string(),
                durable_to_applied_bytes: 4096,
                blocks_global_applied_watermark: false,
            },
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 1,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B8000".to_string(),
                durable_to_applied_bytes: 4096,
                blocks_global_applied_watermark: false,
            },
        ],
    }
}

#[test]
fn partition_watermark_text_output_contains_live_evidence_marker() {
    let output = render_partition_watermark_summary(
        &complete_partition_watermarks(),
        PilotGuideOutputFormat::Text,
    )
    .expect("render partition watermark text");

    assert!(output.contains("Trellara partition watermarks"));
    assert!(output.contains("complete_partition_set: true"));
    assert!(output.contains("missing_partitions: none"));
    assert!(output.contains("partition_scale_health: ready"));
    assert!(output.contains("blocking_partitions: none"));
    assert!(output.contains("global_visibility_releasable: true"));
    assert!(output.contains("global_visibility_blocker_codes: none"));
    assert!(output.contains("global_durable_low_watermark_partitions: 0,1"));
    assert!(output.contains("global_applied_low_watermark_partitions: 0,1"));
    assert!(output.contains("lagging_partitions: none"));
    assert!(output.contains("straggler_partitions: none"));
    assert!(output.contains("observed_applied_skew_bytes: 0"));
    assert!(output.contains("max_partition_lag_bytes: 0"));
    assert!(output.contains("visibility_actions: 0"));
}

#[test]
fn partition_watermark_json_output_preserves_default_contract() {
    let output = render_partition_watermark_summary(
        &complete_partition_watermarks(),
        PilotGuideOutputFormat::Json,
    )
    .expect("render partition watermark json");

    assert!(output.contains("\"complete_partition_set\": true"));
    assert!(output.contains("\"partition_scale_health\""));
    assert!(output.contains("\"status\": \"Ready\""));
    assert!(output.contains("\"global_visibility_releasable\": true"));
    assert!(output.contains("\"global_visibility_blocker_codes\": []"));
    assert!(output.contains("\"lagging_partition_ids\": []"));
    assert!(output.contains("\"straggler_partition_ids\": []"));
    assert!(output.contains("\"visibility_actions\": []"));
    assert!(live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        &output
    ));
}

#[test]
fn partition_watermark_text_output_names_partition_scale_blockers() {
    let output = render_partition_watermark_summary(
        &lagging_partition_watermarks(),
        PilotGuideOutputFormat::Text,
    )
    .expect("render partition watermark text");

    assert!(output.contains("partition_scale_health: lagging_partitions"));
    assert!(output.contains("blocking_partitions: 1"));
    assert!(output.contains("global_visibility_releasable: false"));
    assert!(output.contains(
        "global_visibility_blocker_codes: global_applied_low_watermark_blocked,partition_durable_apply_lag"
    ));
    assert!(output.contains("global_durable_low_watermark_partitions: 2"));
    assert!(output.contains("global_applied_low_watermark_partitions: 1"));
    assert!(output.contains("lagging_partitions: 1"));
    assert!(output.contains("straggler_partitions: 1,2"));
    assert!(output.contains("observed_applied_skew_bytes: 8192"));
    assert!(output.contains("max_partition_lag_bytes: 8192"));
    assert!(output.contains("visibility_actions: 2"));
    assert!(output.contains(
        "action: advance_global_applied_low_watermark partitions: 1 command: trellara apply --config <flow>"
    ));
    assert!(output.contains(
        "action: drain_lagging_partition_lanes partitions: 1,2 command: trellara apply --config <flow>"
    ));
}

#[test]
fn partition_watermark_text_output_names_uniform_lag_without_blockers() {
    let output = render_partition_watermark_summary(
        &uniform_lag_partition_watermarks(),
        PilotGuideOutputFormat::Text,
    )
    .expect("render partition watermark text");

    assert!(output.contains("partition_scale_health: lagging_partitions"));
    assert!(output.contains("blocking_partitions: none"));
    assert!(output.contains("global_visibility_releasable: false"));
    assert!(output.contains("global_visibility_blocker_codes: partition_durable_apply_lag"));
    assert!(output.contains("global_durable_low_watermark_partitions: 0,1"));
    assert!(output.contains("global_applied_low_watermark_partitions: 0,1"));
    assert!(output.contains("lagging_partitions: 0,1"));
    assert!(output.contains("straggler_partitions: none"));
    assert!(output.contains("observed_applied_skew_bytes: 0"));
    assert!(output.contains("max_partition_lag_bytes: 4096"));
    assert!(output.contains("visibility_actions: 1"));
    assert!(output.contains(
        "action: drain_lagging_partition_lanes partitions: 0,1 command: trellara apply --config <flow>"
    ));
}

#[test]
fn partition_watermark_live_evidence_rejects_incomplete_or_lagging_json() {
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        r#"{
            "expected_partition_count": 2,
            "observed_partition_count": 1,
            "complete_partition_set": true,
            "global_durable_lsn": "0/16B9000",
            "global_applied_lsn": "0/16B9000",
            "missing_partitions": [],
            "partitions": []
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        r#"{
            "expected_partition_count": 2,
            "observed_partition_count": 2,
            "complete_partition_set": true,
            "global_durable_lsn": "0/16B9000",
            "global_applied_lsn": "0/16B8000",
            "missing_partitions": [],
            "partitions": [
                {
                    "partition_id": 0,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B8000",
                    "durable_to_applied_bytes": 4096,
                    "blocks_global_applied_watermark": true
                },
                {
                    "partition_id": 1,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "durable_to_applied_bytes": 0,
                    "blocks_global_applied_watermark": false
                }
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "complete_partition_set true",
        "complete_partition_set: true"
    ));
}

#[test]
fn partition_watermark_live_evidence_rejects_uniform_lag_as_ready() {
    let output = render_partition_watermark_summary(
        &uniform_lag_partition_watermarks(),
        PilotGuideOutputFormat::Json,
    )
    .expect("render partition watermark json");

    assert!(!live_evidence_marker_present(
        "partition_watermarks",
        "partition_scale_health ready",
        &output
    ));
}
