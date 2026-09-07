pub(crate) mod evidence;
pub(crate) mod queries;

use std::fmt::Write as _;

use crate::{
    flow_alert_severity_label, flow_health_status_label, push_html_escaped,
    push_source_safety_query_appendix, push_source_safety_subscription_conflicts,
    source_safety_grade_label, source_slot_summary_line, FlowAlertSeverity, FlowHealthStatus,
    SourceSafetyFactor, SourceSafetyInitRecommendation, SourceSafetyTextInput,
};

pub(crate) fn render_source_safety_html(input: SourceSafetyTextInput<'_>) -> String {
    let mut html = String::new();
    push_head(&mut html);
    push_header(&mut html, &input);
    push_facts(&mut html, &input);
    push_findings(&mut html, input.factors);
    push_recommended_actions(&mut html, input.recommended_actions);
    push_init_recommendation(
        &mut html,
        input.init_recommendation,
        input.init_config_written,
    );
    push_source_safety_query_appendix(&mut html);
    html.push_str("</main></body></html>");
    html
}

fn push_head(html: &mut String) {
    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
    html.push_str("<title>Trellara source safety report</title><style>");
    html.push_str("body{margin:0;background:#f6f7f9;color:#172026;font:15px/1.5 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}main{max-width:1120px;margin:0 auto;padding:32px 20px 48px}.hero,.card{background:#fff;border:1px solid #d9e0e7;border-radius:8px}.hero{display:flex;justify-content:space-between;gap:24px;padding:28px}.eyebrow{margin:0 0 8px;color:#586673;font-size:12px;font-weight:700;text-transform:uppercase}h1,h2,h3,p{margin-top:0}h1{font-size:30px;line-height:1.15;margin-bottom:10px}h2{font-size:20px;margin-bottom:14px}.score{min-width:152px;border-radius:8px;padding:18px;text-align:center;color:#fff}.score strong{display:block;font-size:52px;line-height:1}.score span,.score b{display:block}.score.healthy{background:#147d64}.score.degraded{background:#a7660b}.score.blocked{background:#a33a2b}.grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;margin:18px 0}.card{padding:20px}.facts{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}.fact span{display:block;color:#586673;font-size:12px}.fact strong{display:block;font-size:16px;word-break:break-word}.slot{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;background:#eef2f5;border-radius:6px;padding:12px;word-break:break-word}.finding{border:1px solid #d9e0e7;border-left:5px solid #a7660b;border-radius:8px;padding:14px;margin:10px 0;background:#fff}.finding.critical{border-left-color:#a33a2b}.pill{display:inline-block;border-radius:999px;padding:2px 8px;margin-right:8px;background:#edf1f4;color:#3d4a55;font-size:12px;font-weight:700}.pill.critical{background:#f8ded9;color:#8c2f22}.pill.warning{background:#fff2d8;color:#87590a}ol,ul{padding-left:20px}details{border:1px solid #d9e0e7;border-radius:8px;background:#fff;margin:10px 0;padding:12px}summary{cursor:pointer;font-weight:700}summary small{display:block;color:#586673;font-weight:400}pre{overflow:auto;background:#172026;color:#f7fafc;border-radius:6px;padding:12px}code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:13px}@media(max-width:760px){.hero,.grid{display:block}.score{margin-top:18px}.card{margin-top:14px}.facts{grid-template-columns:1fr}}");
    html.push_str("</style></head><body><main>");
}

fn push_header(html: &mut String, input: &SourceSafetyTextInput<'_>) {
    write!(
        html,
        "<header class=\"hero\"><div><p class=\"eyebrow\">Trellara source safety report</p><h1>"
    )
    .expect("write string");
    push_html_escaped(html, input.source_id);
    html.push_str("</h1><p>Read-only CDC readiness report for dataset ");
    push_html_escaped(html, input.dataset_id);
    html.push_str(" in mode ");
    push_html_escaped(html, input.mode);
    html.push_str(".</p></div><div class=\"score ");
    html.push_str(status_class(input.status));
    html.push_str("\"><span>Grade</span><strong>");
    html.push_str(source_safety_grade_label(input.grade));
    write!(html, "</strong><b>{}/100</b></div></header>", input.score).expect("write string");
}

fn push_facts(html: &mut String, input: &SourceSafetyTextInput<'_>) {
    html.push_str(
        "<section class=\"grid\"><div class=\"card\"><h2>Score</h2><div class=\"facts\">",
    );
    push_fact(html, "Status", flow_health_status_label(input.status));
    push_fact(html, "Grade", source_safety_grade_label(input.grade));
    if let Some(read_only) = input.read_only {
        push_fact(html, "Read only", if read_only { "true" } else { "false" });
    }
    if let Some(count) = input.table_count {
        push_fact(html, "Tables checked", &count.to_string());
    }
    if let Some(count) = input.unsafe_table_count {
        push_fact(html, "Unsafe tables", &count.to_string());
    }
    html.push_str("</div></div><div class=\"card\"><h2>Evidence</h2>");
    if let Some(slot) = input.slot {
        html.push_str("<p class=\"slot\">");
        push_html_escaped(html, &source_slot_summary_line(slot));
        html.push_str("</p>");
    } else {
        html.push_str("<p>No source slot evidence was attached.</p>");
    }
    push_source_safety_subscription_conflicts(html, input.subscription_conflicts);
    html.push_str("</div></section>");
}

fn push_findings(html: &mut String, factors: &[SourceSafetyFactor]) {
    html.push_str("<section class=\"card\"><h2>Findings</h2>");
    if factors.is_empty() {
        html.push_str("<p>No findings. Source safety passed with the evidence above.</p>");
    }
    for factor in factors {
        html.push_str("<article class=\"finding ");
        html.push_str(severity_class(factor.severity));
        html.push_str("\"><h3><span class=\"pill ");
        html.push_str(severity_class(factor.severity));
        html.push_str("\">");
        push_html_escaped(html, flow_alert_severity_label(factor.severity));
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

fn push_recommended_actions(html: &mut String, actions: &[String]) {
    html.push_str("<section class=\"card\"><h2>Recommended Actions</h2>");
    if actions.is_empty() {
        html.push_str("<p>No action required by this report.</p>");
    } else {
        html.push_str("<ol>");
        for action in actions {
            html.push_str("<li>");
            push_html_escaped(html, action);
            html.push_str("</li>");
        }
        html.push_str("</ol>");
    }
    html.push_str("</section>");
}

fn push_init_recommendation(
    html: &mut String,
    recommendation: Option<&SourceSafetyInitRecommendation>,
    written_path: Option<&str>,
) {
    if recommendation.is_none() && written_path.is_none() {
        return;
    }
    html.push_str("<section class=\"card\"><h2>Recovery Path</h2>");
    if let Some(recommendation) = recommendation {
        push_fact(html, "Init output", &recommendation.output);
        push_fact(html, "Primary key", &recommendation.primary_key);
        push_fact(
            html,
            "Evaluation ready",
            if recommendation.evaluation_ready {
                "true"
            } else {
                "false"
            },
        );
        html.push_str("<pre><code>");
        push_html_escaped(html, &recommendation.command);
        html.push_str("</code></pre><p>");
        push_html_escaped(html, &recommendation.note);
        html.push_str("</p>");
    }
    if let Some(path) = written_path {
        push_fact(html, "Init config written", path);
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

fn status_class(status: FlowHealthStatus) -> &'static str {
    match status {
        FlowHealthStatus::Healthy => "healthy",
        FlowHealthStatus::Degraded => "degraded",
        FlowHealthStatus::Blocked => "blocked",
    }
}

fn severity_class(severity: FlowAlertSeverity) -> &'static str {
    match severity {
        FlowAlertSeverity::Warning => "warning",
        FlowAlertSeverity::Critical => "critical",
    }
}
