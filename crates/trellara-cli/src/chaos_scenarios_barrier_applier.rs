use crate::{ChaosScenarioInput, ChaosScenarioSummary};

pub(crate) fn barrier_applier_scenarios() -> Vec<ChaosScenarioSummary> {
    vec![ChaosScenarioSummary::covered(ChaosScenarioInput {
        name: "partition_applier_before_ack",
        failure_point: "partitioned applier commits the reconstructed transaction but exits before stream acknowledgement",
        invariant: "manifest_checkpoint_same_transaction_as_apply",
        boundary_mode: "partitioned_scale_mode",
        expected_safety_property:
            "partition checkpoints and dedup state make redelivery safe without partial visibility",
        proof_command:
            "cargo test -p trellara-apply-postgres manifest_envelope_records_partition_checkpoints_atomically",
        recovery_command: Some("trellara status --config <flow> --view dashboard --format text"),
        evidence: "trellara-apply-postgres::manifest_envelope_records_partition_checkpoints_atomically",
    })]
}
