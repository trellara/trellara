use super::*;

#[tokio::test]
async fn evidence_registry_command_renders_verified_package_index() {
    let fixture = PilotEvidencePackageFixture::new("evidence-registry-command");
    let expected_registry_command = format!(
        "trellara evidence-registry --package {} --format text",
        fixture.package_path.display()
    );

    let output = execute(Cli {
        command: Command::EvidenceRegistry(EvidenceRegistryArgs {
            package: fixture.package_path.clone(),
            correctness_report: fixture.correctness_report_path.clone(),
            format: PilotGuideOutputFormat::Text,
        }),
    })
    .await
    .expect("evidence registry output");

    assert!(output.contains("Trellara evidence registry"));
    assert!(output.contains("verified: true"));
    assert!(output.contains("verified_artifacts: 50"));
    assert!(output.contains("review_surfaces:"));
    assert!(output.contains("16 required, 0 missing_required"));
    assert!(output.contains("correctness-report.html"));
    assert!(output.contains("source_safety"));
    assert!(output.contains("identity_audit"));
    assert!(output.contains("schema_ddl_plan"));
    assert!(output.contains("schema_ddl_apply_plan"));
    assert!(output.contains("schema_ddl_envelope_plan"));
    assert!(output.contains("safe SQL and barrier release sequence"));
    assert!(output.contains("runtime DDL envelope transaction barrier"));
    assert!(output.contains("propagation boundary"));
    assert!(output.contains("propagation decisions"));
    assert!(output.contains("policy digest"));
    assert!(output.contains("replay-envelope proof with ddl_events stripped"));
    assert!(output.contains("schema-barrier release gates"));
    assert!(output.contains("post-DDL DML visibility evidence"));
    assert!(output.contains("ddl_barrier_status"));
    assert!(output.contains("ddl_release_proof"));
    assert!(output.contains("cdc_transaction_boundary"));
    assert!(output.contains("required sink ACK evidence"));
    assert!(output.contains("lake_writer_plan"));
    assert!(output.contains("lake_fanin_run"));
    assert!(output.contains("lake_completeness"));
    assert!(output.contains("spark_current_state_template"));
    assert!(output.contains("spark_current_state_runner"));
    assert!(output.contains("template_sha256 for current-state DDL ACK evidence"));
    assert!(output.contains("spark_scd2_template"));
    assert!(output.contains("spark_scd2_runner"));
    assert!(output.contains("template_sha256 for SCD2 DDL ACK evidence"));
    assert!(output.contains("spark_maintenance_template"));
    assert!(output.contains("spark_maintenance_runner"));
    assert!(output.contains("template_sha256 for DDL ACK evidence"));
    assert!(output.contains("spark_completeness_dashboard"));
    assert!(output.contains("spark_completeness_dashboard_runner"));
    assert!(output.contains("template_sha256 for completeness dashboard DDL ACK evidence"));
    assert!(output.contains("spark_golden_fixture"));
    assert!(output.contains("fleet_evidence_plan"));
    assert!(output.contains("live_evidence_template"));
    assert!(output.contains("control_plane_pull"));
    assert!(output.contains("stream inspect-local recovery readiness fields"));
    assert!(output.contains("correctness_report:"));
    assert!(output.contains("package_manifest_sha256:"));
    assert!(output.contains(&expected_registry_command));
}
