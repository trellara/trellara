use crate::{ConsistencyInvariant, DatasetMode, StreamConfig, TrellaraConfig};

pub(crate) fn consistency_run_proof_command(config_path: &str, local_stream: bool) -> String {
    if local_stream {
        format!(
            "trellara run --local --verify --format text --config {config_path} --snapshot-run-id consistency-contract --max-transactions 100 --max-messages 100"
        )
    } else {
        format!("trellara run --verify --format text --config {config_path}")
    }
}

pub(crate) fn consistency_invariants(
    config: &TrellaraConfig,
    config_path: &str,
) -> Vec<ConsistencyInvariant> {
    let mut invariants = vec![
        ConsistencyInvariant {
            code: "source_ack_after_durable_publish".to_string(),
            rule: "CDC source acknowledgement never moves beyond the last completely durable Trellara transaction boundary".to_string(),
            proof_command: format!("trellara status --config {config_path} --view report --format text"),
        },
        ConsistencyInvariant {
            code: "checkpoint_after_target_apply".to_string(),
            rule: "target checkpoint advancement is downstream of idempotent apply and cannot skip quarantined or partially applied transactions".to_string(),
            proof_command: format!("trellara status --config {config_path} --view diagnostics --format text"),
        },
        ConsistencyInvariant {
            code: "snapshot_before_stream_replay".to_string(),
            rule: "initial copy completes at the exported snapshot boundary before CDC replay is trusted".to_string(),
            proof_command: format!("trellara snapshot --config {config_path} --run-id consistency-contract"),
        },
        ConsistencyInvariant {
            code: "transaction_inspection_available".to_string(),
            rule: "every captured transaction boundary can be inspected by checksum, affected tables, manifest mode, and commit LSN".to_string(),
            proof_command: "trellara inspect-transaction --file <envelope.pb> --format text".to_string(),
        },
    ];

    if matches!(config.stream, StreamConfig::Local { .. }) {
        invariants.push(ConsistencyInvariant {
            code: "local_replay_locates_exact_boundary".to_string(),
            rule: "brokerless replay must locate the exact transaction boundary before seeking a consumer cursor".to_string(),
            proof_command: format!(
                "trellara stream locate-local --config {config_path} --transaction-id <tx> --commit-lsn <lsn>"
            ),
        });
    }

    let partitioned = config.dataset.mode == DatasetMode::PartitionedScaleMode;
    if partitioned && matches!(config.stream, StreamConfig::Local { .. }) {
        invariants.push(ConsistencyInvariant {
            code: "local_partition_barrier_reconstructs_before_ack".to_string(),
            rule: "partitioned local stream consumers must reconstruct the manifest, commit marker, and every participating partition chunk before acknowledgement".to_string(),
            proof_command: format!(
                "trellara stream reconstruct-local --config {config_path} --transaction-id <tx> --commit-lsn <lsn>"
            ),
        });
    }

    if partitioned {
        invariants.push(ConsistencyInvariant {
            code: "partition_watermarks_gate_global_visibility".to_string(),
            rule: "global visibility in partitioned scale mode advances only after every participating partition has checkpoint evidence".to_string(),
            proof_command: format!("trellara partition-watermarks --config {config_path}"),
        });
    }

    invariants
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_proof_command_uses_bounded_local_trial_for_local_streams() {
        assert_eq!(
            consistency_run_proof_command("flow.yml", true),
            "trellara run --local --verify --format text --config flow.yml --snapshot-run-id consistency-contract --max-transactions 100 --max-messages 100"
        );
        assert_eq!(
            consistency_run_proof_command("flow.yml", false),
            "trellara run --verify --format text --config flow.yml"
        );
    }
}
