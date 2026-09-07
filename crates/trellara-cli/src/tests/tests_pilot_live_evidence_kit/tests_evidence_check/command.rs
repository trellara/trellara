use super::*;

#[tokio::test]
async fn pilot_evidence_check_command_renders_text() {
    let root = temp_root("pilot-evidence-check-command");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);

    let output = execute(Cli {
        command: Command::Pilot {
            command: PilotCommand::EvidenceCheck(PilotEvidenceCheckArgs {
                config: config_path,
                evidence_dir,
                format: PilotGuideOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("evidence check output");

    assert!(output.contains("Trellara pilot evidence check"));
    assert!(output.contains("verdict: missing_live_evidence"));
    assert!(output.contains("[missing] source_safety"));
    assert!(output.contains("next_commands:"));

    fs::remove_dir_all(root).expect("remove evidence check command temp dir");
}
