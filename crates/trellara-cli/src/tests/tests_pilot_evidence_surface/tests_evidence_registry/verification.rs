use super::*;

#[test]
fn evidence_registry_verifies_pilot_package_manifest_and_review_surfaces() {
    let fixture = PilotEvidencePackageFixture::new("evidence-registry");

    let summary = EvidenceRegistrySummary::from_package(
        &fixture.package_path,
        Some(fixture.correctness_report_path.as_path()),
    )
    .expect("registry summary");

    assert!(summary.verified);
    assert_eq!(summary.artifact_count, 50);
    assert_eq!(summary.verified_artifact_count, 50);
    assert_eq!(summary.missing_artifact_count, 0);
    assert_eq!(summary.digest_mismatch_count, 0);
    assert_eq!(summary.package_manifest_sha256.len(), 64);
    assert!(summary.review_surface_count >= summary.required_review_surface_count);
    assert_eq!(summary.required_review_surface_count, 16);
    assert_eq!(summary.missing_required_review_surface_count, 0);
    assert!(summary
        .correctness_report
        .as_ref()
        .is_some_and(|report| report.present && report.sha256.is_some()));
    for code in expected_review_surfaces() {
        assert!(
            summary
                .review_surfaces
                .iter()
                .any(|surface| surface.code == *code),
            "missing review surface {code}"
        );
    }
    assert!(summary.artifacts.iter().any(|artifact| artifact
        .path
        .ends_with("correctness-report.html")
        && artifact.registry_status == EvidenceRegistryArtifactStatus::Verified));
    assert!(summary.next_commands.iter().any(|command| {
        command
            == &format!(
                "trellara evidence-registry --package {} --format text",
                fixture.package_path.display()
            )
    }));
}

#[test]
fn evidence_registry_indexes_partition_rebalance_surface_when_packaged() {
    let fixture = PilotEvidencePackageFixture::new_with_yaml(
        "evidence-registry-partition-rebalance",
        partitioned_yaml(),
    );

    let summary = EvidenceRegistrySummary::from_package(
        &fixture.package_path,
        Some(fixture.correctness_report_path.as_path()),
    )
    .expect("registry summary");

    assert!(summary.verified);
    assert_eq!(summary.required_review_surface_count, 16);
    assert_eq!(summary.missing_required_review_surface_count, 0);
    assert!(summary.artifacts.iter().any(|artifact| artifact
        .path
        .ends_with("partition-rebalance-plan.json")
        && artifact.registry_status == EvidenceRegistryArtifactStatus::Verified));
    assert!(summary.review_surfaces.iter().any(|surface| {
        surface.code == "partition_rebalance_plan"
            && surface
                .evidence
                .contains("runtime ownership movement disabled")
    }));
    assert!(summary.review_surfaces.iter().any(|surface| {
        surface.code == "support_diagnostics"
            && surface
                .evidence
                .contains("stream inspect-local recovery readiness fields")
    }));
}

#[test]
fn evidence_registry_describes_spark_runners_as_digest_bound_ddl_ack_evidence() {
    let fixture = PilotEvidencePackageFixture::new("evidence-registry-spark-digests");

    let summary = EvidenceRegistrySummary::from_package(
        &fixture.package_path,
        Some(fixture.correctness_report_path.as_path()),
    )
    .expect("registry summary");

    for code in [
        "spark_current_state_runner",
        "spark_scd2_runner",
        "spark_maintenance_runner",
        "spark_completeness_dashboard_runner",
    ] {
        let surface = summary
            .review_surfaces
            .iter()
            .find(|surface| surface.code == code)
            .unwrap_or_else(|| panic!("missing review surface {code}"));

        assert!(
            surface.evidence.contains("template_sha256"),
            "{code} missing template digest evidence wording"
        );
        assert!(
            surface.evidence.contains("DDL ACK evidence"),
            "{code} missing DDL ACK evidence wording"
        );
    }
}

fn expected_review_surfaces() -> &'static [&'static str] {
    &[
        "source_safety",
        "identity_audit",
        "schema_ddl_plan",
        "schema_ddl_apply_plan",
        "schema_ddl_envelope_plan",
        "ddl_barrier_status",
        "ddl_release_proof",
        "lake_writer_plan",
        "lake_fanin_run",
        "spark_current_state_template",
        "spark_current_state_runner",
        "spark_scd2_template",
        "spark_scd2_runner",
        "spark_maintenance_template",
        "spark_completeness_dashboard",
        "spark_maintenance_runner",
        "spark_golden_fixture",
        "fleet_evidence_plan",
        "live_evidence_template",
        "control_plane_pull",
        "support_diagnostics",
        "single_review_bundle",
    ]
}
