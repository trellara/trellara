use std::path::Path;

use serde::Serialize;

use crate::{DatasetMode, FlowStreamSummary, StreamConfig, TrellaraConfig};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowCreateSummary {
    pub(crate) accepted: bool,
    pub(crate) config: String,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) table_count: usize,
    pub(crate) tables: Vec<String>,
    pub(crate) stream: FlowStreamSummary,
    pub(crate) next_commands: Vec<String>,
}

impl FlowCreateSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let mut next_commands = vec![
            format!("trellara preflight --config {}", path.display()),
            format!("trellara contract-test --config {}", path.display()),
            format!("trellara schema-discover --config {}", path.display()),
            format!("trellara lake plan --config {}", path.display()),
            format!("trellara bootstrap --config {}", path.display()),
            format!("trellara relay --config {}", path.display()),
            "trellara chaos run".to_string(),
        ];
        if matches!(config.stream, StreamConfig::Local { .. }) {
            next_commands.push(format!(
                "trellara stream inspect-local --config {}",
                path.display()
            ));
        }
        if config.dataset.mode == DatasetMode::PartitionedScaleMode {
            next_commands.push(format!(
                "trellara partition-local --config {}",
                path.display()
            ));
            if config.target.is_some() {
                next_commands.push(format!(
                    "trellara partition-watermarks --config {}",
                    path.display()
                ));
                next_commands.push(format!(
                    "trellara partition-rebalance-plan --config {} --partition-event-count <partition>=<count> --format text",
                    path.display()
                ));
            }
        }
        if config.target.is_some() {
            next_commands.extend([
                format!("trellara apply-schema --config {}", path.display()),
                format!("trellara apply --config {}", path.display()),
                format!("trellara verify --config {}", path.display()),
                format!(
                    "trellara status --config {} --view report --format text",
                    path.display()
                ),
                format!(
                    "trellara status --config {} --view alerts --format text",
                    path.display()
                ),
                format!(
                    "trellara status --config {} --view dashboard --format text",
                    path.display()
                ),
                format!("trellara status --config {} --view metrics", path.display()),
                format!(
                    "trellara status --config {} --view diagnostics --format text",
                    path.display()
                ),
                format!("trellara check --config {}", path.display()),
                format!("trellara repair-plan --config {}", path.display()),
                format!("trellara quarantine list --config {}", path.display()),
                format!(
                    "trellara quarantine replay-ready --config {} --transaction-id <tx> --commit-lsn <lsn>",
                    path.display()
                ),
                format!("trellara reseed --config {}", path.display()),
            ]);
        }

        Self {
            accepted: true,
            config: path.display().to_string(),
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            table_count: config.dataset.tables.len(),
            tables: config
                .dataset
                .tables
                .iter()
                .map(|table| table.relation_id().display_name())
                .collect(),
            stream: FlowStreamSummary::from_config(&config.stream),
            next_commands,
        }
    }
}
