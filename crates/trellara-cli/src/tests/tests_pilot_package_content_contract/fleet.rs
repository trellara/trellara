use super::*;

impl PilotPackageContents {
    pub(super) fn assert_fleet_contents(&self) {
        assert_content_contains(
            "fleet report",
            &self.fleet_report,
            &[
                "Trellara fleet report",
                "flows: 1",
                "convergence_gates:",
                "target_convergence",
                "local_stream_durability",
                "recovery_drills:",
                "target_quarantine_replay",
                "checksum_reseed",
                "source_wal_loss_reseed",
                "schema_handoff_refresh",
                "lake_offline_source_gap_acceptance",
                "lake_late_source_recompute",
                "lake_conflicting_duplicate_quarantine",
            ],
        );
        assert_content_contains(
            "fleet scorecard",
            &self.fleet_scorecard,
            &[
                "Trellara fleet scorecard",
                "verdict: ready_for_design_partner_fleet_review",
                "flow_status: 1 ready, 0 review_required, 0 blocked",
                "convergence_gates: 6 total, 0 blocked_by_config",
                "recovery_drills: 7",
                "review_sequence:",
            ],
        );
        assert_content_contains(
            "fleet evidence plan",
            &self.fleet_evidence_plan,
            &[
                "Trellara fleet evidence plan",
                "verdict: ready_to_collect_live_evidence",
                "trellara check --config",
                "transaction_boundary",
            ],
        );
        assert_content_contains(
            "consistency contract",
            &self.consistency_contract,
            &[
                "\"source_ack_contract\"",
                "\"snapshot_handoff_contract\"",
                "\"target_checkpoint_contract\"",
                "\"transaction_boundary_contract\"",
                "source_ack_after_durable_publish",
            ],
        );
        assert_content_contains(
            "performance envelope",
            &self.performance_envelope,
            &[
                "\"quickstart_time_budget_minutes\": 10",
                "\"stream_spill_threshold_changes\": 1024",
                "\"stream_spill_threshold_max_changes\": 1000000",
                "\"transaction_boundary_cost\"",
                "streamed_transaction_spill",
                "\"measurement_note\"",
            ],
        );
        assert_content_contains(
            "identity audit",
            &self.identity_audit,
            &[
                "\"ordinary_pk_tables_do_not_require_full\": true",
                "\"cdc_apply_gate\": \"released: configured primary-key apply",
                "\"pk_apply_ready_count\": 1",
                "REPLICA IDENTITY FULL is not required",
                "absent non-key columns are treated as unchanged",
                "plans_delete_with_key_predicate",
            ],
        );
        assert_content_contains(
            "consumer semantics",
            &self.consumer_semantics,
            &[
                "\"strict_language\"",
                "\"partitioned_language\"",
                "\"mode\": \"exact_transaction\"",
                "\"mode\": \"partition_local\"",
                "atomic visibility for cross-partition transactions",
            ],
        );
        assert_content_contains(
            "fleet control plane",
            &self.fleet_control_plane,
            &[
                "\"verdict\": \"validate_with_design_partners\"",
                "\"code\": \"read_only_fleet_topology\"",
                "\"code\": \"evidence_package_registry\"",
                "\"code\": \"deployment_orchestration\"",
                "\"status\": \"defer\"",
            ],
        );
    }
}
