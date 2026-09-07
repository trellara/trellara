use super::*;

#[test]
fn raw_cdc_epoch_writer_deduplicates_replayed_transactions() {
    let envelope = envelope_for(
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

    let plan = plan_raw_cdc_epoch_writes(
        &raw_writer_config(8),
        &config(),
        &[envelope.clone(), envelope.clone()],
    )
    .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 1);
    assert_eq!(plan.change_count, 1);
    assert_eq!(plan.duplicate_transaction_count, 1);
    assert_eq!(
        plan.duplicate_replay_evidence.duplicate_transaction_count,
        1
    );
    assert_eq!(plan.duplicate_replay_evidence.unique_transaction_count, 1);
    assert_eq!(plan.duplicate_replay_evidence.row_intent_count, 1);
    assert_eq!(plan.duplicate_replay_evidence.idempotency_key_count, 1);
    assert!(plan.duplicate_replay_evidence.replay_safe);
    assert!(plan
        .duplicate_replay_evidence
        .contract
        .contains("conflicting idempotency evidence fails closed"));
    assert_eq!(
        plan.data_files
            .iter()
            .map(|file| file.change_count)
            .sum::<usize>(),
        1
    );
    assert_eq!(plan.epoch_metadata.source_rows[0].transaction_count, 1);
    assert_eq!(plan.epoch_metadata.table_rows[0].transaction_count, 1);
}

#[test]
fn raw_cdc_epoch_writer_skips_other_datasets() {
    let included = envelope_for(
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
    let skipped = envelope_for_other_dataset(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-2", "20", "2026-01-01")),
    )]);

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[included, skipped])
        .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 1);
    assert_eq!(plan.change_count, 1);
    assert_eq!(plan.skipped_dataset_transaction_count, 1);
    assert_eq!(plan.data_file_count, 1);
}

#[test]
fn raw_cdc_epoch_writer_skips_other_dataset_before_operation_validation() {
    let included = envelope_for(
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
    let mut skipped = envelope_for_other_dataset(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-2", "20", "2026-01-01")),
    )]);
    skipped.changes[0].operation = 99;
    skipped.finalize_checksum();

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[included, skipped])
        .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 1);
    assert_eq!(plan.skipped_dataset_transaction_count, 1);
    assert_eq!(plan.data_file_count, 1);
}

#[test]
fn raw_cdc_epoch_writer_skips_other_dataset_before_value_kind_validation() {
    let included = envelope_for(
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
    let mut skipped = envelope_for_other_dataset(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-2", "20", "2026-01-01")),
    )]);
    skipped.changes[0]
        .after
        .as_mut()
        .expect("after image")
        .columns[1]
        .value_kind = 99;
    skipped.finalize_checksum();

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[included, skipped])
        .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 1);
    assert_eq!(plan.skipped_dataset_transaction_count, 1);
    assert_eq!(plan.data_file_count, 1);
}

#[test]
fn raw_cdc_epoch_writer_skips_sources_outside_required_scope() {
    let included = envelope_for(
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
    let unexpected = envelope_for(
        "store-999",
        "tx-2",
        "0/16B6D00",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-999", "999", "2026-01-01")),
        )],
    );
    let writer_config = raw_writer_config(4).with_required_sources(["store-001"]);

    let plan = plan_raw_cdc_epoch_writes(&writer_config, &config(), &[included, unexpected])
        .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 1);
    assert_eq!(plan.change_count, 1);
    assert_eq!(plan.data_file_count, 1);
    assert_eq!(plan.row_intents.len(), 1);
    assert_eq!(plan.data_files[0].source_ids, vec!["store-001"]);
    assert_eq!(plan.epoch_metadata.epoch_row.required_source_count, 1);
    assert_eq!(plan.epoch_metadata.epoch_row.complete_source_count, 1);
    assert_eq!(plan.epoch_metadata.source_rows.len(), 1);
    assert_eq!(plan.epoch_metadata.source_rows[0].source_id, "store-001");
}

#[test]
fn raw_cdc_epoch_writer_skips_unscoped_source_before_validation() {
    let included = envelope_for(
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
    let mut unexpected = envelope_for(
        "store-999",
        "tx-2",
        "0/16B6D00",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-999", "999", "2026-01-01")),
        )],
    );
    unexpected.changes[0].operation = 99;
    unexpected.finalize_checksum();
    let writer_config = raw_writer_config(4).with_required_sources(["store-001"]);

    let plan = plan_raw_cdc_epoch_writes(&writer_config, &config(), &[included, unexpected])
        .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 1);
    assert_eq!(plan.change_count, 1);
    assert_eq!(plan.data_file_count, 1);
}

#[test]
fn raw_cdc_epoch_writer_includes_all_dataset_sources_when_unscoped() {
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
        "store-999",
        "tx-2",
        "0/16B6D00",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-999", "999", "2026-01-01")),
        )],
    );

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[first, second])
        .expect("raw CDC writer plan");

    assert_eq!(plan.transaction_count, 2);
    assert_eq!(plan.change_count, 2);
    assert_eq!(plan.row_intents.len(), 2);
    assert_eq!(
        plan.epoch_metadata
            .source_rows
            .iter()
            .map(|source| source.source_id.as_str())
            .collect::<Vec<_>>(),
        vec!["store-001", "store-999"]
    );
}
