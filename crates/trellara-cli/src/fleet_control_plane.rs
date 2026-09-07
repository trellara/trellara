use crate::{
    fleet_control_plane_capabilities, fleet_control_plane_evidence_gaps, CliError,
    FleetControlPlaneArgs, FleetControlPlaneCapabilityStatus, FleetControlPlaneShape,
    FleetControlPlaneSummary, FleetReportSummary, FleetScorecardSummary, Result, TrellaraConfig,
};

impl FleetControlPlaneSummary {
    pub(crate) fn from_args(args: &FleetControlPlaneArgs) -> Result<Self> {
        if args.config.is_empty() {
            return Err(CliError::InvalidConfig(
                "fleet control-plane requires at least one --config".to_string(),
            ));
        }

        let mut configs = Vec::new();
        for path in &args.config {
            let config = TrellaraConfig::from_path(path)?;
            config.validate()?;
            configs.push((config, path.clone()));
        }

        let config_refs = configs
            .iter()
            .map(|(config, path)| (config, path.as_path()))
            .collect::<Vec<_>>();
        let report = FleetReportSummary::from_configs(&config_refs)?;
        let scorecard = FleetScorecardSummary::from_configs(&configs)?;

        Ok(Self::from_report_and_scorecard(&report, &scorecard))
    }

    pub(crate) fn from_report_and_scorecard(
        report: &FleetReportSummary,
        scorecard: &FleetScorecardSummary,
    ) -> Self {
        let identity_collisions = report
            .warnings
            .iter()
            .filter(|warning| warning.contains("control-plane identity would collide"))
            .cloned()
            .collect::<Vec<_>>();
        let evidence_gaps = fleet_control_plane_evidence_gaps(report, scorecard);
        let capabilities_to_build =
            fleet_control_plane_capabilities(report, scorecard, &evidence_gaps);
        let capability_pull_count = capabilities_to_build
            .iter()
            .filter(|capability| capability.status == FleetControlPlaneCapabilityStatus::PulledNow)
            .count();
        let verdict = if !identity_collisions.is_empty() {
            "blocked_by_identity_collision"
        } else if !evidence_gaps.is_empty() {
            "validate_with_design_partners"
        } else if capability_pull_count > 0 {
            "build_minimal_control_plane"
        } else {
            "defer_cloud_control_plane"
        }
        .to_string();
        let build_recommendation = match verdict.as_str() {
            "blocked_by_identity_collision" => {
                "do not build hosted control-plane workflows until flow identities are stable and collision-free"
            }
            "validate_with_design_partners" => {
                "keep using local artifacts; gather missing fleet proof before committing hosted control-plane scope"
            }
            "build_minimal_control_plane" => {
                "build only the pulled capabilities, starting with read-only fleet topology and evidence packaging"
            }
            _ => "defer hosted control plane; current evidence is better served by local proof artifacts",
        }
        .to_string();
        let defer_until_pulled = capabilities_to_build
            .iter()
            .filter(|capability| capability.status == FleetControlPlaneCapabilityStatus::Defer)
            .map(|capability| capability.code.clone())
            .collect();
        let mut proof_commands = report.proof_commands.clone();
        proof_commands.push("trellara fleet report --config <flow> --format text".to_string());
        proof_commands.push("trellara fleet scorecard --config <flow> --format text".to_string());
        proof_commands
            .push("trellara fleet identity-audit --config <flow> --format text".to_string());
        proof_commands.sort();
        proof_commands.dedup();
        let next_commands = vec![
            "run this report with every design-partner flow config".to_string(),
            "review feature-pull-list.md after the first fleet review".to_string(),
            "build only PulledNow capabilities after repeated partner evidence".to_string(),
            "keep Cloud out of the critical path until local proof artifacts are insufficient"
                .to_string(),
        ];

        Self {
            verdict,
            build_recommendation,
            fleet_shape: FleetControlPlaneShape {
                flow_count: report.flow_count,
                source_count: report.source_count,
                dataset_count: report.dataset_count,
                table_count: report.table_count,
                local_stream_count: report.local_stream_count,
                kafka_stream_count: report.kafka_stream_count,
                partitioned_flow_count: report.partitioned_flow_count,
                target_configured_count: report.target_configured_count,
                topology_verdict: report.topology_verdict.clone(),
                fleet_scorecard_verdict: scorecard.verdict.clone(),
            },
            capability_pull_count,
            capabilities_to_build,
            defer_until_pulled,
            evidence_gaps,
            identity_collisions,
            proof_commands,
            next_commands,
        }
    }
}
