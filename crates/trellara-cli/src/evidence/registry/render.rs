use std::fmt::Write as _;

use crate::{EvidenceRegistrySummary, PilotGuideOutputFormat, Result};

pub(crate) fn render_evidence_registry_summary(
    summary: &EvidenceRegistrySummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_evidence_registry_text(summary)),
    }
}

pub(crate) fn render_evidence_registry_text(summary: &EvidenceRegistrySummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara evidence registry").expect("write string");
    writeln!(&mut output, "package: {}", summary.package).expect("write string");
    writeln!(&mut output, "manifest: {}", summary.manifest).expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "verified: {}", summary.verified).expect("write string");
    writeln!(&mut output, "artifacts: {}", summary.artifact_count).expect("write string");
    writeln!(
        &mut output,
        "verified_artifacts: {}",
        summary.verified_artifact_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "missing_artifacts: {}",
        summary.missing_artifact_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "digest_mismatches: {}",
        summary.digest_mismatch_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "package_manifest_sha256: {}",
        summary.package_manifest_sha256
    )
    .expect("write string");
    writeln!(
        &mut output,
        "review_surfaces: {} verified, {} required, {} missing_required",
        summary.review_surface_count,
        summary.required_review_surface_count,
        summary.missing_required_review_surface_count
    )
    .expect("write string");
    if let Some(report) = &summary.correctness_report {
        writeln!(
            &mut output,
            "correctness_report: {} present={} sha256={}",
            report.path,
            report.present,
            report.sha256.as_deref().unwrap_or("missing")
        )
        .expect("write string");
    }

    output.push_str("\nreview_surfaces:\n");
    for surface in &summary.review_surfaces {
        writeln!(
            &mut output,
            "- {}: {} ({})",
            surface.code, surface.artifact, surface.evidence
        )
        .expect("write string");
    }

    if !summary.issues.is_empty() {
        output.push_str("\nissues:\n");
        for issue in &summary.issues {
            writeln!(&mut output, "- {issue}").expect("write string");
        }
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
