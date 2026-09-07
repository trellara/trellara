use super::*;

#[test]
fn cli_parses_evaluate_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "evaluate",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Evaluate(EvaluateArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}

#[test]
fn cli_parses_consistency_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "consistency",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Consistency(ConsistencyContractArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}

#[test]
fn cli_parses_performance_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "performance",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Performance(PerformanceEnvelopeArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}

#[test]
fn cli_parses_identity_audit_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "identity-audit",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::IdentityAudit(IdentityAuditArgs {
            format: PilotGuideOutputFormat::Text,
            ..
        })
    ));
}

#[test]
fn cli_parses_evidence_registry_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "evidence-registry",
        "--package",
        "target/pilot-package",
        "--correctness-report",
        "docs/correctness-report.html",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::EvidenceRegistry(EvidenceRegistryArgs {
            package,
            format: PilotGuideOutputFormat::Text,
            ..
        }) if package.as_path() == Path::new("target/pilot-package")
    ));
}
