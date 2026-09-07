use super::*;

impl PilotPackageContents {
    pub(super) fn assert_operational_contents(&self, config_path: &std::path::Path) {
        assert_content_contains(
            "quickstart readiness",
            &self.readiness,
            &[
                "trellara check --config",
                "trellara run --local --verify",
                "trellara status --config",
                "--view report",
                "trellara pilot-package --config",
                "target/trellara-quickstart-evidence",
                "recovery: trellara status --config",
                "--view diagnostics",
            ],
        );
        assert_content_contains(
            "quickstart plan",
            &self.plan,
            &["trellara run --local --verify"],
        );
        assert_content_contains(
            "pilot guide",
            &self.guide,
            &[
                "large_transaction_evidence:",
                "pilot can run without adopting Kafka",
            ],
        );
        assert_content_contains(
            "pilot guide json",
            &self.guide_json,
            &["\"capture_spill_boundary\""],
        );
        assert_content_contains(
            "scorecard",
            &self.scorecard,
            &[
                "Trellara pilot scorecard",
                "bounded_memory_capture",
                "bounded_memory_contract",
                "visibility_contract",
            ],
        );
        assert_content_contains(
            "scorecard json",
            &self.scorecard_json,
            &["\"verdict\": \"ready_for_live_pilot\""],
        );
        assert_content_contains(
            "local run proof",
            &self.local_run_proof,
            &[
                "Trellara Local Run Proof",
                "trellara run --local --verify --format text",
                "snapshot_handoff_boundary",
                "bounded_large_transaction_capture",
                "bounded_memory_contract",
                "visibility_contract",
                "source_ack_after_local_durability",
                "source_ack_lsn",
                "source ACK publish destinations matched",
                "every Trellara publish ack is durable",
                "snapshot_handoff_blocker_codes",
                "snapshot_handoff_recovery_actions",
                "target_apply_checkpoint",
                "barrier_pending_blockers",
                "barrier_pending_blocker_codes",
                "barrier_pending_recovery_actions",
                "target relation identity convergence",
            ],
        );
        assert_content_contains(
            "source safety checklist",
            &self.source_safety_checklist,
            &[
                "Trellara Source Safety Checklist",
                "trellara check --config",
                "trellara-check",
                "source-safety.html",
                "exact read-only SQL",
                "--write-init",
                "wal_status",
                "failover slot",
                "replica identity",
                "source.wal_retention_warn_bytes",
            ],
        );
        assert_content_contains(
            "deployment guide",
            &self.deployment_guide,
            &[
                "Trellara Design-Partner Deployment Guide",
                "Deployment Steps",
                "Go/No-Go Rule",
            ],
        );
        assert_content_contains(
            "operational burden",
            &self.operational_burden,
            &[
                "Operational Burden Notes",
                "Before Trellara",
                "With This Pilot",
            ],
        );
        assert_content_contains(
            "feature pull",
            &self.feature_pull,
            &[
                "Design-Partner Feature Pull List",
                "Validate Before Building Cloud",
                "Current Gate Status",
            ],
        );
        assert_content_contains(
            "live evidence README",
            &self.live_evidence_readme,
            &[
                "Trellara Live Evidence Collection",
                "source-safety.txt",
                "transaction-boundary.txt",
                "verified-apply.txt",
                "ddl-release-proof.json",
                "release_dml true",
                "ack_commands",
            ],
        );
        assert_content_contains(
            "live evidence script",
            &self.live_evidence_script,
            &[
                "set -eu",
                "trellara check --config",
                "trellara status --config",
                "trellara pilot-package --config",
                "ddl-release-proof-package/ddl-release-proof.json",
                "trellara pilot evidence-check --config",
            ],
        );
        assert_content_contains(
            "diagnostics",
            &self.diagnostics,
            &[
                "Trellara diagnostics",
                "attachment_commands:",
                &config_path.display().to_string(),
            ],
        );
        assert_content_contains(
            "diagnostics json",
            &self.diagnostics_json,
            &["\"repair_plan\"", "\"metrics\"", "\"attachment_commands\""],
        );
        assert_content_contains(
            "copied correctness report",
            &self.copied_correctness_report,
            &["Trellara correctness report"],
        );
    }
}
