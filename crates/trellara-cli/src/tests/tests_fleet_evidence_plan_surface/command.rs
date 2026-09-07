use super::*;

#[tokio::test]
async fn fleet_evidence_plan_command_renders_text() {
    let fixture = FleetEvidenceFixture::new("trellara-fleet-evidence-plan-command");
    let local_path = fixture.write_config("local.yml", local_stream_yaml());

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::EvidencePlan(FleetEvidencePlanArgs {
                config: vec![local_path],
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("fleet evidence plan output");

    assert!(output.contains("Trellara fleet evidence plan"));
    assert!(output.contains("ready_to_collect_live_evidence"));
    assert!(output.contains("required_live_artifacts:"));
    assert!(output.contains("trellara check --config"));
    assert!(output.contains("source-safety.txt"));
    assert!(output.contains("lake-writer-plan.json"));
    assert!(output.contains("lake-completeness.json"));
    assert!(output.contains("trellara evidence-registry --package"));
}

#[tokio::test]
async fn fleet_evidence_plan_command_renders_partition_rebalance_evidence() {
    let fixture = FleetEvidenceFixture::new("trellara-fleet-evidence-plan-partitioned-command");
    let partitioned_path = fixture.write_config("partitioned.yml", partitioned_yaml());

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::EvidencePlan(FleetEvidencePlanArgs {
                config: vec![partitioned_path],
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("partitioned fleet evidence plan output");

    assert!(output.contains("partition_rebalance_plan"));
    assert!(output.contains("partition-rebalance-plan.json"));
    assert!(output.contains("runtime movement disabled"));
    assert!(output.contains("trellara partition-rebalance-plan --config"));
}
