use std::fmt::Write as _;

use crate::{
    checksum_status_label, push_affected_tables, push_boundary_proof, push_ddl_events,
    push_manifest, transaction_boundary_status_label, Result, TransactionInspectOutputFormat,
    TransactionInspectSummary,
};

pub(crate) fn render_transaction_inspect_summary(
    summary: &TransactionInspectSummary,
    format: TransactionInspectOutputFormat,
) -> Result<String> {
    match format {
        TransactionInspectOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        TransactionInspectOutputFormat::Text => Ok(render_transaction_inspect_text(summary)),
    }
}

fn render_transaction_inspect_text(summary: &TransactionInspectSummary) -> String {
    let boundary = &summary.transaction_boundary;
    let mut output = String::new();
    writeln!(&mut output, "Trellara transaction inspection").expect("write string");
    writeln!(
        &mut output,
        "transaction: {} status={} mode={}",
        summary.transaction_id,
        transaction_boundary_status_label(boundary.status),
        boundary.mode
    )
    .expect("write string");
    writeln!(
        &mut output,
        "source: {} database={} dataset={}",
        summary.source_id, summary.database_id, summary.dataset_id
    )
    .expect("write string");
    writeln!(
        &mut output,
        "lsn_boundary: begin={} commit={}",
        summary.begin_lsn, summary.commit_lsn
    )
    .expect("write string");
    writeln!(
        &mut output,
        "events: dml={} ddl={} source_total={} schema_versions={} checksum={} checksum_status={}",
        summary.event_count,
        summary.ddl_event_count,
        summary.source_event_count,
        summary.schema_version_count,
        summary.checksum,
        checksum_status_label(boundary.checksum_status)
    )
    .expect("write string");
    writeln!(&mut output, "guarantee: {}", boundary.guarantee).expect("write string");
    writeln!(
        &mut output,
        "visibility_contract: {}",
        boundary.visibility_contract
    )
    .expect("write string");

    push_ddl_events(&mut output, summary);

    push_boundary_proof(&mut output, boundary);
    push_affected_tables(&mut output, summary);
    push_manifest(&mut output, summary);

    output
}
