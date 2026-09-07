use trellara_verify::{inspect_postgres_relation, PostgresRelationInspectionConfig};

use crate::contract_target_apply::apply_target_contracts;
use crate::{Result, TrellaraConfig};

pub(crate) fn apply_schema_contracts(
    config: &TrellaraConfig,
    mut tables: Vec<trellara_pg_capture::TablePreflight>,
) -> Vec<trellara_pg_capture::TablePreflight> {
    for table in &mut tables {
        let Some(configured) =
            config.dataset.tables.iter().find(|configured| {
                configured.schema == table.schema && configured.name == table.name
            })
        else {
            continue;
        };
        let Some(contract) = &configured.contract else {
            continue;
        };
        if let Some(expected) = contract.source_schema_fingerprint {
            match table.schema_fingerprint {
                Some(actual) if actual == expected => {}
                Some(actual) => table.issues.push(format!(
                    "source schema fingerprint mismatch: expected {expected}, got {actual}"
                )),
                None => table.issues.push(format!(
                    "source schema fingerprint mismatch: expected {expected}, got no schema"
                )),
            }
        }
    }
    tables
}

pub(crate) async fn apply_preflight_contracts(
    config: &TrellaraConfig,
    tables: Vec<trellara_pg_capture::TablePreflight>,
) -> Result<Vec<trellara_pg_capture::TablePreflight>> {
    let tables = apply_schema_contracts(config, tables);
    let Some(target) = &config.target else {
        return Ok(tables);
    };

    let mut inspections = Vec::new();
    for table in &config.dataset.tables {
        let relation = table.relation_id();
        inspections.push(
            inspect_postgres_relation(PostgresRelationInspectionConfig {
                database_url: target.database_url.expose().to_string(),
                relation,
            })
            .await?,
        );
    }

    Ok(apply_target_contracts(config, tables, inspections))
}
