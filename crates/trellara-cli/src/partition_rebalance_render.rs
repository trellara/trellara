use std::fmt::Write as _;

use trellara_protocol::{PartitionRebalancePlan, PartitionRebalanceStatus};

use crate::{PilotGuideOutputFormat, Result};

pub(crate) fn render_partition_rebalance_plan(
    plan: &PartitionRebalancePlan,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(plan)?),
        PilotGuideOutputFormat::Text => Ok(render_partition_rebalance_text(plan)),
    }
}

fn render_partition_rebalance_text(plan: &PartitionRebalancePlan) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara partition rebalance plan").expect("write string");
    writeln!(&mut output, "source_id: {}", plan.source_id).expect("write string");
    writeln!(&mut output, "dataset_id: {}", plan.dataset_id).expect("write string");
    writeln!(&mut output, "status: {}", status_label(plan.status)).expect("write string");
    writeln!(
        &mut output,
        "runtime_movement_allowed: {}",
        plan.runtime_movement_allowed
    )
    .expect("write string");
    writeln!(
        &mut output,
        "visibility_contract: {}",
        plan.visibility_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "event_count_range: {}..{}",
        option_u64(plan.min_event_count),
        option_u64(plan.max_event_count)
    )
    .expect("write string");
    writeln!(&mut output, "total_event_count: {}", plan.total_event_count).expect("write string");
    writeln!(&mut output, "max_skew_percent: {}", plan.max_skew_percent).expect("write string");
    writeln!(
        &mut output,
        "skew_ratio_basis_points: {}",
        option_u64(plan.skew_ratio_basis_points)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "missing_partitions: {}",
        csv_u32(&plan.missing_partitions)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "blocking_partitions: {}",
        csv_u32(&plan.blocking_partition_ids)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "recommended_moves: {}",
        plan.recommended_moves.len()
    )
    .expect("write string");
    for candidate in &plan.recommended_moves {
        writeln!(
            &mut output,
            "- from_partition_id: {} to_partition_id: {} estimated_event_delta: {} reason: {}",
            candidate.from_partition_id,
            candidate.to_partition_id,
            candidate.estimated_event_delta,
            candidate.reason
        )
        .expect("write string");
    }
    output
}

fn status_label(status: PartitionRebalanceStatus) -> &'static str {
    match status {
        PartitionRebalanceStatus::IncompleteEvidence => "incomplete_evidence",
        PartitionRebalanceStatus::Stable => "stable",
        PartitionRebalanceStatus::Skewed => "skewed",
    }
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn csv_u32(values: &[u32]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
