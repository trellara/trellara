use super::*;

#[tokio::test]
async fn pilot_evidence_template_command_renders_text() {
    let root = temp_root("pilot-evidence-template-command");
    let output_path = root.join("live-evidence");
    fs::create_dir_all(&root).expect("create evidence template temp dir");
    let config_path = write_local_config(&root);

    let output = execute(Cli {
        command: Command::Pilot {
            command: PilotCommand::EvidenceTemplate(PilotEvidenceTemplateArgs {
                config: config_path,
                output: output_path.clone(),
                format: PilotGuideOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("evidence template output");

    assert!(output.contains("Trellara pilot evidence template"));
    assert!(output.contains("source_safety"));
    assert!(output.contains("source-safety.txt"));
    assert!(output_path.join("README.md").exists());
    assert!(output_path.join("collect.sh").exists());

    fs::remove_dir_all(root).expect("remove evidence template command temp dir");
}
