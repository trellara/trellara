use std::fmt::Write as _;
use std::path::Path;

use crate::{shell_quote, PilotEvidenceTemplateArtifact, StreamConfig, TrellaraConfig};

pub(crate) fn render_pilot_evidence_template_readme(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    artifacts: &[PilotEvidenceTemplateArtifact],
) -> String {
    let mut readme = String::new();
    writeln!(&mut readme, "# Trellara Live Evidence Collection").expect("write string");
    writeln!(&mut readme).expect("write string");
    writeln!(&mut readme, "config: {}", config_path.display()).expect("write string");
    writeln!(&mut readme, "source: {}", config.source.id).expect("write string");
    writeln!(&mut readme, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut readme).expect("write string");
    readme.push_str("Run `sh collect.sh` only after reviewing the commands. Each command writes one artifact that `trellara pilot evidence-check` validates with gate-specific success markers.\n\n");
    readme.push_str("## Identity Binding\n\n");
    readme.push_str("Every artifact that lists `source dataset identity` must include meaningful source and dataset identifiers for this flow. Placeholder values such as `missing`, `none`, `null`, or `unknown` are rejected.\n\n");
    readme.push_str("## Artifacts\n\n");
    for artifact in artifacts {
        writeln!(
            &mut readme,
            "- `{}`: `{}`",
            artifact.artifact, artifact.proof_command
        )
        .expect("write string");
        writeln!(
            &mut readme,
            "  success markers: {}",
            artifact.success_markers.join(", ")
        )
        .expect("write string");
        readme.push_str("  requirements:\n");
        for requirement in &artifact.collection_requirements {
            writeln!(&mut readme, "  - {requirement}").expect("write string");
        }
    }
    readme.push_str("\n## Verify\n\n");
    writeln!(
        &mut readme,
        "`trellara pilot evidence-check --config {} --evidence-dir {} --format text`",
        config_path.display(),
        output.display()
    )
    .expect("write string");
    readme
}

pub(crate) fn render_pilot_evidence_collect_script(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    artifacts: &[PilotEvidenceTemplateArtifact],
) -> String {
    let mut script = String::new();
    script.push_str("#!/usr/bin/env sh\n");
    script.push_str("set -eu\n\n");
    writeln!(
        &mut script,
        "mkdir -p {}",
        shell_quote(&output.display().to_string())
    )
    .expect("write string");
    for artifact in artifacts {
        writeln!(
            &mut script,
            "\n# {}\n{} > {}",
            artifact.gate_code,
            collect_artifact_command(config, config_path, output, artifact),
            shell_quote(&artifact.artifact)
        )
        .expect("write string");
    }
    writeln!(
        &mut script,
        "\ntrellara pilot evidence-check --config {} --evidence-dir {} --format text",
        shell_quote(&config_path.display().to_string()),
        shell_quote(&output.display().to_string())
    )
    .expect("write string");
    script
}

fn collect_artifact_command(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
    artifact: &PilotEvidenceTemplateArtifact,
) -> String {
    if artifact.gate_code == "transaction_boundary"
        && matches!(config.stream, StreamConfig::Local { .. })
    {
        return format!(
            "trellara run --local --verify --format text --config {} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100",
            shell_quote(&config_path.display().to_string())
        );
    }
    if artifact.gate_code != "ddl_release_proof" {
        return artifact.proof_command.clone();
    }

    let package_dir = output.join("ddl-release-proof-package");
    let proof_path = package_dir.join("ddl-release-proof.json");
    format!(
        "trellara pilot-package --config {} --output {} >/dev/null\ncat {}",
        shell_quote(&config_path.display().to_string()),
        shell_quote(&package_dir.display().to_string()),
        shell_quote(&proof_path.display().to_string())
    )
}
