use super::*;

#[tokio::test]
async fn lake_writer_plan_command_renders_raw_cdc_epoch_intent() {
    let config_file = lake_writer_config_file("intent");
    let envelope_file = lake_writer_envelope_file("intent");
    fs::write(
        &config_file,
        STRICT_YAML
            .replace(
                "mode: strict_transaction_order",
                "mode: partitioned_scale_mode\n  partition:\n    partition_count: 8\n    key_column: store_id\n    null_key_policy: quarantine\n    key_change_policy: quarantine",
            )
            .replace(
                "    - schema: public\n      name: sales",
                "    - schema: public\n      name: sales\n      verify:\n        primary_key: id\n      contract:\n        source_schema_fingerprint: 42",
            ),
    )
    .expect("write config");

    let envelope = lake_writer_envelope(
        "local-source",
        "tx-lake-writer",
        "local-source:0/16B6C50:tx-lake-writer:1",
        true,
    );
    fs::write(
        &envelope_file,
        envelope.encode_checked().expect("encoded envelope"),
    )
    .expect("write envelope");
    let expected_epoch_id = trellara_lake::deterministic_epoch_id(
        "retail-sales",
        ["local-source"],
        &trellara_lake::LakeStragglerPolicy::WaitAllRequired,
        &[trellara_lake::LakeEpochSourceWindow::new(
            "local-source",
            Some("0/16B6C50"),
            Some("0/16B6C50"),
        )],
    )
    .expect("deterministic epoch id");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::WriterPlan(LakeWriterPlanArgs {
                config: config_file.clone(),
                files: vec![envelope_file.clone(), envelope_file.clone()],
                epoch_id: DETERMINISTIC_EPOCH_ID.to_string(),
                source_bucket_count: 1,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("writer plan");

    assert!(output.contains("Trellara raw CDC writer plan"));
    assert!(output.contains(&format!("dataset: retail-sales epoch: {expected_epoch_id}")));
    assert!(output.contains("transactions=1 changes=1 duplicate_replays=1"));
    assert!(output.contains("duplicate_replay_evidence: replay_safe=true duplicates=1"));
    assert!(output.contains("unique_transactions=1 row_intents=1 idempotency_keys=1"));
    assert!(output.contains("conflicting idempotency evidence fails closed"));
    assert!(output.contains("committer_topology:"));
    assert!(output.contains("single_table_committer_epoch_batched_append_only_raw_cdc"));
    assert!(
        output.contains("source acknowledgement advances after durable Trellara stream publish")
    );
    assert!(output.contains("catalog_backpressure_rule=slow Iceberg catalog commits"));
    assert!(output.contains("table_committer table=retail_sales__public__sales__raw_cdc"));
    assert!(output.contains("one committer owns this Iceberg table"));
    assert!(output.contains("commit_steps:"));
    assert!(output.contains("order=10 phase=raw_cdc_data"));
    assert!(output.contains("before source acknowledgement"));
    assert!(output.contains("replay this data-file intent idempotently"));
    assert!(output.contains("order=120 phase=epoch_partitions_metadata"));
    assert!(output.contains("order=130 phase=epoch_row_metadata"));
    assert!(output.contains("order=140 phase=verification_metadata"));
    assert!(output.contains("Iceberg checkpoint receipts"));
    assert!(output.contains(&format!("discover epoch {expected_epoch_id} metadata")));
    assert!(output.contains("verify checkpoint receipts"));
    assert!(output.contains("recovery_scenarios:"));
    assert!(output.contains("code=writer_crash_after_epoch_metadata"));
    assert!(output.contains("replay_policy=discover_existing_epoch_metadata_before_publish"));
    assert!(output.contains("publishing duplicate visibility"));
    assert!(output.contains("code=verification_mismatch_after_commit"));
    assert!(output.contains("hold Spark consumption"));
    assert!(output.contains("retail_sales__public__sales__raw_cdc"));
    assert!(output.contains(&format!("epoch_id={expected_epoch_id}/source_bucket=0000")));
    assert!(output.contains("row_intents: count=1"));
    assert!(output.contains("raw_row source=local-source relation=public.sales op=insert"));
    assert!(output.contains("tx=tx-lake-writer order=1 commit_lsn=0/16B6C50"));
    assert!(output.contains("record_key=sale-1"));
    assert!(output.contains("partition_key=store-001"));
    assert!(output.contains(
        "manifest_boundary_mode=<none> manifest_events=<none> manifest_partitions=<none>"
    ));
    assert!(output.contains("idempotency_key=local-source:0/16B6C50:tx-lake-writer:1"));
    assert!(output.contains("schema_fingerprint=42"));
    assert!(output.contains(&format!("epoch={expected_epoch_id}")));
    assert!(output.contains("before_cols=0 after_cols=3"));
    assert!(output.contains("checksum="));
    assert!(output.contains("retail_sales__trellara__fanin___trellara_epochs"));
    assert!(output.contains("retail_sales__trellara__fanin___trellara_epoch_partitions"));
    assert!(output.contains("retail_sales__trellara__fanin___trellara_quarantine"));
    assert!(output.contains("state=complete policy=wait_all_required"));
    assert!(output.contains("sources=1/1 missing=0 quarantined=0"));
    assert!(output.contains("manifest_digest="));
    assert!(output.contains("iceberg_snapshot_id=pending_catalog_commit"));
    assert!(output.contains("partition_rows=0"));
    assert!(output.contains("quarantine_rows=0"));
    assert!(output.contains("verification=match"));
    assert!(output.contains(&format!(
        "verification_id=verify:retail-sales:{expected_epoch_id}:"
    )));
    assert!(output.contains("completed_at=planned_after_raw_cdc_files_durable"));
    assert!(output.contains("source_rows: count=1"));
    assert!(output.contains(
        "source_row source=local-source state=complete start_lsn=0/16B6C50 end_lsn=0/16B6C50"
    ));
    assert!(output.contains("transactions=1 changes=1"));
    assert!(output.contains("lag_reason=<none>"));
    assert!(output.contains("partition_rows: count=0"));

    fs::remove_file(config_file).expect("remove config");
    fs::remove_file(envelope_file).expect("remove envelope");
}
