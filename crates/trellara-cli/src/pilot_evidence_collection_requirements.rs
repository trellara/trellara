pub(crate) fn live_evidence_collection_requirements(code: &str) -> Vec<String> {
    match code {
        "source_safety" => vec![
            "include source_id, dataset_id, and an acceptable status from source-safety".to_string(),
            "include findings context proving there are no critical source-safety factors"
                .to_string(),
        ],
        "contract_preflight" => vec![
            "collect the structured contract-test JSON artifact, not a one-line pass/fail note"
                .to_string(),
            "include source_id and dataset_id for the contract-test proof".to_string(),
            "include a non-empty checks array with check_count and issue_count matching the checks"
                .to_string(),
            "all checks must pass and no check may carry error or critical severity".to_string(),
        ],
        "transaction_boundary" => vec![
            "include source_id and dataset_id for the transaction-boundary proof".to_string(),
            "include concrete source and target checkpoint LSN evidence where target applied LSN reaches the source durable LSN".to_string(),
            "include source_ack_lsn and source_ack_durability_proof showing every local publish ACK is proven with crash_safe_ack=true before source acknowledgement"
                .to_string(),
            "include source_ack_publish_destinations_match=true and a positive publish destination count"
                .to_string(),
            "include the parallel replay contract proving partition workers only replay DML-only committed transactions and route DDL-bearing transactions through the barrier path".to_string(),
            "for partitioned replay, include manifest checksum, event-count coverage, participating partition IDs, and the global visibility contract from the stream headers".to_string(),
        ],
        "snapshot_handoff" => vec![
            "include the structured snapshot summary with source_id, dataset_id, run_id, slot, and consistent_lsn"
                .to_string(),
            "include selected_table_count equal to table_count and the number of completed table entries"
                .to_string(),
            "every selected table must be copy_complete at the same non-empty handoff watermark"
                .to_string(),
        ],
        "verified_apply" => vec![
            "include source_id and dataset_id for the verified apply run".to_string(),
            "include source and target watermark LSNs".to_string(),
            "include non-empty table-level convergence evidence with checksum_status=match"
                .to_string(),
            "include target_relation and relation_match=true for every verified table".to_string(),
        ],
        "failure_drill" => vec![
            "include diagnostics identity fields, readiness/status, and recommended actions"
                .to_string(),
            "include repair-plan state plus quarantine list or replay-ready command surface"
                .to_string(),
        ],
        "lake_spark_consumption" => vec![
            "include dataset_id and at least one source_rows entry with source_id".to_string(),
            "include lake verification_status=match and spark_consumption_allowed=true".to_string(),
            "include the spark_consumption_contract, a released Spark consumption gate, and source acknowledgement boundary"
                .to_string(),
            "include source_counts_match=true with matching stream and lake source counters"
                .to_string(),
            "include checksum_rollup_match=true with matching stream and lake checksum rollups"
                .to_string(),
        ],
        "lake_writer_plan" => vec![
            "include dataset_id and raw CDC row_intents with source_id, relation, transaction_id, total_order, and idempotency_key".to_string(),
            "include epoch_metadata source_rows and partition_rows proving every partition source_id has matching source-row evidence".to_string(),
            "include ordered commit_steps for raw_cdc_data, epoch_row_metadata, and verification_metadata before source acknowledgement".to_string(),
            "include duplicate_transaction_count plus replay policy evidence for idempotent writer recovery".to_string(),
            "include Iceberg checkpoint receipt evidence proving epoch metadata is gated on durable catalog receipts".to_string(),
            "include DDL boundary metadata on row_intents: schema_version, ddl_barrier_id, post_ddl_dml_release, and before/after schema fingerprints".to_string(),
        ],
        "partition_watermarks" => vec![
            "include source_id, dataset_id, complete expected and observed partition counts, and no missing partitions"
                .to_string(),
            "include global durable/applied LSNs and per-partition watermarks that do not block global visibility"
                .to_string(),
            "include partition_scale_health with status=Ready, global_watermark_available=true, global_visibility_releasable=true, no blocking partitions, and no missing partitions"
                .to_string(),
        ],
        "partition_rebalance_plan" => vec![
            "include source_id and dataset_id for the rebalance plan".to_string(),
            "include evidence_complete=true, no missing partitions, and no blocking partition IDs"
                .to_string(),
            "include runtime_movement_allowed=false and a visibility contract proving movement waits for reviewed cutover evidence"
                .to_string(),
            "include total_event_count, max_skew_percent, and skew_ratio_basis_points so skew severity is reviewable across flows"
                .to_string(),
            "include reviewable recommended_moves entries with from/to partitions, estimated_event_delta, and reason when the plan is skewed"
                .to_string(),
        ],
        "ddl_release_proof" => vec![
            "include source_id and dataset_id for the DDL release proof".to_string(),
            "include release_dml=true, post_ddl_dml_release gate satisfaction, and no release blockers"
                .to_string(),
            "include the cdc_transaction_boundary proving post-DDL DML is held until sink acknowledgement"
                .to_string(),
            "include DDL propagation policy evidence with propagation_boundary, propagation_decisions, target ACK requirements, and propagation_policy_sha256"
                .to_string(),
            "include DDL sink ack_commands and ack_evidence with source_ack_lsn values and matching schema_version".to_string(),
            "include ddl_dml_replay_proof showing target ACK LSN and DML commit LSN both match the DDL barrier LSN".to_string(),
            "include non-empty release_evidence summarizing the barrier, ACK count, and release decision"
                .to_string(),
        ],
        _ => {
            vec!["include enough context for the gate-specific validator to accept the artifact".to_string()]
        }
    }
}
