use std::fmt::Write as _;

use crate::{
    html_escape::push_html_escaped,
    labels::{grade_label, severity_label, status_label},
    render_html_evidence::{push_slot_evidence, push_subscription_conflicts},
    render_html_queries::push_query_appendix,
    CheckSeverity, CheckStatus, CheckSummary,
};

pub(crate) fn render_check_html(summary: &CheckSummary) -> String {
    let mut html = String::new();
    push_head(&mut html);
    push_header(&mut html, summary);
    push_facts(&mut html, summary);
    push_findings(&mut html, summary);
    push_actions(&mut html, summary);
    push_slot_evidence(&mut html, summary);
    push_subscription_conflicts(&mut html, summary);
    push_query_appendix(&mut html);
    html.push_str("</main></body></html>");
    html
}

fn push_head(html: &mut String) {
    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    html.push_str("<title>Trellara check report</title><style>");
    html.push_str("body{margin:0;background:#f6f7f9;color:#172026;font:15px/1.5 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}main{max-width:1120px;margin:0 auto;padding:32px 20px 48px}.hero,.card{background:#fff;border:1px solid #d9e0e7;border-radius:8px}.hero{display:flex;justify-content:space-between;gap:24px;padding:28px}.eyebrow{margin:0 0 8px;color:#586673;font-size:12px;font-weight:700;text-transform:uppercase}h1,h2,h3,p{margin-top:0}h1{font-size:30px;line-height:1.15;margin-bottom:10px}h2{font-size:20px;margin-bottom:14px}.score{min-width:152px;border-radius:8px;padding:18px;text-align:center;color:#fff}.score strong{display:block;font-size:52px;line-height:1}.score span,.score b{display:block}.healthy{background:#147d64}.degraded{background:#a7660b}.blocked{background:#a33a2b}.grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;margin:18px 0}.card{padding:20px;margin-top:16px}.facts{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}.fact span{display:block;color:#586673;font-size:12px}.fact strong{display:block;font-size:16px;word-break:break-word}.evidence{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;background:#eef2f5;border-radius:6px;padding:12px;word-break:break-word}.finding{border:1px solid #d9e0e7;border-left:5px solid #a7660b;border-radius:8px;padding:14px;margin:10px 0;background:#fff}.finding.critical{border-left-color:#a33a2b}.pill{display:inline-block;border-radius:999px;padding:2px 8px;margin-right:8px;background:#edf1f4;color:#3d4a55;font-size:12px;font-weight:700}.pill.critical{background:#f8ded9;color:#8c2f22}.pill.warning{background:#fff2d8;color:#87590a}ol,ul{padding-left:20px}details{border:1px solid #d9e0e7;border-radius:8px;background:#fff;margin:10px 0;padding:12px}summary{cursor:pointer;font-weight:700}summary small{display:block;color:#586673;font-weight:400}pre{overflow:auto;background:#172026;color:#f7fafc;border-radius:6px;padding:12px}code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:13px}@media(max-width:760px){.hero,.grid{display:block}.score{margin-top:18px}.facts{grid-template-columns:1fr}}");
    html.push_str("</style></head><body><main>");
}

fn push_header(html: &mut String, summary: &CheckSummary) {
    html.push_str("<header class=\"hero\"><div><p class=\"eyebrow\">Trellara check report</p>");
    html.push_str("<h1>PostgreSQL CDC source safety</h1><p>");
    push_html_escaped(html, &summary.database);
    html.push_str("</p><p>");
    push_html_escaped(html, &summary.managed_postgres_ready);
    html.push_str("</p></div><div class=\"score ");
    html.push_str(status_class(summary.status));
    html.push_str("\"><span>Grade</span><strong>");
    html.push_str(grade_label(summary.grade));
    write!(html, "</strong><b>{}/100</b></div></header>", summary.score).expect("write string");
}

fn push_facts(html: &mut String, summary: &CheckSummary) {
    html.push_str(
        "<section class=\"grid\"><div class=\"card\"><h2>Score</h2><div class=\"facts\">",
    );
    push_fact(html, "Status", status_label(summary.status));
    push_fact(html, "Grade", grade_label(summary.grade));
    push_fact(
        html,
        "Read only",
        if summary.read_only { "true" } else { "false" },
    );
    push_fact(html, "Tables checked", &summary.table_count.to_string());
    push_fact(
        html,
        "Unsafe tables",
        &summary.unsafe_table_count.to_string(),
    );
    push_fact(
        html,
        "Logical slots",
        &summary.logical_slot_count.to_string(),
    );
    push_fact(
        html,
        "At-risk slots",
        &summary.at_risk_slot_count.to_string(),
    );
    push_fact(
        html,
        "Subscriptions with stats",
        &summary.subscription_conflict_count.to_string(),
    );
    html.push_str("</div></div><div class=\"card\"><h2>Executive Readout</h2>");
    if summary.findings.is_empty() {
        html.push_str(
            "<p>No findings. The inspected source is clear for this one-shot diagnostic.</p>",
        );
    } else {
        html.push_str("<p>This database has source-safety findings that can block or degrade CDC. Recommended actions and evidence are below.</p>");
    }
    html.push_str("</div></section>");
}

fn push_findings(html: &mut String, summary: &CheckSummary) {
    html.push_str("<section class=\"card\"><h2>Findings</h2>");
    if summary.findings.is_empty() {
        html.push_str("<p>No findings.</p>");
    }
    for factor in &summary.findings {
        html.push_str("<article class=\"finding ");
        html.push_str(severity_class(factor.severity));
        html.push_str("\"><h3><span class=\"pill ");
        html.push_str(severity_class(factor.severity));
        html.push_str("\">");
        push_html_escaped(html, severity_label(factor.severity));
        html.push_str("</span>");
        push_html_escaped(html, &factor.code);
        write!(
            html,
            " <span class=\"pill\">-{} pts</span></h3><p>",
            factor.points_lost
        )
        .expect("write string");
        push_html_escaped(html, &factor.evidence);
        html.push_str("</p><p><strong>Recommended action:</strong> ");
        push_html_escaped(html, &factor.recommendation);
        html.push_str("</p></article>");
    }
    html.push_str("</section>");
}

fn push_actions(html: &mut String, summary: &CheckSummary) {
    html.push_str("<section class=\"card\"><h2>Recommended Actions</h2>");
    if summary.recommended_actions.is_empty() {
        html.push_str("<p>No action required by this report.</p>");
    } else {
        html.push_str("<ol>");
        for action in &summary.recommended_actions {
            html.push_str("<li>");
            push_html_escaped(html, action);
            html.push_str("</li>");
        }
        html.push_str("</ol>");
    }
    html.push_str("</section>");
}

fn push_fact(html: &mut String, label: &str, value: &str) {
    html.push_str("<div class=\"fact\"><span>");
    push_html_escaped(html, label);
    html.push_str("</span><strong>");
    push_html_escaped(html, value);
    html.push_str("</strong></div>");
}

fn status_class(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Healthy => "healthy",
        CheckStatus::Degraded => "degraded",
        CheckStatus::Blocked => "blocked",
    }
}

fn severity_class(severity: CheckSeverity) -> &'static str {
    match severity {
        CheckSeverity::Warning => "warning",
        CheckSeverity::Critical => "critical",
    }
}
