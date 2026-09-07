use super::*;

mod buyer_facing;
mod fleet;
mod lake;
mod manifest;
mod operational;
mod spark;

pub(super) struct PilotPackageContents {
    readme: String,
    readiness: String,
    plan: String,
    guide: String,
    guide_json: String,
    scorecard: String,
    scorecard_json: String,
    executive_evidence: String,
    enterprise_evaluation: String,
    enterprise_evaluation_json: String,
    schema_ddl_plan: String,
    schema_ddl_apply_plan: String,
    schema_ddl_envelope_plan: String,
    ddl_barrier_status: String,
    ddl_release_proof: String,
    local_run_proof: String,
    source_safety_checklist: String,
    proof_bundle: String,
    deployment_guide: String,
    operational_burden: String,
    feature_pull: String,
    fleet_report: String,
    fleet_scorecard: String,
    fleet_evidence_plan: String,
    consistency_contract: String,
    performance_envelope: String,
    identity_audit: String,
    consumer_semantics: String,
    lake_ddl: String,
    lake_epoch: String,
    lake_verify: String,
    lake_completeness: String,
    lake_writer_plan: String,
    lake_fanin_run: String,
    spark_current_state: String,
    spark_current_state_runner: String,
    spark_scd2: String,
    spark_scd2_runner: String,
    spark_maintenance: String,
    spark_maintenance_runner: String,
    spark_completeness_dashboard: String,
    spark_completeness_dashboard_runner: String,
    spark_golden_fixture: String,
    fleet_control_plane: String,
    diagnostics: String,
    diagnostics_json: String,
    live_evidence_readme: String,
    live_evidence_script: String,
    copied_correctness_report: String,
    manifest: String,
}

impl PilotPackageContents {
    pub(super) fn read_from(output_path: &std::path::Path) -> Self {
        Self {
            readme: read_package_file(output_path, "README.md"),
            readiness: read_package_file(output_path, "quickstart-readiness.txt"),
            plan: read_package_file(output_path, "quickstart-plan.txt"),
            guide: read_package_file(output_path, "pilot-guide.txt"),
            guide_json: read_package_file(output_path, "pilot-guide.json"),
            scorecard: read_package_file(output_path, "pilot-scorecard.txt"),
            scorecard_json: read_package_file(output_path, "pilot-scorecard.json"),
            executive_evidence: read_package_file(output_path, "executive-evidence.md"),
            enterprise_evaluation: read_package_file(output_path, "enterprise-evaluation.txt"),
            enterprise_evaluation_json: read_package_file(
                output_path,
                "enterprise-evaluation.json",
            ),
            schema_ddl_plan: read_package_file(output_path, "schema-ddl-plan.json"),
            schema_ddl_apply_plan: read_package_file(output_path, "schema-ddl-apply-plan.json"),
            schema_ddl_envelope_plan: read_package_file(
                output_path,
                "schema-ddl-envelope-plan.json",
            ),
            ddl_barrier_status: read_package_file(output_path, "ddl-barrier-status.json"),
            ddl_release_proof: read_package_file(output_path, "ddl-release-proof.json"),
            local_run_proof: read_package_file(output_path, "local-run-proof.md"),
            source_safety_checklist: read_package_file(output_path, "source-safety-checklist.md"),
            proof_bundle: read_package_file(output_path, "proof-bundle.md"),
            deployment_guide: read_package_file(output_path, "deployment-guide.md"),
            operational_burden: read_package_file(output_path, "operational-burden-notes.md"),
            feature_pull: read_package_file(output_path, "feature-pull-list.md"),
            fleet_report: read_package_file(output_path, "fleet-report.txt"),
            fleet_scorecard: read_package_file(output_path, "fleet-scorecard.txt"),
            fleet_evidence_plan: read_package_file(output_path, "fleet-evidence-plan.txt"),
            consistency_contract: read_package_file(output_path, "consistency-contract.json"),
            performance_envelope: read_package_file(output_path, "performance-envelope.json"),
            identity_audit: read_package_file(output_path, "identity-audit.json"),
            consumer_semantics: read_package_file(output_path, "consumer-semantics.json"),
            lake_ddl: read_package_file(output_path, "lake-ddl.json"),
            lake_epoch: read_package_file(output_path, "lake-epoch.json"),
            lake_verify: read_package_file(output_path, "lake-verify.json"),
            lake_completeness: read_package_file(output_path, "lake-completeness.json"),
            lake_writer_plan: read_package_file(output_path, "lake-writer-plan.json"),
            lake_fanin_run: read_package_file(output_path, "lake-fanin-run.json"),
            spark_current_state: read_package_file(output_path, "spark-current-state.sql"),
            spark_current_state_runner: read_package_file(output_path, "spark-current-state.py"),
            spark_scd2: read_package_file(output_path, "spark-scd2.sql"),
            spark_scd2_runner: read_package_file(output_path, "spark-scd2.py"),
            spark_maintenance: read_package_file(output_path, "spark-maintenance.sql"),
            spark_maintenance_runner: read_package_file(output_path, "spark-maintenance.py"),
            spark_completeness_dashboard: read_package_file(
                output_path,
                "spark-completeness-dashboard.sql",
            ),
            spark_completeness_dashboard_runner: read_package_file(
                output_path,
                "spark-completeness-dashboard.py",
            ),
            spark_golden_fixture: read_package_file(output_path, "spark-golden-fixture.json"),
            fleet_control_plane: read_package_file(output_path, "fleet-control-plane.json"),
            diagnostics: read_package_file(output_path, "diagnostics.txt"),
            diagnostics_json: read_package_file(output_path, "diagnostics.json"),
            live_evidence_readme: read_package_file(output_path, "live-evidence/README.md"),
            live_evidence_script: read_package_file(output_path, "live-evidence/collect.sh"),
            copied_correctness_report: read_package_file(output_path, "correctness-report.html"),
            manifest: read_package_file(output_path, "manifest.json"),
        }
    }

    pub(super) fn assert_contract(&self, config_path: &std::path::Path) {
        self.assert_buyer_facing_contents();
        self.assert_operational_contents(config_path);
        self.assert_fleet_contents();
        self.assert_lake_contents();
        self.assert_spark_contents();
        self.assert_manifest();
    }
}

fn read_package_file(output_path: &std::path::Path, relative_path: &str) -> String {
    fs::read_to_string(output_path.join(relative_path))
        .unwrap_or_else(|error| panic!("read package file {relative_path}: {error}"))
}

fn assert_content_contains(label: &str, content: &str, fragments: &[&str]) {
    for fragment in fragments {
        assert!(
            content.contains(fragment),
            "{label} missing expected fragment: {fragment}"
        );
    }
}
