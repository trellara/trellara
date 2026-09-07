use std::fmt::Write as _;

use crate::{IdentityAuditSummary, IdentityAuditTableStatus, PilotGuideOutputFormat, Result};

pub(crate) fn render_identity_audit_summary(
    summary: &IdentityAuditSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_identity_audit_text(summary)),
    }
}

pub(crate) fn render_identity_audit_text(summary: &IdentityAuditSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara identity audit").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "tables: {}", summary.table_count).expect("write string");
    writeln!(
        &mut output,
        "pk_apply_ready: {}",
        summary.pk_apply_ready_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "live_identity_review_required: {}",
        summary.live_identity_review_required_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "ordinary_pk_tables_do_not_require_full: {}",
        summary.ordinary_pk_tables_do_not_require_full
    )
    .expect("write string");
    writeln!(&mut output, "cdc_apply_gate: {}", summary.cdc_apply_gate).expect("write string");
    writeln!(
        &mut output,
        "toast_preservation_contract: {}",
        summary.toast_preservation_contract
    )
    .expect("write string");

    output.push_str("\napply_guarantees:\n");
    for guarantee in &summary.apply_guarantees {
        writeln!(&mut output, "- {}", guarantee.code).expect("write string");
        writeln!(&mut output, "  guarantee: {}", guarantee.guarantee).expect("write string");
        writeln!(
            &mut output,
            "  transaction_boundary: {}",
            guarantee.transaction_boundary
        )
        .expect("write string");
        writeln!(&mut output, "  proof_command: {}", guarantee.proof_command)
            .expect("write string");
    }

    output.push_str("\ntables:\n");
    for table in &summary.tables {
        writeln!(
            &mut output,
            "- [{}] {}",
            identity_audit_status_label(table.status),
            table.relation
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  configured_primary_key: {}",
            table.configured_primary_key.as_deref().unwrap_or("missing")
        )
        .expect("write string");
        writeln!(&mut output, "  apply_strategy: {}", table.apply_strategy).expect("write string");
        writeln!(
            &mut output,
            "  replica_identity_requirement: {}",
            table.replica_identity_requirement
        )
        .expect("write string");
        writeln!(&mut output, "  toast_handling: {}", table.toast_handling).expect("write string");
        if !table.target_owned_columns.is_empty() {
            writeln!(
                &mut output,
                "  target_owned_columns: {}",
                table.target_owned_columns.join(", ")
            )
            .expect("write string");
        }
    }

    output.push_str("\nproof_commands:\n");
    for command in &summary.proof_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nlive_evidence_required:\n");
    for evidence in &summary.live_evidence_required {
        writeln!(&mut output, "- {evidence}").expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}

fn identity_audit_status_label(status: IdentityAuditTableStatus) -> &'static str {
    match status {
        IdentityAuditTableStatus::PkApplyReady => "pk_apply_ready",
        IdentityAuditTableStatus::LiveIdentityReviewRequired => "live_identity_review_required",
    }
}
