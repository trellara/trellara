use super::*;

#[tokio::test]
async fn fleet_init_command_writes_summary_and_files() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-init-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::Init(fleet_init_args(root.clone())),
        },
    })
    .await
    .expect("fleet init output");

    assert!(output.contains("\"fleet_id\": \"design-partner-fleet\""));
    assert!(output.contains("\"flow_count\": 2"));
    assert!(output.contains("fleet-manifest.json"));
    assert!(output.contains("README.md"));
    assert!(output.contains("customer-east.yml"));
    assert!(output.contains("customer-west.yml"));
    assert!(output.contains("trellara fleet report --config"));
    assert!(output.contains("trellara quickstart --config"));
    assert!(output.contains("trellara pilot-package --config"));
    assert!(root.join("configs/customer-east.yml").exists());
    assert!(root.join("configs/customer-west.yml").exists());
    assert!(root.join("fleet-manifest.json").exists());
    assert!(root.join("README.md").exists());

    fs::remove_dir_all(root).expect("remove fleet init command temp dir");
}

#[tokio::test]
async fn fleet_scorecard_command_renders_text() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-scorecard-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet scorecard command temp dir");
    let local_path = root.join("local.yml");
    fs::write(&local_path, local_stream_yaml()).expect("write local config");

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::Scorecard(FleetScorecardArgs {
                config: vec![local_path],
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("fleet scorecard output");

    assert!(output.contains("Trellara fleet scorecard"));
    assert!(output.contains("verdict: ready_for_design_partner_fleet_review"));
    assert!(output.contains("flow_status: 1 ready, 0 review_required, 0 blocked"));
    assert!(output.contains("local-source:retail-sales"));
    assert!(output.contains("review_sequence:"));
    assert!(output.contains("trellara fleet report --config"));

    fs::remove_dir_all(root).expect("remove fleet scorecard command temp dir");
}

#[tokio::test]
async fn fleet_report_command_renders_text() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-report-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet report command temp dir");
    let local_path = root.join("local.yml");
    fs::write(&local_path, local_stream_yaml()).expect("write local config");

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::Report(FleetReportArgs {
                config: vec![local_path.clone()],
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("fleet report output");

    assert!(output.contains("Trellara fleet report"));
    assert!(output.contains("verdict: ready_for_design_partner_review"));
    assert!(output.contains("local-source:retail-sales"));
    assert!(output.contains("trellara status --config"));
    assert!(output.contains("stream inspect-local"));
    assert!(output.contains("recovery_drills: 7"));
    assert!(output.contains("quarantine replay-ready"));
    assert!(output.contains("reseed"));
    assert!(output.contains("lake_offline_source_gap_acceptance"));
    assert!(output.contains("lake_conflicting_duplicate_quarantine"));

    fs::remove_dir_all(root).expect("remove fleet report command temp dir");
}
