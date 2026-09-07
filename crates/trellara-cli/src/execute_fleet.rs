use crate::*;

pub(crate) fn execute_fleet_command(command: FleetCommand) -> Result<String> {
    match command {
        FleetCommand::Init(args) => Ok(serde_json::to_string_pretty(&init_fleet_configs(&args)?)?),
        FleetCommand::Report(args) => {
            render_fleet_report_summary(&FleetReportSummary::from_args(&args)?, args.format)
        }
        FleetCommand::Scorecard(args) => {
            render_fleet_scorecard_summary(&FleetScorecardSummary::from_args(&args)?, args.format)
        }
        FleetCommand::ControlPlane(args) => render_fleet_control_plane_summary(
            &FleetControlPlaneSummary::from_args(&args)?,
            args.format,
        ),
        FleetCommand::IdentityAudit(args) => render_fleet_identity_audit_summary(
            &FleetIdentityAuditSummary::from_args(&args)?,
            args.format,
        ),
        FleetCommand::EvidencePlan(args) => render_fleet_evidence_plan_summary(
            &FleetEvidencePlanSummary::from_args(&args)?,
            args.format,
        ),
    }
}
