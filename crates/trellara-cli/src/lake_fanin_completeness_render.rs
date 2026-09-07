use std::fmt::Write as _;

use crate::{
    lake_completeness_state_label, LakeFaninCompletenessDecision, LakeFaninCompletenessSummary,
    QuickstartOutputFormat, Result,
};

pub(crate) fn render_lake_fanin_completeness_summary(
    summary: &LakeFaninCompletenessSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_lake_fanin_completeness_text(summary)),
    }
}

fn render_lake_fanin_completeness_text(summary: &LakeFaninCompletenessSummary) -> String {
    let mut output = String::new();
    push_header(&mut output, summary);
    push_table_refs(&mut output, "raw_cdc_tables", &summary.raw_cdc_tables);
    push_table_refs(
        &mut output,
        "epoch_metadata_tables",
        &summary.epoch_metadata_tables,
    );
    push_spark_templates(&mut output, summary);
    push_list(&mut output, "proof_artifacts", &summary.proof_artifacts);
    push_list(&mut output, "proof_commands", &summary.proof_commands);
    push_list(&mut output, "recovery_path", &summary.recovery_path);
    push_list(
        &mut output,
        "deferred_sink_work",
        &summary.deferred_sink_work,
    );
    writeln!(
        &mut output,
        "verification: status={} matched={} mismatches={} warnings={} blockers={}",
        verify_status_label(summary.verification.status),
        summary.verification.matched_check_count,
        summary.verification.mismatch_count,
        summary.verification.warning_mismatch_count,
        summary.verification.blocker_mismatch_count
    )
    .expect("write string");
    output
}

fn push_header(output: &mut String, summary: &LakeFaninCompletenessSummary) {
    writeln!(output, "Trellara lake fan-in completeness").expect("write string");
    writeln!(
        output,
        "dataset: {} source={} mode={}",
        summary.dataset_id, summary.source_id, summary.mode
    )
    .expect("write string");
    writeln!(output, "positioning: {}", summary.positioning).expect("write string");
    writeln!(
        output,
        "epoch: {} decision={} state={} accepted_gaps={} spark_consumption_allowed={}",
        summary.epoch_id,
        decision_label(summary.decision),
        lake_completeness_state_label(summary.completeness_state),
        summary.accepted_complete_with_gaps,
        summary.spark_consumption_allowed
    )
    .expect("write string");
    writeln!(
        output,
        "spark_consumption_gate: {}",
        summary.spark_consumption_gate
    )
    .expect("write string");
    writeln!(
        output,
        "spark_consumption_contract: {}",
        summary.spark_consumption_contract
    )
    .expect("write string");
    writeln!(
        output,
        "sources: required={} complete={} lagging={} missing={} quarantined={} reseeding={} unknown={}",
        summary.required_source_count,
        summary.source_state_counts.complete,
        summary.source_state_counts.lagging,
        summary.source_state_counts.missing,
        summary.source_state_counts.quarantined,
        summary.source_state_counts.reseeding,
        summary.source_state_counts.unknown
    )
    .expect("write string");
    writeln!(
        output,
        "changes: transactions={} changes={} checksum_rollup={} manifest_digest={}",
        summary.transaction_count,
        summary.change_count,
        summary.checksum_rollup,
        summary.manifest_digest
    )
    .expect("write string");
}

fn push_table_refs(
    output: &mut String,
    heading: &str,
    tables: &[crate::LakeFaninCompletenessTableRef],
) {
    writeln!(output, "{heading}:").expect("write string");
    for table in tables {
        writeln!(
            output,
            "- {} table={}",
            table.materialization, table.table_name
        )
        .expect("write string");
    }
}

fn push_spark_templates(output: &mut String, summary: &LakeFaninCompletenessSummary) {
    writeln!(output, "spark_templates:").expect("write string");
    for template in &summary.spark_templates {
        writeln!(
            output,
            "- {} artifact={} command={} gate={}",
            template.kind, template.artifact, template.command, template.gate
        )
        .expect("write string");
    }
}

fn push_list(output: &mut String, heading: &str, values: &[String]) {
    writeln!(output, "{heading}:").expect("write string");
    for value in values {
        writeln!(output, "- {value}").expect("write string");
    }
}

fn decision_label(decision: LakeFaninCompletenessDecision) -> &'static str {
    match decision {
        LakeFaninCompletenessDecision::Ready => "ready",
        LakeFaninCompletenessDecision::ReadyWithAcceptedGaps => "ready_with_accepted_gaps",
        LakeFaninCompletenessDecision::BlockedNeedsGapAcceptance => "blocked_needs_gap_acceptance",
        LakeFaninCompletenessDecision::BlockedVerificationMismatch => {
            "blocked_verification_mismatch"
        }
        LakeFaninCompletenessDecision::BlockedNonConsumableEpoch => "blocked_non_consumable_epoch",
    }
}

fn verify_status_label(status: crate::LakeFaninVerifyStatus) -> &'static str {
    match status {
        crate::LakeFaninVerifyStatus::Match => "match",
        crate::LakeFaninVerifyStatus::Mismatch => "mismatch",
        crate::LakeFaninVerifyStatus::Blocked => "blocked",
    }
}
