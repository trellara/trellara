use crate::{
    render_fleet_control_plane_text, render_fleet_report_text, render_fleet_scorecard_text,
    FleetControlPlaneSummary, FleetEvidencePlanSummary, FleetIdentityAuditSummary,
    FleetReportSummary, FleetScorecardSummary, QuickstartOutputFormat, Result,
};

pub(crate) use crate::fleet_evidence_render::render_fleet_evidence_plan_text;
pub(crate) use crate::fleet_identity_render::render_fleet_identity_audit_text;

pub(crate) fn render_fleet_report_summary(
    summary: &FleetReportSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_fleet_report_text(summary)),
    }
}

pub(crate) fn render_fleet_scorecard_summary(
    summary: &FleetScorecardSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_fleet_scorecard_text(summary)),
    }
}

pub(crate) fn render_fleet_control_plane_summary(
    summary: &FleetControlPlaneSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_fleet_control_plane_text(summary)),
    }
}

pub(crate) fn render_fleet_identity_audit_summary(
    summary: &FleetIdentityAuditSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_fleet_identity_audit_text(summary)),
    }
}

pub(crate) fn render_fleet_evidence_plan_summary(
    summary: &FleetEvidencePlanSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_fleet_evidence_plan_text(summary)),
    }
}
