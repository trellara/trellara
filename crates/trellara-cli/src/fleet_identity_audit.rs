use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::{
    CliError, FleetIdentityAuditArgs, FleetIdentityAuditSummary, FleetIdentityDuplicateGroup,
    FleetIdentityFlow, FleetIdentityStatus, FlowStreamSummary, Result, TrellaraConfig,
};

impl FleetIdentityAuditSummary {
    pub(crate) fn from_args(args: &FleetIdentityAuditArgs) -> Result<Self> {
        if args.config.is_empty() {
            return Err(CliError::InvalidConfig(
                "fleet identity-audit requires at least one --config".to_string(),
            ));
        }

        let mut config_refs = Vec::new();
        for path in &args.config {
            let config = TrellaraConfig::from_path(path)?;
            config.validate()?;
            config_refs.push((config, path.clone()));
        }

        Self::from_configs(&config_refs)
    }

    pub(crate) fn from_configs(configs: &[(TrellaraConfig, PathBuf)]) -> Result<Self> {
        let flow_ids = configs
            .iter()
            .map(|(config, _)| format!("{}:{}", config.source.id, config.dataset.id))
            .collect::<Vec<_>>();
        let mut flow_id_counts = BTreeMap::<String, usize>::new();
        for flow_id in &flow_ids {
            *flow_id_counts.entry(flow_id.clone()).or_default() += 1;
        }
        let duplicate_ids = flow_id_counts
            .iter()
            .filter(|(_, count)| **count > 1)
            .map(|(flow_id, _)| flow_id.clone())
            .collect::<BTreeSet<_>>();

        let flows = configs
            .iter()
            .zip(flow_ids.iter())
            .map(|((config, path), flow_id)| FleetIdentityFlow {
                flow_id: flow_id.clone(),
                config: path.display().to_string(),
                source_id: config.source.id.clone(),
                dataset_id: config.dataset.id.clone(),
                mode: config.status_mode(),
                stream_kind: FlowStreamSummary::from_config(&config.stream).kind,
                status: if duplicate_ids.contains(flow_id) {
                    FleetIdentityStatus::Duplicate
                } else {
                    FleetIdentityStatus::Unique
                },
            })
            .collect::<Vec<_>>();
        let duplicate_groups = duplicate_ids
            .iter()
            .map(|flow_id| {
                let configs = flows
                    .iter()
                    .filter(|flow| &flow.flow_id == flow_id)
                    .map(|flow| flow.config.clone())
                    .collect::<Vec<_>>();
                FleetIdentityDuplicateGroup {
                    flow_id: flow_id.clone(),
                    remediation: format!(
                        "assign unique source.id and dataset.id values before creating hosted control-plane state for {}",
                        configs.join(", ")
                    ),
                    configs,
                }
            })
            .collect::<Vec<_>>();
        let flow_count = flows.len();
        let duplicate_flow_count = flows
            .iter()
            .filter(|flow| flow.status == FleetIdentityStatus::Duplicate)
            .count();
        let unique_flow_count = flow_count.saturating_sub(duplicate_flow_count);
        let source_count = configs
            .iter()
            .map(|(config, _)| config.source.id.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let dataset_count = configs
            .iter()
            .map(|(config, _)| config.dataset.id.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        let verdict = if duplicate_flow_count > 0 {
            "blocked_by_identity_collision"
        } else if flow_count < 2 {
            "single_flow_ready"
        } else {
            "identity_ready_for_control_plane"
        }
        .to_string();
        let remediation = if duplicate_groups.is_empty() {
            vec![
                "keep source.id and dataset.id stable across every generated package".to_string(),
                "run fleet report, fleet scorecard, and fleet control-plane after adding or renaming flows".to_string(),
            ]
        } else {
            duplicate_groups
                .iter()
                .map(|group| group.remediation.clone())
                .chain(std::iter::once(
                    "rerun trellara fleet identity-audit before sharing the evidence registry"
                        .to_string(),
                ))
                .collect()
        };
        let config_flags = configs
            .iter()
            .map(|(_, path)| format!("--config {}", path.display()))
            .collect::<Vec<_>>()
            .join(" ");
        let proof_commands = vec![
            format!("trellara fleet identity-audit {config_flags} --format text"),
            format!("trellara fleet report {config_flags} --format text"),
            format!("trellara fleet control-plane {config_flags} --format text"),
        ];
        let next_commands = if duplicate_groups.is_empty() {
            vec![
                "generate pilot packages for identity-ready flows".to_string(),
                "run trellara evidence-registry for each package before enterprise review"
                    .to_string(),
            ]
        } else {
            vec![
                "rename colliding source.id or dataset.id values in the listed configs".to_string(),
                format!("trellara fleet identity-audit {config_flags} --format text"),
            ]
        };

        Ok(Self {
            verdict,
            flow_count,
            unique_flow_count,
            duplicate_flow_count,
            source_count,
            dataset_count,
            flows,
            duplicate_groups,
            remediation,
            proof_commands,
            next_commands,
        })
    }
}
