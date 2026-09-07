#[path = "pilot_evidence_transaction_markers/json.rs"]
mod json;
#[path = "pilot_evidence_transaction_markers/json_ack.rs"]
mod json_ack;
#[path = "pilot_evidence_transaction_markers/json_helpers.rs"]
mod json_helpers;
#[path = "pilot_evidence_transaction_markers/manifest.rs"]
mod manifest;
#[path = "pilot_evidence_transaction_markers/manifest_partitions.rs"]
mod manifest_partitions;
#[path = "pilot_evidence_transaction_markers/text.rs"]
mod text;
#[path = "pilot_evidence_transaction_markers/text_ack.rs"]
mod text_ack;
#[path = "pilot_evidence_transaction_markers/text_values.rs"]
mod text_values;

pub(crate) fn transaction_boundary_marker_present(marker: &str, contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    let compact = compact(&lower);

    match marker {
        "source dataset identity" => {
            json::source_dataset_identity_present(contents)
                || text::source_dataset_identity_present(&lower)
        }
        "transaction_boundary verified" => {
            json::transaction_boundary_verified(contents)
                || text::transaction_boundary_verified(&lower, &compact)
        }
        "source checkpoint evidence" => {
            json::checkpoint_evidence(contents, "source_checkpoint")
                || text::checkpoint_evidence(&lower, "source_checkpoint")
        }
        "target checkpoint evidence" => {
            json::checkpoint_evidence(contents, "target_checkpoint")
                || text::checkpoint_evidence(&lower, "target_checkpoint")
        }
        "source_ack_lsn evidence" => {
            json::source_ack_lsn_evidence(contents) || text::source_ack_lsn_evidence(&lower)
        }
        "source ack after durable publish" => {
            json::source_ack_after_durable_publish(contents)
                || text::source_ack_after_durable_publish(&lower, &compact)
        }
        "source ack publish destinations" => {
            json::source_ack_publish_destinations(contents)
                || text::source_ack_publish_destinations(&lower, &compact)
        }
        "parallel replay contract" => {
            json::parallel_replay_contract(contents) || text::parallel_replay_contract(&lower)
        }
        "partitioned manifest evidence headers" => {
            manifest::manifest_evidence_headers(contents, &lower)
        }
        _ => false,
    }
}

fn compact(lower: &str) -> String {
    lower
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect()
}
