use std::fmt::Write as _;

use trellara_pg_capture::SubscriptionConflictStats;

use crate::push_html_escaped;

pub(crate) fn push_source_safety_subscription_conflicts(
    html: &mut String,
    stats: &[SubscriptionConflictStats],
) {
    if stats.is_empty() {
        return;
    }
    html.push_str("<h3>Subscription Conflicts</h3><ul>");
    for stat in stats {
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
    html.push_str("</ul>");
}

fn html_text(value: &str) -> String {
    let mut escaped = String::new();
    push_html_escaped(&mut escaped, value);
    escaped
}
