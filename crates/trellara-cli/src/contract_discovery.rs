use std::collections::HashSet;

use crate::{contract_types::*, TrellaraConfig};

impl PreflightSummary {
    pub(crate) fn from_tables(tables: Vec<trellara_pg_capture::TablePreflight>) -> Self {
        let issue_count = tables.iter().map(|table| table.issues.len()).sum();
        Self {
            passed: issue_count == 0,
            issue_count,
            tables,
        }
    }
}

impl SchemaDiscoverySummary {
    pub(crate) fn from_preflight(
        config: &TrellaraConfig,
        tables: Vec<trellara_pg_capture::TablePreflight>,
    ) -> Self {
        let configured_tables = config
            .dataset
            .tables
            .iter()
            .map(|table| (table.schema.as_str(), table.name.as_str()))
            .collect::<HashSet<_>>();
        let tables = tables
            .into_iter()
            .map(|table| {
                let configured =
                    configured_tables.contains(&(table.schema.as_str(), table.name.as_str()));
                SchemaDiscoveryTable::from_preflight(table, configured)
            })
            .collect::<Vec<_>>();

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            table_count: tables.len(),
            tables,
        }
    }
}

impl SchemaDiscoveryTable {
    fn from_preflight(table: trellara_pg_capture::TablePreflight, configured: bool) -> Self {
        let suggested_primary_key = table.primary_key_columns.first().cloned();
        let config_snippet =
            schema_discovery_config_snippet(&table, suggested_primary_key.as_deref());
        Self {
            relation: table.qualified_name(),
            configured,
            exists: table.exists,
            replica_identity: table
                .replica_identity
                .map(|identity| format!("{identity:?}").to_ascii_lowercase()),
            update_delete_safe: table.update_delete_safe,
            primary_key_columns: table.primary_key_columns,
            column_count: table.columns.len(),
            schema_fingerprint: table.schema_fingerprint,
            issues: table.issues,
            contract_notes: table.contract_notes,
            suggested_primary_key,
            config_snippet,
        }
    }
}

fn schema_discovery_config_snippet(
    table: &trellara_pg_capture::TablePreflight,
    suggested_primary_key: Option<&str>,
) -> String {
    let mut lines = vec![
        format!("- schema: {}", table.schema),
        format!("  name: {}", table.name),
    ];
    if let Some(fingerprint) = table.schema_fingerprint {
        lines.extend([
            "  contract:".to_string(),
            format!("    source_schema_fingerprint: {fingerprint}"),
        ]);
    }
    if let Some(primary_key) = suggested_primary_key {
        lines.extend([
            "  verify:".to_string(),
            format!("    primary_key: {primary_key}"),
        ]);
    }
    lines.join("\n")
}
