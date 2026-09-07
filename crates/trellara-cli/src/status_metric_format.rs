use crate::{ChecksumStatus, CorrectnessProofStatus, FlowHealthStatus, TransactionBoundaryStatus};

pub(crate) fn push_metric<T: std::fmt::Display>(
    output: &mut String,
    name: &str,
    labels: &[(&str, &str)],
    value: T,
) {
    output.push_str(name);
    output.push('{');
    for (index, (key, value)) in labels.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(key);
        output.push_str("=\"");
        push_escaped_label_value(output, value);
        output.push('"');
    }
    output.push_str("} ");
    output.push_str(&value.to_string());
    output.push('\n');
}

fn push_escaped_label_value(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            _ => output.push(character),
        }
    }
}

pub(crate) fn bool_value(value: bool) -> u8 {
    u8::from(value)
}

pub(crate) fn flow_health_status_label(status: FlowHealthStatus) -> &'static str {
    match status {
        FlowHealthStatus::Healthy => "healthy",
        FlowHealthStatus::Degraded => "degraded",
        FlowHealthStatus::Blocked => "blocked",
    }
}

pub(crate) fn checksum_status_label(status: ChecksumStatus) -> &'static str {
    match status {
        ChecksumStatus::Unknown => "unknown",
        ChecksumStatus::Match => "match",
        ChecksumStatus::Mismatch => "mismatch",
    }
}

pub(crate) fn transaction_boundary_status_label(status: TransactionBoundaryStatus) -> &'static str {
    match status {
        TransactionBoundaryStatus::Verified => "verified",
        TransactionBoundaryStatus::PendingEvidence => "pending_evidence",
        TransactionBoundaryStatus::AtRisk => "at_risk",
    }
}

pub(crate) fn correctness_proof_status_label(status: CorrectnessProofStatus) -> &'static str {
    match status {
        CorrectnessProofStatus::Verified => "verified",
        CorrectnessProofStatus::AtRisk => "at_risk",
        CorrectnessProofStatus::MissingEvidence => "missing_evidence",
    }
}
