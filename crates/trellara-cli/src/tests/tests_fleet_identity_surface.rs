use super::*;

#[test]
fn fleet_identity_audit_marks_unique_flow_ids_ready() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-identity-ready-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet identity temp dir");
    let east_path = root.join("east.yml");
    let west_path = root.join("west.yml");
    fs::write(
        &east_path,
        local_stream_yaml()
            .replace("id: local-source", "id: east-source")
            .replace("id: retail-sales", "id: retail-east"),
    )
    .expect("write east config");
    fs::write(
        &west_path,
        local_stream_yaml()
            .replace("id: local-source", "id: west-source")
            .replace("id: retail-sales", "id: retail-west"),
    )
    .expect("write west config");

    let summary = FleetIdentityAuditSummary::from_args(&FleetIdentityAuditArgs {
        config: vec![east_path, west_path],
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet identity audit");

    assert_eq!(summary.verdict, "identity_ready_for_control_plane");
    assert_eq!(summary.flow_count, 2);
    assert_eq!(summary.unique_flow_count, 2);
    assert_eq!(summary.duplicate_flow_count, 0);
    assert!(summary.duplicate_groups.is_empty());
    assert!(summary
        .flows
        .iter()
        .all(|flow| flow.status == FleetIdentityStatus::Unique));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("evidence-registry")));

    fs::remove_dir_all(root).expect("remove fleet identity temp dir");
}

#[test]
fn fleet_identity_audit_blocks_duplicate_flow_ids() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-identity-collision-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet identity temp dir");
    let first_path = root.join("first.yml");
    let second_path = root.join("second.yml");
    fs::write(&first_path, local_stream_yaml()).expect("write first config");
    fs::write(&second_path, local_stream_yaml()).expect("write second config");

    let summary = FleetIdentityAuditSummary::from_args(&FleetIdentityAuditArgs {
        config: vec![first_path.clone(), second_path.clone()],
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet identity audit");

    assert_eq!(summary.verdict, "blocked_by_identity_collision");
    assert_eq!(summary.flow_count, 2);
    assert_eq!(summary.unique_flow_count, 0);
    assert_eq!(summary.duplicate_flow_count, 2);
    assert_eq!(summary.duplicate_groups.len(), 1);
    assert_eq!(
        summary.duplicate_groups[0].flow_id,
        "local-source:retail-sales"
    );
    assert!(summary.duplicate_groups[0]
        .configs
        .contains(&first_path.display().to_string()));
    assert!(summary.duplicate_groups[0]
        .remediation
        .contains("assign unique source.id and dataset.id values"));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("rename colliding source.id")));

    fs::remove_dir_all(root).expect("remove fleet identity temp dir");
}

#[test]
fn fleet_identity_audit_text_renders_remediation() {
    let summary = FleetIdentityAuditSummary {
        verdict: "blocked_by_identity_collision".to_string(),
        flow_count: 2,
        unique_flow_count: 0,
        duplicate_flow_count: 2,
        source_count: 1,
        dataset_count: 1,
        flows: vec![FleetIdentityFlow {
            flow_id: "source-a:sales".to_string(),
            config: "first.yml".to_string(),
            source_id: "source-a".to_string(),
            dataset_id: "sales".to_string(),
            mode: "strict_transaction_order".to_string(),
            stream_kind: "local".to_string(),
            status: FleetIdentityStatus::Duplicate,
        }],
        duplicate_groups: vec![FleetIdentityDuplicateGroup {
            flow_id: "source-a:sales".to_string(),
            configs: vec!["first.yml".to_string(), "second.yml".to_string()],
            remediation: "assign unique source.id and dataset.id values".to_string(),
        }],
        remediation: vec!["assign unique source.id and dataset.id values".to_string()],
        proof_commands: vec![
            "trellara fleet identity-audit --config first.yml --format text".to_string(),
        ],
        next_commands: vec!["rename colliding source.id or dataset.id values".to_string()],
    };

    let output = render_fleet_identity_audit_text(&summary);

    assert!(output.contains("Trellara fleet identity audit"));
    assert!(output.contains("verdict: blocked_by_identity_collision"));
    assert!(output.contains("[duplicate] source-a:sales"));
    assert!(output.contains("duplicate_groups:"));
    assert!(output.contains("assign unique source.id and dataset.id values"));
}

#[tokio::test]
async fn fleet_identity_audit_command_renders_text() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-identity-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet identity command temp dir");
    let first_path = root.join("first.yml");
    let second_path = root.join("second.yml");
    fs::write(&first_path, local_stream_yaml()).expect("write first config");
    fs::write(&second_path, local_stream_yaml()).expect("write second config");

    let output = execute(Cli {
        command: Command::Fleet {
            command: FleetCommand::IdentityAudit(FleetIdentityAuditArgs {
                config: vec![first_path, second_path],
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("fleet identity audit output");

    assert!(output.contains("Trellara fleet identity audit"));
    assert!(output.contains("verdict: blocked_by_identity_collision"));
    assert!(output.contains("duplicate_groups:"));
    assert!(output.contains("rerun trellara fleet identity-audit"));

    fs::remove_dir_all(root).expect("remove fleet identity command temp dir");
}
