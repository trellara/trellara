use crate::{push_html_escaped, ChaosRunSummary, ChaosScenarioStatus};

pub(crate) fn push_curated_failure_matrix(html: &mut String, summary: &ChaosRunSummary) {
    html.push_str("<section><h2>Curated Failure Matrix</h2><table><thead><tr><th>Scenario</th><th>Status</th><th>Boundary</th><th>Invariant</th><th>Safety property</th><th>Proof</th><th>Recovery</th></tr></thead><tbody>");
    for scenario in &summary.scenarios {
        html.push_str("<tr><td>");
        push_html_escaped(html, &scenario.name);
        html.push_str("</td><td>");
        match scenario.status {
            ChaosScenarioStatus::CoveredByTests => {
                html.push_str("<span class=\"status ok\">covered</span>")
            }
        }
        html.push_str("</td><td><code>");
        push_html_escaped(html, &scenario.boundary_mode);
        html.push_str("</code></td><td>");
        push_html_escaped(html, &scenario.invariant);
        html.push_str("</td><td>");
        push_html_escaped(html, &scenario.expected_safety_property);
        html.push_str("</td><td><code>");
        push_html_escaped(html, &scenario.proof_command);
        html.push_str("</code></td><td>");
        if let Some(command) = &scenario.recovery_command {
            html.push_str("<code>");
            push_html_escaped(html, command);
            html.push_str("</code>");
        } else {
            html.push_str("automatic");
        }
        html.push_str("</td></tr>");
    }
    html.push_str("</tbody></table></section>");
}
