pub(crate) use crate::pilot_evidence_collection_requirements::live_evidence_collection_requirements;

pub(crate) fn live_evidence_artifact_name(code: &str) -> &'static str {
    match code {
        "source_safety" => "source-safety.txt",
        "contract_preflight" => "contract-test.json",
        "transaction_boundary" => "transaction-boundary.txt",
        "snapshot_handoff" => "snapshot.txt",
        "verified_apply" => "verified-apply.txt",
        "failure_drill" => "diagnostics.txt",
        "lake_writer_plan" => "lake-writer-plan.json",
        "lake_spark_consumption" => "lake-completeness.json",
        "partition_watermarks" => "partition-watermarks.json",
        "partition_rebalance_plan" => "partition-rebalance-plan.json",
        "ddl_release_proof" => "ddl-release-proof.json",
        _ => "evidence.txt",
    }
}

pub(crate) fn live_evidence_expected_markers(code: &str) -> Vec<String> {
    match code {
        "source_safety" => vec![
            "source dataset identity".to_string(),
            "Trellara source safety".to_string(),
            "no critical findings".to_string(),
            "ready_or_warning_status".to_string(),
        ],
        "contract_preflight" => vec![
            "source dataset identity".to_string(),
            "passed true".to_string(),
            "checks present".to_string(),
            "check counts consistent".to_string(),
            "no failed or error checks".to_string(),
        ],
        "transaction_boundary" => vec![
            "source dataset identity".to_string(),
            "transaction_boundary verified".to_string(),
            "source checkpoint evidence".to_string(),
            "target checkpoint evidence".to_string(),
            "source_ack_lsn evidence".to_string(),
            "source ack after durable publish".to_string(),
            "source ack publish destinations".to_string(),
            "parallel replay contract".to_string(),
            "partitioned manifest evidence headers".to_string(),
        ],
        "snapshot_handoff" => vec![
            "source dataset identity".to_string(),
            "stream_handoff_ready".to_string(),
            "selected table coverage".to_string(),
            "copy_complete".to_string(),
            "durable handoff watermark".to_string(),
        ],
        "verified_apply" => vec![
            "source dataset identity".to_string(),
            "converged true".to_string(),
            "checksum match".to_string(),
            "target relation identity".to_string(),
        ],
        "failure_drill" => vec![
            "source dataset identity".to_string(),
            "Trellara diagnostics".to_string(),
            "repair_plan_required".to_string(),
            "quarantine".to_string(),
        ],
        "lake_spark_consumption" => vec![
            "source dataset identity".to_string(),
            "verification_status match".to_string(),
            "spark_consumption_allowed true".to_string(),
            "spark consumption contract".to_string(),
            "released spark consumption gate".to_string(),
            "source ack boundary".to_string(),
            "source count agreement".to_string(),
            "checksum rollup agreement".to_string(),
        ],
        "lake_writer_plan" => vec![
            "source dataset identity".to_string(),
            "raw CDC row intents".to_string(),
            "partition source evidence".to_string(),
            "commit ordering".to_string(),
            "durability gates".to_string(),
            "duplicate replay accounting".to_string(),
            "Iceberg checkpoint receipt gate".to_string(),
            "DDL boundary metadata".to_string(),
        ],
        "partition_watermarks" => vec![
            "source dataset identity".to_string(),
            "complete_partition_set true".to_string(),
            "partition_scale_health ready".to_string(),
        ],
        "partition_rebalance_plan" => vec![
            "source dataset identity".to_string(),
            "rebalance evidence complete".to_string(),
            "runtime movement disabled".to_string(),
            "visibility contract present".to_string(),
            "skew metrics present".to_string(),
            "move candidates reviewable".to_string(),
        ],
        "ddl_release_proof" => vec![
            "source dataset identity".to_string(),
            "release_dml true".to_string(),
            "post_ddl_dml_release".to_string(),
            "cdc_transaction_boundary".to_string(),
            "propagation_boundary".to_string(),
            "propagation_decisions".to_string(),
            "propagation_policy_sha256".to_string(),
            "ack_commands".to_string(),
            "ack_evidence".to_string(),
            "release_evidence".to_string(),
            "ddl_dml_replay_proof".to_string(),
            "no release blockers".to_string(),
        ],
        _ => vec!["accepted".to_string()],
    }
}
