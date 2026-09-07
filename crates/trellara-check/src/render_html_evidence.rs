use std::fmt::Write as _;

use crate::{html_escape::push_html_escaped, slot_evidence::slot_summary_line, CheckSummary};

pub(crate) fn push_slot_evidence(html: &mut String, summary: &CheckSummary) {
    html.push_str("<section class=\"card\"><h2>Evidence</h2>");
    if summary.logical_slots.is_empty() {
        html.push_str("<p>No logical replication slots were found.</p>");
    } else {
        for slot in &summary.logical_slots {
            html.push_str("<p class=\"evidence\">");
            push_html_escaped(html, &slot_summary_line(slot));
            html.push_str("</p>");
        }
    }
    html.push_str("</section>");
}

pub(crate) fn push_subscription_conflicts(html: &mut String, summary: &CheckSummary) {
    if summary.subscription_conflicts.is_empty() {
        return;
    }
    html.push_str("<section class=\"card\"><h2>Subscription Conflicts</h2><ul>");
    for stat in &summary.subscription_conflicts {
        write!(
            html,
            "<li>{} apply_errors={} sync_errors={} conflicts={} update_missing={} delete_missing={}</li>",
            html_text(&stat.subscription_name),
            stat.apply_error_count,
            stat.sync_error_count,
            stat.conflicts.total(),
            stat.conflicts.update_missing,
            stat.conflicts.delete_missing
        )
        .expect("write string");
    }
    html.push_str("</ul></section>");
}

fn html_text(value: &str) -> String {
    let mut escaped = String::new();
    push_html_escaped(&mut escaped, value);
    escaped
}
