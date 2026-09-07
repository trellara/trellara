use super::*;
use assertions::{
    assert_commit_steps, assert_data_file, assert_epoch_metadata, assert_first_row_intent,
    assert_recovery_scenarios,
};

#[test]
fn raw_cdc_epoch_writer_groups_files_and_epoch_metadata() {
    let first = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )],
    );
    let second = envelope_for(
        "store-002",
        "tx-2",
        "0/16B6C70",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-2", "20", "2026-01-01")),
        )],
    );

    let plan = plan_raw_cdc_epoch_writes(
        &raw_writer_config(1),
        &config(),
        &[first.clone(), second.clone()],
    )
    .expect("raw CDC writer plan");

    assert_eq!(plan.dataset_id, "retail");
    assert_eq!(plan.epoch_id, "epoch-2026-08-16T06");
    assert_eq!(plan.transaction_count, 2);
    assert_eq!(plan.change_count, 2);
    assert_eq!(plan.duplicate_transaction_count, 0);
    assert_eq!(plan.skipped_dataset_transaction_count, 0);
    assert_eq!(plan.data_file_count, 1);
    assert_eq!(plan.checksum_rollup, first.checksum ^ second.checksum);
    assert_eq!(
        plan.duplicate_replay_evidence.duplicate_transaction_count,
        0
    );
    assert_eq!(plan.duplicate_replay_evidence.unique_transaction_count, 2);
    assert_eq!(plan.duplicate_replay_evidence.idempotency_key_count, 2);
    assert!(plan.duplicate_replay_evidence.replay_safe);
    assert!(plan.visibility_rule.contains("publish epoch metadata"));
    assert_eq!(
        plan.committer_topology.strategy,
        "single_table_committer_epoch_batched_append_only_raw_cdc"
    );
    assert_eq!(plan.committer_topology.committer_count, 1);
    assert_eq!(plan.committer_topology.table_count, 1);
    assert_eq!(plan.committer_topology.source_bucket_count, 1);
    assert!(plan
        .committer_topology
        .source_ack_boundary
        .contains("durable Trellara stream publish"));
    assert!(plan
        .committer_topology
        .catalog_backpressure_rule
        .contains("source WAL remains protected"));
    assert_eq!(plan.committer_topology.table_committers.len(), 1);
    assert_eq!(
        plan.committer_topology.table_committers[0].table_name,
        "retail__public__sales__raw_cdc"
    );
    assert_eq!(
        plan.committer_topology.table_committers[0].data_file_count,
        1
    );
    assert!(plan.committer_topology.table_committers[0]
        .commit_policy
        .contains("one committer owns this Iceberg table"));
    assert_commit_steps(&plan);
    assert_recovery_scenarios(&plan);
    assert_data_file(&plan, &first, &second);
    assert_first_row_intent(&plan, &first);
    assert_epoch_metadata(&plan, first.checksum ^ second.checksum);
}
