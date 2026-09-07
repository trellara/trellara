use super::*;

#[tokio::test]
async fn lake_writer_plan_command_reports_configured_source_gap() {
    let config_file = lake_writer_config_file("gap");
    let envelope_file = lake_writer_envelope_file("gap");
    fs::write(
        &config_file,
        STRICT_YAML.replace(
            "    - schema: public\n      name: sales",
            "    - schema: public\n      name: sales\n      verify:\n        primary_key: id",
        ),
    )
    .expect("write config");

    let envelope = lake_writer_envelope(
        "unexpected-source",
        "tx-lake-writer-gap",
        "unexpected-source:0/16B6C50:tx-lake-writer-gap:1",
        false,
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
        &[],
    )
    .expect("deterministic epoch id");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::WriterPlan(LakeWriterPlanArgs {
                config: config_file.clone(),
                files: vec![envelope_file.clone()],
                epoch_id: DETERMINISTIC_EPOCH_ID.to_string(),
                source_bucket_count: 1,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("writer plan");

    assert!(output.contains(&format!("dataset: retail-sales epoch: {expected_epoch_id}")));
    assert!(output.contains("state=open policy=wait_all_required"));
    assert!(output.contains("sources=0/1 missing=1 quarantined=0"));
    assert!(output.contains("verification=unknown"));
    assert!(output.contains("transactions=0 changes=0"));
    assert!(output.contains("data_files=0"));
    assert!(output.contains("row_intents: count=0"));
    assert!(output.contains("source_rows: count=1"));
    assert!(output.contains("source_row source=local-source state=lagging"));
    assert!(output.contains("start_lsn= end_lsn= transactions=0 changes=0 checksum=0"));
    assert!(output.contains("lag_reason=required source has not reached epoch boundary"));
    assert!(!output.contains("source_row source=unexpected-source"));

    fs::remove_file(config_file).expect("remove config");
    fs::remove_file(envelope_file).expect("remove envelope");
}

#[test]
fn lake_writer_plan_text_lists_quarantine_rows() {
    let writer_config = trellara_lake::LakeRawCdcWriterConfig::new("retail-sales", "epoch-1", 1)
        .with_required_sources(["local-source"])
        .with_straggler_policy(trellara_lake::LakeStragglerPolicy::QuarantineOnGap);
    let plan_config =
        trellara_lake::LakePlanConfig::new(vec![trellara_lake::LakeTableConfig::new(
            "public", "sales", "id",
        )]);
    let plan = trellara_lake::plan_raw_cdc_epoch_writes(&writer_config, &plan_config, &[])
        .expect("raw CDC writer plan");

    let output = render_lake_writer_plan_summary(&plan, QuickstartOutputFormat::Text)
        .expect("writer plan text");

    assert!(output.contains("state=quarantined policy=quarantine_on_gap"));
    assert!(output.contains("sources=0/1 missing=0 quarantined=1"));
    assert!(output.contains("quarantine_rows=1"));
    assert!(output.contains("quarantine_rows: count=1"));
    assert!(output.contains("quarantine_row source=local-source"));
    assert!(output.contains("commit_lsn=<none>"));
    assert!(output.contains("reason=source_gap_quarantined"));
    assert!(output.contains("required source missing under quarantine-on-gap policy"));
    assert!(output.contains("recovery_command=trellara lake fanin verify"));
}
