#[test]
fn spark_current_state_template_is_epoch_guarded_and_parameterized() {
    let template = include_str!("../../../../../examples/spark/current_state.sql");

    for placeholder in [
        "${catalog}",
        "${namespace}",
        "${raw_cdc_table}",
        "${epochs_table}",
        "${verification_table}",
        "${target_table}",
        "${epoch_id}",
        "${primary_key_column}",
        "${accept_complete_with_gaps}",
    ] {
        assert!(
            template.contains(placeholder),
            "missing placeholder {placeholder}"
        );
    }
    assert!(template.contains("e.state = 'complete'"));
    assert!(template.contains("e.state = 'complete_with_gaps'"));
    assert!(template.contains("e.policy = 'publish_with_gaps'"));
    assert!(template.contains("${accept_complete_with_gaps} = true"));
    assert!(template.contains("v.checksum_status = 'match'"));
    assert!(template.contains("raise_error('Trellara epoch is not consumable"));
    assert!(template.contains("trellara_epoch_consumable"));
    assert!(template.contains("__trellara_epoch_id"));
    assert!(template.contains("__trellara_idempotency_key"));
    assert!(template.contains("__trellara_commit_lsn"));
    assert!(template.contains("c.commit_lsn"));
    assert!(template.contains("c.record_key"));
    assert!(template.contains("c.payload_after_json"));
    assert!(template.contains("__trellara_commit_lsn_numeric"));
    assert!(template.contains("MERGE INTO"));
    assert!(template.contains("source.__trellara_operation = 'delete'"));
    assert!(template.contains("source.__trellara_commit_lsn_numeric >"));
    assert!(template.contains(") THEN DELETE"));
}

#[test]
fn spark_maintenance_template_is_verification_guarded_and_parameterized() {
    let template = include_str!("../../../../../examples/spark/maintenance.sql");

    for placeholder in [
        "${catalog}",
        "${namespace}",
        "${raw_cdc_table}",
        "${epochs_table}",
        "${verification_table}",
        "${target_table}",
        "${epoch_id}",
        "${primary_key_column}",
        "${accept_complete_with_gaps}",
    ] {
        assert!(
            template.contains(placeholder),
            "missing placeholder {placeholder}"
        );
    }
    assert!(template.contains("v.checksum_status = 'match'"));
    assert!(template.contains("e.state = 'complete'"));
    assert!(template.contains("e.state = 'complete_with_gaps'"));
    assert!(template.contains("e.policy = 'publish_with_gaps'"));
    assert!(template.contains("raise_error('Trellara epoch is not consumable"));
    assert!(template.contains("rewrite_data_files"));
    assert!(template.contains("expire_snapshots"));
    assert!(template.contains("remove_orphan_files"));
    assert!(template.contains("target-file-size-bytes"));
    assert!(template.contains("retain_last => 10"));
    assert!(template.contains("_trellara_epoch_sources"));
    assert!(template.contains("_trellara_epoch_tables"));
    assert!(template.contains("_trellara_quarantine"));
    assert!(template.contains("${epoch_partitions_table}"));
}

#[test]
fn spark_scd2_template_is_epoch_guarded_and_idempotency_ready() {
    let template = include_str!("../../../../../examples/spark/scd2.sql");

    for placeholder in [
        "${catalog}",
        "${namespace}",
        "${raw_cdc_table}",
        "${epochs_table}",
        "${verification_table}",
        "${target_table}",
        "${epoch_id}",
        "${primary_key_column}",
        "${accept_complete_with_gaps}",
    ] {
        assert!(
            template.contains(placeholder),
            "missing placeholder {placeholder}"
        );
    }
    assert!(template.contains("e.state = 'complete'"));
    assert!(template.contains("e.state = 'complete_with_gaps'"));
    assert!(template.contains("e.policy = 'publish_with_gaps'"));
    assert!(template.contains("${accept_complete_with_gaps} = true"));
    assert!(template.contains("v.checksum_status = 'match'"));
    assert!(template.contains("raise_error('Trellara epoch is not consumable"));
    assert!(template.contains("trellara_epoch_consumable"));
    assert!(template.contains("__trellara_valid_from"));
    assert!(template.contains("__trellara_valid_to"));
    assert!(template.contains("__trellara_is_current"));
    assert!(template.contains("__trellara_idempotency_key"));
    assert!(template.contains("c.commit_lsn"));
    assert!(template.contains("c.record_key"));
    assert!(template.contains("c.payload_after_json"));
    assert!(template.contains("LEAD(__trellara_commit_timestamp)"));
    assert!(template.contains("MERGE INTO"));
    assert!(template.contains("WHEN NOT MATCHED THEN INSERT"));
}

#[test]
fn spark_dashboard_template_surfaces_partition_evidence() {
    let template = include_str!("../../../../../examples/spark/completeness_dashboard.sql");

    for placeholder in [
        "${catalog}",
        "${namespace}",
        "${epochs_table}",
        "${epoch_partitions_table}",
        "${verification_table}",
        "${quarantine_table}",
        "${target_table}",
        "${epoch_id}",
        "${accept_complete_with_gaps}",
    ] {
        assert!(
            template.contains(placeholder),
            "missing placeholder {placeholder}"
        );
    }
    assert!(template.contains("partition_evidence"));
    assert!(template.contains("e.policy = 'publish_with_gaps'"));
    assert!(template.contains("first_commit_lsn"));
    assert!(template.contains("last_commit_lsn"));
    assert!(template.contains("partition_event_count"));
}
