use super::*;

mod package;

#[tokio::test]
async fn pilot_evidence_command_renders_executive_brief() {
    let root = std::env::temp_dir().join(format!(
        "trellara-pilot-evidence-command-{}",
        std::process::id()
    ));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create pilot evidence temp dir");
    fs::write(&config_path, local_stream_yaml()).expect("write pilot evidence config");

    let output = execute(Cli {
        command: Command::PilotEvidence(PilotEvidenceArgs {
            config: config_path,
            format: PilotGuideOutputFormat::Text,
        }),
    })
    .await
    .expect("pilot evidence output");

    assert!(output.contains("Trellara Executive Evidence Brief"));
    assert!(output.contains("North Star"));
    assert!(output.contains("transaction-boundary proof"));
    assert!(output.contains("brokerless local stream"));
    assert!(output.contains("quarantine reason"));

    fs::remove_dir_all(root).expect("remove pilot evidence temp dir");
}
