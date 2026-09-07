use super::*;

#[test]
fn evidence_registry_requires_recovery_review_surfaces() {
    let fixture = PilotEvidencePackageFixture::new("evidence-registry-recovery-surfaces");
    remove_manifest_artifacts(&fixture, &["diagnostics.json", "proof-bundle.md"]);

    let summary = EvidenceRegistrySummary::from_package(&fixture.package_path, None)
        .expect("registry summary");

    assert!(!summary.verified);
    assert!(summary.review_surfaces.len() >= 8);
    assert!(summary
        .issues
        .iter()
        .any(|issue| { issue.contains("required review surface support_diagnostics") }));
    assert!(summary
        .issues
        .iter()
        .any(|issue| { issue.contains("required review surface single_review_bundle") }));
}

#[test]
fn evidence_registry_flags_tampered_package_artifact() {
    let fixture = PilotEvidencePackageFixture::new("evidence-registry-tamper");
    fs::write(
        fixture.package_path.join("identity-audit.json"),
        "{\"tampered\":true}\n",
    )
    .expect("tamper artifact");

    let summary = EvidenceRegistrySummary::from_package(&fixture.package_path, None)
        .expect("registry summary");

    assert!(!summary.verified);
    assert_eq!(summary.digest_mismatch_count, 1);
    assert!(summary
        .issues
        .iter()
        .any(|issue| issue.contains("identity-audit.json") && issue.contains("digest mismatch")));
    assert!(summary
        .issues
        .iter()
        .any(|issue| issue.contains("required review surface identity_audit")));
    assert!(!summary
        .review_surfaces
        .iter()
        .any(|surface| surface.code == "identity_audit"));
}

#[test]
fn evidence_registry_requires_specific_enterprise_review_surfaces() {
    let fixture = PilotEvidencePackageFixture::new("evidence-registry-required-surfaces");
    remove_manifest_artifacts(&fixture, &["schema-ddl-plan.json"]);

    let summary = EvidenceRegistrySummary::from_package(&fixture.package_path, None)
        .expect("registry summary");

    assert!(!summary.verified);
    assert_eq!(summary.digest_mismatch_count, 0);
    assert!(summary.review_surfaces.len() >= 8);
    assert!(!summary
        .review_surfaces
        .iter()
        .any(|surface| surface.code == "schema_ddl_plan"));
    assert!(summary
        .issues
        .iter()
        .any(|issue| issue.contains("required review surface schema_ddl_plan")));
}

fn remove_manifest_artifacts(fixture: &PilotEvidencePackageFixture, suffixes: &[&str]) {
    let manifest_path = fixture.package_path.join("manifest.json");
    let manifest_contents = fs::read_to_string(&manifest_path).expect("read manifest");
    let mut manifest: PilotPackageManifest =
        serde_json::from_str(&manifest_contents).expect("parse manifest");
    manifest.artifacts.retain(|artifact| {
        !suffixes
            .iter()
            .any(|suffix| artifact.path.ends_with(suffix))
    });
    manifest.artifact_count = manifest.artifacts.len();
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}
