use super::*;

mod fixtures;
use fixtures::*;

#[test]
fn fleet_control_plane_report_defers_cloud_until_partner_evidence() {
    let fixture = two_flow_control_plane_fixture("trellara-fleet-control-plane");

    let summary = FleetControlPlaneSummary::from_args(&FleetControlPlaneArgs {
        config: fixture.configs(),
        format: QuickstartOutputFormat::Json,
    })
    .expect("control-plane report");

    assert_eq!(summary.verdict, "validate_with_design_partners");
    assert!(summary
        .build_recommendation
        .contains("gather missing fleet proof"));
    assert_eq!(summary.fleet_shape.flow_count, 2);
    assert_eq!(summary.fleet_shape.source_count, 2);
    assert_eq!(summary.fleet_shape.dataset_count, 2);
    assert!(summary
        .evidence_gaps
        .iter()
        .any(|gap| gap.contains("live source-safety")));
    assert!(summary.identity_collisions.is_empty());
    let topology = summary
        .capabilities_to_build
        .iter()
        .find(|capability| capability.code == "read_only_fleet_topology")
        .expect("topology capability");
    assert_eq!(
        topology.status,
        FleetControlPlaneCapabilityStatus::PulledNow
    );
    let orchestration = summary
        .capabilities_to_build
        .iter()
        .find(|capability| capability.code == "deployment_orchestration")
        .expect("deployment capability");
    assert_eq!(
        orchestration.status,
        FleetControlPlaneCapabilityStatus::Defer
    );
    assert!(summary
        .defer_until_pulled
        .contains(&"deployment_orchestration".to_string()));

    fixture.remove();
}

#[test]
fn fleet_control_plane_report_blocks_identity_collisions() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-control-plane-collision-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet control-plane temp dir");
    let first_path = root.join("first.yml");
    let second_path = root.join("second.yml");
    fs::write(&first_path, local_stream_yaml()).expect("write first config");
    fs::write(&second_path, local_stream_yaml()).expect("write second config");

    let summary = FleetControlPlaneSummary::from_args(&FleetControlPlaneArgs {
        config: vec![first_path, second_path],
        format: QuickstartOutputFormat::Json,
    })
    .expect("control-plane report");

    assert_eq!(summary.verdict, "blocked_by_identity_collision");
    assert!(summary
        .build_recommendation
        .contains("flow identities are stable"));
    assert!(summary
        .identity_collisions
        .iter()
        .any(|collision| collision.contains("control-plane identity would collide")));

    fs::remove_dir_all(root).expect("remove fleet control-plane temp dir");
}

#[test]
fn fleet_control_plane_text_renders_capability_pull() {
    let summary = FleetControlPlaneSummary {
        verdict: "build_minimal_control_plane".to_string(),
        build_recommendation: "build only read-only fleet topology and evidence packaging"
            .to_string(),
        fleet_shape: FleetControlPlaneShape {
            flow_count: 2,
            source_count: 2,
            dataset_count: 2,
            table_count: 6,
            local_stream_count: 2,
            kafka_stream_count: 0,
            partitioned_flow_count: 0,
            target_configured_count: 2,
            topology_verdict: "ready_for_design_partner_review".to_string(),
            fleet_scorecard_verdict: "ready_for_design_partner_fleet_review".to_string(),
        },
        capability_pull_count: 1,
        capabilities_to_build: vec![FleetControlPlaneCapability {
            code: "read_only_fleet_topology".to_string(),
            status: FleetControlPlaneCapabilityStatus::PulledNow,
            rationale: "many configs need one review surface".to_string(),
            evidence: vec!["flows=2".to_string()],
            build_when: "two or more partner-owned flows need one topology".to_string(),
        }],
        defer_until_pulled: vec!["deployment_orchestration".to_string()],
        evidence_gaps: Vec::new(),
        identity_collisions: Vec::new(),
        proof_commands: Vec::new(),
        next_commands: vec!["review feature-pull-list.md".to_string()],
    };

    let output = render_fleet_control_plane_text(&summary);

    assert!(output.contains("Trellara fleet control-plane pull report"));
    assert!(output.contains("verdict: build_minimal_control_plane"));
    assert!(output.contains("[pulled_now] read_only_fleet_topology"));
    assert!(output.contains("review feature-pull-list.md"));
}

#[tokio::test]
async fn fleet_control_plane_command_renders_text() {
    let fixture = two_flow_control_plane_fixture("trellara-fleet-control-plane-command");

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::ControlPlane(FleetControlPlaneArgs {
                config: fixture.configs(),
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("fleet control-plane output");

    assert!(output.contains("Trellara fleet control-plane pull report"));
    assert!(output.contains("verdict: validate_with_design_partners"));
    assert!(output.contains("[pulled_now] read_only_fleet_topology"));
    assert!(output.contains("[defer] deployment_orchestration"));

    fixture.remove();
}
