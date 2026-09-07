use super::*;

mod fixtures;
use fixtures::*;

#[tokio::test]
async fn lake_fanin_run_command_renders_bounded_dry_run() {
    let fixture = LakeFaninRunFixture::new(
        "trellara-lake-fanin-run",
        "local-source",
        "tx-lake-fanin-run",
    );
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
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Run(LakeFaninRunArgs {
                    config: fixture.config_file.clone(),
                    files: vec![fixture.envelope_file.clone(), fixture.envelope_file.clone()],
                    epoch_id: DETERMINISTIC_EPOCH_ID.to_string(),
                    source_bucket_count: 1,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("fan-in run");

    assert!(output.contains("Trellara lake fan-in run"));
    assert!(output.contains(&format!("dataset: retail-sales epoch: {expected_epoch_id}")));
    assert!(output.contains("status=planned_with_duplicate_replays"));
    assert!(output.contains("transactions=1 changes=1 duplicate_replays=1"));
    assert!(output.contains("data_files=1 source_buckets=1 replay_safe=true"));
    assert!(output.contains("trellara lake fanin verify reports match"));
    assert!(
        output.contains("source acknowledgement advances after durable Trellara stream publish")
    );
    assert!(output.contains("slow Iceberg catalog commits backpressure lake fan-in consumers"));
    assert!(output.contains("dry-run local trial: no Iceberg catalog writes are performed"));
    assert!(
        output.contains("writer_plan: committers=1 data_files=1 state=complete verification=match")
    );

    fixture.remove();
}

#[tokio::test]
async fn lake_fanin_run_command_blocks_non_consumable_writer_epoch() {
    let fixture = LakeFaninRunFixture::new(
        "trellara-lake-fanin-run-gap",
        "unexpected-source",
        "tx-lake-fanin-run-gap",
    );
    let expected_epoch_id = trellara_lake::deterministic_epoch_id(
        "retail-sales",
        ["local-source"],
        &trellara_lake::LakeStragglerPolicy::WaitAllRequired,
        &[],
    )
    .expect("deterministic epoch id");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Run(LakeFaninRunArgs {
                    config: fixture.config_file.clone(),
                    files: vec![fixture.envelope_file.clone()],
                    epoch_id: DETERMINISTIC_EPOCH_ID.to_string(),
                    source_bucket_count: 1,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("fan-in run");

    assert!(output.contains(&format!("dataset: retail-sales epoch: {expected_epoch_id}")));
    assert!(output.contains("status=blocked_no_data_files"));
    assert!(output.contains("transactions=0 changes=0 duplicate_replays=0"));
    assert!(output.contains("data_files=0"));
    assert!(output.contains(
        "spark_release_gate: blocked: writer plan epoch state open with verification unknown is not consumable"
    ));
    assert!(
        output.contains("writer_plan: committers=0 data_files=0 state=open verification=unknown")
    );

    fixture.remove();
}
