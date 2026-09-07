use crate::{
    push_chaos_report_overview, push_curated_failure_matrix, push_fleet_fanin_simulation_section,
    push_qualification_simulation_section, push_replay_simulation_section,
    push_snapshot_simulation_section, push_strict_chunk_simulation_section, ChaosRunSummary,
};

pub(crate) fn render_chaos_report_html(summary: &ChaosRunSummary) -> String {
    let mut html = String::new();
    push_chaos_report_overview(&mut html, summary);
    push_replay_simulation_section(&mut html, summary);
    push_snapshot_simulation_section(&mut html, summary);
    push_strict_chunk_simulation_section(&mut html, summary);
    push_fleet_fanin_simulation_section(&mut html, summary);
    push_qualification_simulation_section(&mut html, summary);
    push_curated_failure_matrix(&mut html, summary);
    html.push_str("</main></body></html>");
    html
}
