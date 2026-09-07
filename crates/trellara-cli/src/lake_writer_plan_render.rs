use std::fmt::Write as _;

#[path = "lake_writer_plan_epoch_render.rs"]
mod lake_writer_plan_epoch_render;

use lake_writer_plan_epoch_render::push_epoch_metadata;

pub(crate) fn render_lake_writer_plan_text(
    plan: &trellara_lake::LakeRawCdcEpochWritePlan,
) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara raw CDC writer plan").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} epoch: {}",
        plan.dataset_id, plan.epoch_id
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transactions={} changes={} duplicate_replays={} skipped_dataset={} data_files={} checksum_rollup={}",
        plan.transaction_count,
        plan.change_count,
        plan.duplicate_transaction_count,
        plan.skipped_dataset_transaction_count,
        plan.data_file_count,
        plan.checksum_rollup
    )
    .expect("write string");
    writeln!(&mut output, "visibility_rule: {}", plan.visibility_rule).expect("write string");
    writeln!(
        &mut output,
        "duplicate_replay_evidence: replay_safe={} duplicates={} unique_transactions={} row_intents={} idempotency_keys={} contract={}",
        plan.duplicate_replay_evidence.replay_safe,
        plan.duplicate_replay_evidence.duplicate_transaction_count,
        plan.duplicate_replay_evidence.unique_transaction_count,
        plan.duplicate_replay_evidence.row_intent_count,
        plan.duplicate_replay_evidence.idempotency_key_count,
        plan.duplicate_replay_evidence.contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "committer_topology: strategy={} committers={} tables={} source_buckets={} source_ack_boundary={} catalog_backpressure_rule={}",
        plan.committer_topology.strategy,
        plan.committer_topology.committer_count,
        plan.committer_topology.table_count,
        plan.committer_topology.source_bucket_count,
        plan.committer_topology.source_ack_boundary,
        plan.committer_topology.catalog_backpressure_rule
    )
    .expect("write string");
    for committer in &plan.committer_topology.table_committers {
        writeln!(
            &mut output,
            "- table_committer table={} relation={} source_buckets={} data_files={} commit_policy={}",
            committer.table_name,
            committer.relation,
            committer.source_bucket_count,
            committer.data_file_count,
            committer.commit_policy
        )
        .expect("write string");
    }
    writeln!(&mut output, "commit_steps:").expect("write string");
    for step in &plan.commit_steps {
        writeln!(
            &mut output,
            "- order={} phase={} action={} durability_gate={} recovery_rule={}",
            step.order, step.phase, step.action, step.durability_gate, step.recovery_rule
        )
        .expect("write string");
    }
    writeln!(&mut output, "recovery_scenarios:").expect("write string");
    for scenario in &plan.recovery_scenarios {
        writeln!(
            &mut output,
            "- code={} trigger={} replay_policy={} operator_evidence={} recovery_action={}",
            scenario.code,
            scenario.trigger,
            scenario.replay_policy,
            scenario.operator_evidence,
            scenario.recovery_action
        )
        .expect("write string");
    }
    writeln!(&mut output, "data_files:").expect("write string");
    for file in &plan.data_files {
        writeln!(
            &mut output,
            "- table={} relation={} source_bucket={} tx={} changes={} lsn={}..{} idempotency_keys={} object={}",
            file.table_name,
            file.relation,
            file.source_bucket,
            file.transaction_count,
            file.change_count,
            file.min_commit_lsn,
            file.max_commit_lsn,
            file.idempotency_key_count,
            file.object_key_hint
        )
        .expect("write string");
    }
    writeln!(&mut output, "row_intents: count={}", plan.row_intents.len()).expect("write string");
    for row in &plan.row_intents {
        writeln!(
            &mut output,
            "- raw_row source={} relation={} op={} tx={} order={} commit_lsn={} record_key={} partition_key={} manifest_boundary_mode={} manifest_events={} manifest_partitions={} idempotency_key={} schema_fingerprint={} schema_version={} ddl_barrier={} ddl_release_gate={} ddl_fingerprint_before={} ddl_fingerprint_after={} epoch={} before_cols={} after_cols={} checksum={}",
            row.source_id,
            row.relation,
            row.operation,
            row.transaction_id,
            row.total_order,
            row.commit_lsn,
            row.record_key.as_deref().unwrap_or("<none>"),
            row.partition_key.as_deref().unwrap_or("<none>"),
            row.manifest_boundary_mode.as_deref().unwrap_or("<none>"),
            row.manifest_global_event_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            row.manifest_participating_partition_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            row.idempotency_key,
            row.schema_fingerprint
                .map(|fingerprint| fingerprint.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            row.schema_version
                .map(|version| version.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            row.ddl_barrier_id.as_deref().unwrap_or("<none>"),
            row.ddl_release_gate.as_deref().unwrap_or("<none>"),
            row.ddl_schema_fingerprint_before
                .map(|fingerprint| fingerprint.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            row.ddl_schema_fingerprint_after
                .map(|fingerprint| fingerprint.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            row.epoch_id,
            row.payload_before.len(),
            row.payload_after.len(),
            row.envelope_checksum
        )
        .expect("write string");
    }
    push_epoch_metadata(&mut output, plan);
    output
}
