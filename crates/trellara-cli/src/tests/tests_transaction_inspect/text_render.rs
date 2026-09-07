use super::*;

#[test]
fn transaction_inspector_text_renders_boundary_evidence() {
    let summary = TransactionInspectSummary::from_envelope(&inspect_envelope());
    let output = render_transaction_inspect_summary(&summary, TransactionInspectOutputFormat::Text)
        .expect("transaction inspect text");

    assert!(output.contains("Trellara transaction inspection"));
    assert!(output.contains("transaction: tx-inspect status=verified mode=partitioned_scale_mode"));
    assert!(output.contains("lsn_boundary: begin=0/16B6B00 commit=0/16B6C50"));
    assert!(output.contains("events: dml=3 ddl=0 source_total=3"));
    assert!(output.contains("checksum_status=match"));
    assert!(output.contains("visibility_contract: barrier-aware consumers wait for the manifest"));
    assert!(output.contains("partition-local consumers may read a lane earlier"));
    assert!(output.contains("ddl_events:\n- none"));
    assert!(output.contains("manifest_barrier_required=true"));
    assert!(output.contains("manifest_valid=true"));
    assert!(output.contains("source_boundary_kind=dml_only"));
    assert!(output.contains("partitioned_scale_decision=partition_parallel_dml"));
    assert!(output.contains("partition_parallel_safe=true"));
    assert!(output.contains("requires_ddl_barrier=false"));
    assert!(output.contains("dml_replay_after_ddl_barrier_required=false"));
    assert!(output.contains(
        "partitioned_scale_reason=DML-only transaction can be partitioned without a DDL barrier"
    ));
    assert!(output.contains("global_event_count_matches=true"));
    assert!(output.contains("partition_event_count_matches=true"));
    assert!(output.contains("partitioned_scale_manifest_checksum="));
    assert!(output.contains("partitioned_scale_manifest_event_count=3"));
    assert!(output.contains("partitioned_scale_envelope_event_count=3"));
    assert!(output.contains("partitioned_scale_event_count_coverage=true"));
    assert!(output.contains("partitioned_scale_participating_partition_ids=0,2"));
    assert!(output.contains(
        "partitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition"
    ));
    assert!(output.contains("participating_partition_count=2"));
    assert!(output.contains("public.orders events=2 inserts=1 updates=0 deletes=1"));
    assert!(output.contains("partition_manifest:"));
    assert!(output.contains(
        "boundary_mode=partitioned_scale_mode global_event_count=3 participating_partition_count=2"
    ));
    assert!(output.contains("partition=0 events=1 total_order=1..1"));
    assert!(output.contains("partition=2 events=2 total_order=2..3"));
}

#[test]
fn transaction_inspector_text_renders_ddl_replay_boundary() {
    let mut envelope = inspect_envelope();
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::manual_review(
        "tx-inspect",
        4,
        trellara_protocol::DdlOperation::RenameColumn,
        trellara_protocol::RelationId::new(1, "public", "orders"),
        "ALTER TABLE \"public\".\"orders\" RENAME COLUMN \"coupon\" TO \"discount_code\";",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let output = render_transaction_inspect_summary(
        &TransactionInspectSummary::from_envelope(&envelope),
        TransactionInspectOutputFormat::Text,
    )
    .expect("transaction inspect text");

    assert!(output.contains("events: dml=3 ddl=1 source_total=4"));
    assert!(output.contains("source_boundary_kind=mixed_ddl_and_dml"));
    assert!(output.contains("partitioned_scale_decision=ddl_barrier_required"));
    assert!(output.contains("partition_parallel_safe=false"));
    assert!(output.contains("requires_ddl_barrier=true"));
    assert!(output.contains("dml_replay_after_ddl_barrier_required=true"));
    assert!(output.contains(
        "partitioned_scale_reason=mixed DDL and DML transaction must apply DDL barrier before partitioned DML replay"
    ));
    assert!(output.contains(
        "order=4 operation=rename_column relation=public.orders auto_apply=false release_gate=post_ddl_dml_release"
    ));
    assert!(output.contains(
        "dml_replay_after_ddl_barrier: barrier_id=source-a:retail:sales:tx-inspect:0/16B6C50:ddl changes=3"
    ));
    assert!(output.contains("ddl_events_stripped=true"));
    assert!(output.contains("use only after the DDL barrier releases post-DDL DML"));
}
