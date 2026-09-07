use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    pilot_evidence_template_artifacts, render_pilot_evidence_collect_script,
    render_pilot_evidence_template_readme, shell_quote, CliError, PilotEvidenceTemplateFile,
    PilotEvidenceTemplateSummary, PilotGuideOutputFormat, PilotScorecardSummary, Result,
    TrellaraConfig,
};

pub(crate) fn write_pilot_evidence_template(
    config: &TrellaraConfig,
    config_path: &Path,
    output: &Path,
) -> Result<PilotEvidenceTemplateSummary> {
    fs::create_dir_all(output).map_err(|source| CliError::WriteOutput {
        path: output.display().to_string(),
        source,
    })?;

    let scorecard = PilotScorecardSummary::from_config(config, config_path);
    let artifacts = pilot_evidence_template_artifacts(&scorecard, output);

    let readme_path = output.join("README.md");
    let script_path = output.join("collect.sh");
    fs::write(
        &readme_path,
        render_pilot_evidence_template_readme(config, config_path, output, &artifacts),
    )
    .map_err(|source| CliError::WriteOutput {
        path: readme_path.display().to_string(),
        source,
    })?;
    fs::write(
        &script_path,
        render_pilot_evidence_collect_script(config, config_path, output, &artifacts),
    )
    .map_err(|source| CliError::WriteOutput {
        path: script_path.display().to_string(),
        source,
    })?;

    let files = vec![
        PilotEvidenceTemplateFile {
            path: readme_path.display().to_string(),
            purpose: "operator handoff for collecting live scorecard evidence".to_string(),
        },
        PilotEvidenceTemplateFile {
            path: script_path.display().to_string(),
            purpose: "reviewable shell script that writes gate evidence artifacts".to_string(),
        },
    ];

    Ok(PilotEvidenceTemplateSummary {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        config: config_path.display().to_string(),
        output: output.display().to_string(),
        artifact_count: artifacts.len(),
        files,
        artifacts,
        next_commands: vec![
            format!("sh {}", shell_quote(&script_path.display().to_string())),
            format!(
                "trellara pilot evidence-check --config {} --evidence-dir {} --format text",
                config_path.display(),
                output.display()
            ),
        ],
    })
}

pub(crate) fn render_pilot_evidence_template_summary(
    summary: &PilotEvidenceTemplateSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_pilot_evidence_template_text(summary)),
    }
}

pub(crate) fn render_pilot_evidence_template_text(
    summary: &PilotEvidenceTemplateSummary,
) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara pilot evidence template").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "output: {}", summary.output).expect("write string");
    writeln!(&mut output, "artifacts: {}", summary.artifact_count).expect("write string");

    output.push_str("\nfiles:\n");
    for file in &summary.files {
        writeln!(&mut output, "- {}: {}", file.path, file.purpose).expect("write string");
    }

    output.push_str("\nlive_artifacts:\n");
    for artifact in &summary.artifacts {
        writeln!(
            &mut output,
            "- {} -> {}",
            artifact.gate_code, artifact.artifact
        )
        .expect("write string");
        writeln!(&mut output, "  command: {}", artifact.proof_command).expect("write string");
        writeln!(
            &mut output,
            "  success_markers: {}",
            artifact.success_markers.join(", ")
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  requirements: {}",
            artifact.collection_requirements.join("; ")
        )
        .expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
