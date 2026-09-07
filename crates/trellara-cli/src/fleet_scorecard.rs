use std::path::PathBuf;

use crate::{
    blocked_flow_count, configuration_ready_gate_count, fleet_score, fleet_scorecard_blockers,
    fleet_scorecard_flow, fleet_scorecard_next_commands, fleet_scorecard_review_sequence,
    fleet_scorecard_verdict, ready_flow_count, CliError, FleetReportSummary, FleetScorecardArgs,
    FleetScorecardSummary, Result, TrellaraConfig,
};

impl FleetScorecardSummary {
    pub(crate) fn from_args(args: &FleetScorecardArgs) -> Result<Self> {
        if args.config.is_empty() {
            return Err(CliError::InvalidConfig(
                "fleet scorecard requires at least one --config".to_string(),
            ));
        }

        let mut configs = Vec::new();
        for path in &args.config {
            let config = TrellaraConfig::from_path(path)?;
            config.validate()?;
            configs.push((config, path.clone()));
        }

        Self::from_configs(&configs)
    }

    pub(crate) fn from_configs(configs: &[(TrellaraConfig, PathBuf)]) -> Result<Self> {
        let config_refs = configs
            .iter()
            .map(|(config, path)| (config, path.as_path()))
            .collect::<Vec<_>>();
        let report = FleetReportSummary::from_configs(&config_refs)?;
        let flows = configs
            .iter()
            .zip(report.flows.iter())
            .map(|((config, path), flow_report)| {
                fleet_scorecard_flow(config, path.as_path(), flow_report)
            })
            .collect::<Vec<_>>();
        let blockers = fleet_scorecard_blockers(&flows);

        let flow_count = flows.len();
        let ready_flow_count = ready_flow_count(&flows);
        let blocked_flow_count = blocked_flow_count(&flows);
        let review_required_flow_count =
            flow_count.saturating_sub(ready_flow_count + blocked_flow_count);
        let configuration_ready_gate_count = configuration_ready_gate_count(&flows);
        let needs_live_evidence_gate_count = flows
            .iter()
            .map(|flow| flow.needs_live_evidence_gate_count)
            .sum();
        let blocked_gate_count = flows.iter().map(|flow| flow.blocked_gate_count).sum();
        let score = fleet_score(&flows);
        let verdict = fleet_scorecard_verdict(blocked_flow_count, &report);
        let next_commands = fleet_scorecard_next_commands(configs, &report);
        let review_sequence = fleet_scorecard_review_sequence();

        Ok(Self {
            flow_count,
            verdict,
            score,
            topology_verdict: report.topology_verdict,
            ready_flow_count,
            review_required_flow_count,
            blocked_flow_count,
            local_stream_count: report.local_stream_count,
            kafka_stream_count: report.kafka_stream_count,
            target_configured_count: report.target_configured_count,
            configuration_ready_gate_count,
            needs_live_evidence_gate_count,
            blocked_gate_count,
            convergence_gate_count: report.convergence_gate_count,
            blocked_convergence_gate_count: report.blocked_convergence_gate_count,
            recovery_drill_count: report.recovery_drill_count,
            lake_ready_flow_count: report.lake_ready_flow_count,
            lake_publishable_with_gaps_flow_count: report.lake_publishable_with_gaps_flow_count,
            lake_blocked_flow_count: report.lake_blocked_flow_count,
            lake_fanin_verdict: report.lake_fanin_verdict,
            flows,
            blockers,
            warnings: report.warnings,
            review_sequence,
            next_commands,
        })
    }
}
