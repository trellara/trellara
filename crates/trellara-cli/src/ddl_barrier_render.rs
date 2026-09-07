use serde::Serialize;
use trellara_checkpoint::DdlBarrierSummary;

use crate::ddl_barrier_next_actions::{next_actions, push_next_actions};
use crate::ddl_barrier_render_sections::{
    push_barrier_header, push_barrier_sink_summary, push_release_blocker_details,
    push_release_gates, push_release_summary, push_sink_evidence,
};
use crate::{QuickstartOutputFormat, Result};

#[derive(Serialize)]
struct RenderedDdlBarrierSummary<'a> {
    #[serde(flatten)]
    summary: &'a DdlBarrierSummary,
    next_actions: Vec<String>,
}

pub(crate) fn render_ddl_barrier_summary(
    summary: &DdlBarrierSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => {
            Ok(serde_json::to_string_pretty(&RenderedDdlBarrierSummary {
                summary,
                next_actions: next_actions(summary),
            })?)
        }
        QuickstartOutputFormat::Text => Ok(render_ddl_barrier_text(summary)),
    }
}

pub(crate) fn render_ddl_barrier_text(summary: &DdlBarrierSummary) -> String {
    let mut output = String::new();
    push_barrier_header(&mut output, summary);
    push_barrier_sink_summary(&mut output, summary);
    push_release_summary(&mut output, summary);
    push_release_blocker_details(&mut output, summary);
    push_release_gates(&mut output, summary);
    push_sink_evidence(&mut output, summary);
    push_next_actions(&mut output, summary);
    output
}
