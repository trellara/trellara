use super::*;

#[test]
fn transaction_inspector_surfaces_ddl_events_and_dml_replay_projection() {
    let mut envelope = inspect_envelope();
    envelope.transaction_id = "tx-ddl-inspect".to_string();
    for change in &mut envelope.changes {
        change.transaction_id = "tx-ddl-inspect".to_string();
        change.total_order += 1;
    }
    let manifest = envelope.manifest.as_mut().expect("manifest");
    manifest.transaction_id = "tx-ddl-inspect".to_string();
    manifest.global_event_count = 3;
    for partition in &mut manifest.partitions {
        partition.first_total_order += 1;
        partition.last_total_order += 1;
    }
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::additive_column(
        "tx-ddl-inspect",
        1,
        trellara_protocol::RelationId::new(1, "public", "orders"),
        "ALTER TABLE \"public\".\"orders\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let summary = TransactionInspectSummary::from_envelope(&envelope);

    assert_eq!(summary.event_count, 3);
    assert_eq!(summary.ddl_event_count, 1);
    assert_eq!(summary.source_event_count, 4);
    assert_eq!(
        summary.transaction_boundary.source_boundary_kind,
        "mixed_ddl_and_dml"
    );
    assert_eq!(
        summary.transaction_boundary.partitioned_scale_decision,
        "ddl_barrier_required"
    );
    assert_eq!(summary.ddl_events[0].operation, "add_column");
    assert_eq!(summary.ddl_events[0].relation, "public.orders");
    assert!(summary.ddl_events[0].target_auto_apply);
    let replay = summary
        .dml_replay_after_ddl_barrier
        .expect("DML replay projection");
    assert_eq!(
        replay.barrier_id,
        "source-a:retail:sales:tx-ddl-inspect:0/16B6C50:ddl"
    );
    assert_eq!(replay.change_count, 3);
    assert!(replay.ddl_events_stripped);
    assert_ne!(replay.checksum, envelope.checksum);
}
