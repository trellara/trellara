use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(super) fn pgoutput_schema_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_schema_change_during_stream",
            failure_point: "pgoutput sends changed relation metadata for a table already seen by the relay",
            invariant: "source_schema_fingerprint_fail_closed",
            boundary_mode: "pgoutput_relation_metadata",
            expected_safety_property:
                "capture stops before assembling changes under an unexpected relation schema",
            proof_command:
                "cargo test -p trellara-pg-capture pgoutput_decoder_fails_closed_on_relation_schema_change && cargo test -p trellara-pg-capture pgoutput_decoder_fails_closed_on_schema_change_during_stream",
            recovery_command: Some("trellara contract-test --config <flow>"),
            evidence:
                "trellara-pg-capture::pgoutput_decoder_fails_closed_on_schema_change_during_stream",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "pgoutput_row_without_relation_metadata",
            failure_point:
                "pgoutput sends a row message before the relay has relation metadata for that relation id",
            invariant: "relation_metadata_required_before_rows",
            boundary_mode: "pgoutput_relation_metadata",
            expected_safety_property:
                "capture fails closed before constructing a transaction envelope with unknown column identity",
            proof_command:
                "cargo test -p trellara-pg-capture pgoutput_decoder_fails_closed_without_relation_metadata",
            recovery_command: Some("trellara contract-test --config <flow>"),
            evidence: "trellara-pg-capture::pgoutput_decoder_fails_closed_without_relation_metadata",
        }),
        ChaosScenarioSummary::covered(ChaosScenarioInput {
            name: "source_schema_handoff_recovery",
            failure_point: "a pinned pgoutput schema fingerprint no longer matches live source metadata",
            invariant: "schema_drift_requires_fresh_snapshot_handoff",
            boundary_mode: "pgoutput_relation_metadata",
            expected_safety_property:
                "operators receive a scripted schema-discover, contract-test, fresh snapshot handoff, resume, and verify sequence before CDC continues",
            proof_command:
                "cargo test -p trellara-cli --lib source_schema_drift_recovery_action_requires_fresh_handoff && cargo test -p trellara-cli --lib contract_test_scripts_schema_handoff_when_pinned_fingerprint_drifts",
            recovery_command: Some("trellara repair-plan --config <flow>"),
            evidence: "trellara-cli::source_schema_drift_recovery_action_requires_fresh_handoff",
        }),
    ]
}
