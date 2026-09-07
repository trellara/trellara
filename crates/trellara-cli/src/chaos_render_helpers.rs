pub(crate) fn push_metric_card(html: &mut String, label: &str, value: &str) {
    html.push_str("<div class=\"metric\"><span>");
    push_html_escaped(html, label);
    html.push_str("</span><strong>");
    push_html_escaped(html, value);
    html.push_str("</strong></div>");
}

pub(crate) fn push_key_value(html: &mut String, label: &str, value: &str) {
    html.push_str("<p><strong>");
    push_html_escaped(html, label);
    html.push_str(":</strong> <code>");
    push_html_escaped(html, value);
    html.push_str("</code></p>");
}

pub(crate) fn push_status(html: &mut String, passed: bool) {
    if passed {
        html.push_str("<span class=\"status ok\">pass</span>");
    } else {
        html.push_str("<span class=\"status risk\">at risk</span>");
    }
}

pub(crate) fn lake_completeness_state_label(
    state: trellara_lake::LakeCompletenessState,
) -> &'static str {
    match state {
        trellara_lake::LakeCompletenessState::Open => "open",
        trellara_lake::LakeCompletenessState::Sealing => "sealing",
        trellara_lake::LakeCompletenessState::Complete => "complete",
        trellara_lake::LakeCompletenessState::CompleteWithGaps => "complete_with_gaps",
        trellara_lake::LakeCompletenessState::Quarantined => "quarantined",
        trellara_lake::LakeCompletenessState::Reseeding => "reseeding",
        trellara_lake::LakeCompletenessState::FailedRecoverable => "failed_recoverable",
    }
}

pub(crate) fn lake_epoch_verification_status_label(
    status: trellara_lake::LakeEpochVerificationStatus,
) -> &'static str {
    match status {
        trellara_lake::LakeEpochVerificationStatus::Match => "match",
        trellara_lake::LakeEpochVerificationStatus::Mismatch => "mismatch",
        trellara_lake::LakeEpochVerificationStatus::Unknown => "unknown",
    }
}

pub(crate) fn lake_epoch_source_state_label(
    state: trellara_lake::LakeEpochSourceState,
) -> &'static str {
    match state {
        trellara_lake::LakeEpochSourceState::Complete => "complete",
        trellara_lake::LakeEpochSourceState::Lagging => "lagging",
        trellara_lake::LakeEpochSourceState::Missing => "missing",
        trellara_lake::LakeEpochSourceState::Quarantined => "quarantined",
        trellara_lake::LakeEpochSourceState::Reseeding => "reseeding",
    }
}

pub(crate) fn push_html_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
}
