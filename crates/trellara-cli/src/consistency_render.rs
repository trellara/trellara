use std::fmt::Write as _;

use crate::{ConsistencyContractSummary, PilotGuideOutputFormat, Result};

pub(crate) fn render_consistency_contract_summary(
    summary: &ConsistencyContractSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_consistency_contract_text(summary)),
    }
}

pub(crate) fn render_consistency_contract_text(summary: &ConsistencyContractSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara consistency contract").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", summary.mode).expect("write string");
    writeln!(
        &mut output,
        "selected_consumer_mode: {}",
        summary.selected_consumer_mode
    )
    .expect("write string");
    writeln!(&mut output, "stream: {}", summary.stream_kind).expect("write string");
    writeln!(
        &mut output,
        "source_capture_contract: {}",
        summary.source_capture_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transaction_boundary_contract: {}",
        summary.transaction_boundary_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "source_ack_contract: {}",
        summary.source_ack_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "snapshot_handoff_contract: {}",
        summary.snapshot_handoff_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transport_durability_contract: {}",
        summary.transport_durability_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "consumer_visibility_contract: {}",
        summary.consumer_visibility_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "target_checkpoint_contract: {}",
        summary.target_checkpoint_contract
    )
    .expect("write string");
    writeln!(&mut output, "replay_contract: {}", summary.replay_contract).expect("write string");
    writeln!(&mut output, "reseed_contract: {}", summary.reseed_contract).expect("write string");
    writeln!(
        &mut output,
        "lake_visibility_contract: {}",
        summary.lake_visibility_contract
    )
    .expect("write string");

    output.push_str("\ntopics:\n");
    for topic in &summary.topics {
        writeln!(&mut output, "- {topic}").expect("write string");
    }

    if let Some(partition) = &summary.partition_contract {
        output.push_str("\npartition_contract:\n");
        writeln!(&mut output, "- key_column: {}", partition.key_column).expect("write string");
        writeln!(
            &mut output,
            "- partition_count: {}",
            partition.partition_count
        )
        .expect("write string");
        writeln!(
            &mut output,
            "- null_key_policy: {}",
            partition.null_key_policy
        )
        .expect("write string");
        writeln!(
            &mut output,
            "- key_change_policy: {}",
            partition.key_change_policy
        )
        .expect("write string");
        writeln!(
            &mut output,
            "- global_visibility_rule: {}",
            partition.global_visibility_rule
        )
        .expect("write string");
        writeln!(
            &mut output,
            "- partition_local_visibility_rule: {}",
            partition.partition_local_visibility_rule
        )
        .expect("write string");
    }

    output.push_str("\ninvariants:\n");
    for invariant in &summary.invariants {
        writeln!(&mut output, "- {}: {}", invariant.code, invariant.rule).expect("write string");
        writeln!(&mut output, "  proof: {}", invariant.proof_command).expect("write string");
    }

    output.push_str("\nproof_commands:\n");
    for command in &summary.proof_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
