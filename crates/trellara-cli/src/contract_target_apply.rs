use std::collections::{HashMap, HashSet};

use trellara_verify::PostgresRelationInspection;

use crate::contract_type_compat::is_safe_postgres_type_widening;
use crate::TrellaraConfig;

pub(crate) fn apply_target_contracts(
    config: &TrellaraConfig,
    mut tables: Vec<trellara_pg_capture::TablePreflight>,
    inspections: Vec<PostgresRelationInspection>,
) -> Vec<trellara_pg_capture::TablePreflight> {
    let inspections = inspections
        .into_iter()
        .map(|inspection| (inspection.relation.display_name(), inspection))
        .collect::<HashMap<_, _>>();

    for table in &mut tables {
        let Some(configured) =
            config.dataset.tables.iter().find(|configured| {
                configured.schema == table.schema && configured.name == table.name
            })
        else {
            continue;
        };
        let relation = configured.relation_id();
        let target_owned = configured
            .contract
            .as_ref()
            .map(|contract| {
                contract
                    .target_owned_columns
                    .iter()
                    .cloned()
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        let Some(inspection) = inspections.get(&relation.display_name()) else {
            continue;
        };

        if !inspection.exists {
            table.issues.push("target table does not exist".to_string());
            continue;
        }

        let target_columns = inspection
            .columns
            .iter()
            .map(|column| (column.name.clone(), column))
            .collect::<HashMap<_, _>>();

        apply_source_column_contracts(table, &target_columns, &target_owned);
        apply_target_owned_column_contracts(table, &target_columns, &target_owned);
        apply_target_only_column_contracts(table, inspection, &target_owned);
    }

    tables
}

fn apply_source_column_contracts(
    table: &mut trellara_pg_capture::TablePreflight,
    target_columns: &HashMap<String, &trellara_verify::PostgresColumnInspection>,
    target_owned: &HashSet<String>,
) {
    for column in &table.columns {
        if target_owned.contains(&column.name) {
            continue;
        }
        match target_columns.get(&column.name) {
            Some(target_column) if target_column.type_name == column.type_name => {}
            Some(target_column)
                if is_safe_postgres_type_widening(&column.type_name, &target_column.type_name) =>
            {
                table.contract_notes.push(format!(
                    "compatible target type widening for {}: source {}, target {}",
                    column.name, column.type_name, target_column.type_name
                ));
            }
            Some(target_column) => table.issues.push(format!(
                "target column {} type mismatch: source {}, target {}",
                column.name, column.type_name, target_column.type_name
            )),
            None => table.issues.push(format!(
                "target table is missing required column {}",
                column.name
            )),
        }
    }
}

fn apply_target_owned_column_contracts(
    table: &mut trellara_pg_capture::TablePreflight,
    target_columns: &HashMap<String, &trellara_verify::PostgresColumnInspection>,
    target_owned: &HashSet<String>,
) {
    for column in target_owned {
        if !target_columns.contains_key(column) {
            table.issues.push(format!(
                "target-owned column {column} does not exist on target"
            ));
        }
    }
}

fn apply_target_only_column_contracts(
    table: &mut trellara_pg_capture::TablePreflight,
    inspection: &PostgresRelationInspection,
    target_owned: &HashSet<String>,
) {
    for target_column in &inspection.columns {
        if target_owned.contains(&target_column.name) {
            continue;
        }
        if table
            .columns
            .iter()
            .any(|column| column.name == target_column.name)
        {
            continue;
        }
        if !target_column.nullable {
            table.issues.push(format!(
                "target-only column {} is non-null and not target-owned",
                target_column.name
            ));
        } else {
            table.contract_notes.push(format!(
                "compatible nullable target-only column {}",
                target_column.name
            ));
        }
    }
}
