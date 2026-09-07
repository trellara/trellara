use std::path::Path;

use crate::fleet_types::*;
use crate::{
    fleet_convergence_gates, fleet_lake_fanin_readiness, fleet_recovery_drills,
    fleet_transaction_boundary, CliError, DatasetMode, FleetReportArgs, FlowStreamSummary, Result,
    StreamConfig, TrellaraConfig,
};

impl FleetReportSummary {
    pub(crate) fn from_args(args: &FleetReportArgs) -> Result<Self> {
        if args.config.is_empty() {
            return Err(CliError::InvalidConfig(
                "fleet report requires at least one --config".to_string(),
            ));
        }

        let mut flows = Vec::new();
        for path in &args.config {
            let config = TrellaraConfig::from_path(path)?;
            config.validate()?;
            flows.push(FleetFlowSummary::from_config(&config, path)?);
        }

        Ok(Self::from_flows(flows))
    }

    pub(crate) fn from_configs(configs: &[(&TrellaraConfig, &Path)]) -> Result<Self> {
        if configs.is_empty() {
            return Err(CliError::InvalidConfig(
                "fleet report requires at least one config".to_string(),
            ));
        }

        let flows = configs
            .iter()
            .map(|(config, path)| FleetFlowSummary::from_config(config, path))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::from_flows(flows))
    }
}

impl FleetFlowSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Result<Self> {
        let mode = config.status_mode();
        let stream = FlowStreamSummary::from_config(&config.stream);
        let target_configured = config.target.is_some();
        let convergence_gates = fleet_convergence_gates(config, path);
        let recovery_drills = fleet_recovery_drills(config, path);
        let lake_fanin = fleet_lake_fanin_readiness(config, path);
        let proof_commands = convergence_gates
            .iter()
            .map(|gate| gate.proof_command.clone())
            .chain(
                recovery_drills
                    .iter()
                    .flat_map(|drill| drill.commands.iter().cloned()),
            )
            .chain(lake_fanin.proof_commands.iter().cloned())
            .collect();

        let mut risks = Vec::new();
        if !target_configured {
            risks.push("target.database_url missing; convergence proof cannot run".to_string());
        }
        if config.dataset.mode == DatasetMode::PartitionedScaleMode {
            risks.push(
                "partitioned scale requires manifest and partition-watermark review before global visibility"
                    .to_string(),
            );
        }
        if matches!(config.stream, StreamConfig::Kafka { .. }) {
            risks.push(
                "Kafka-backed flow requires external broker operations for the first pilot"
                    .to_string(),
            );
        }

        Ok(Self {
            flow_id: format!("{}:{}", config.source.id, config.dataset.id),
            config: path.display().to_string(),
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode,
            stream_kind: stream.kind,
            target_configured,
            table_count: config.dataset.tables.len(),
            tables: config
                .dataset
                .tables
                .iter()
                .map(|table| table.relation_id().display_name())
                .collect(),
            topics: config.replay_redelivery_topics()?,
            transaction_boundary: fleet_transaction_boundary(config),
            lake_fanin,
            convergence_gates,
            recovery_drills,
            proof_commands,
            risks,
        })
    }
}
