use trellara_protocol::ReplicaIdentity;

use crate::{ContractCheck, ContractSeverity};

pub(crate) fn toast_contract_checks(
    table: &trellara_pg_capture::TablePreflight,
) -> Vec<ContractCheck> {
    if table.replica_identity != Some(ReplicaIdentity::Default)
        || table.primary_key_columns.is_empty()
    {
        return Vec::new();
    }

    let toastable_columns = table
        .columns
        .iter()
        .filter(|column| !column.is_key && is_potentially_toasted_cli_type(&column.type_name))
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    if toastable_columns.is_empty() {
        return Vec::new();
    }

    let relation = table.qualified_name();
    vec![ContractCheck::passed_with_severity(
        format!("toast_patching:{relation}"),
        ContractSeverity::Warning,
        format!(
            "{relation} has TOAST patching required for {}; apply must preserve absent non-key columns as unchanged across CDC transaction boundaries",
            toastable_columns.join(", ")
        ),
    )]
}

fn is_potentially_toasted_cli_type(type_name: &str) -> bool {
    let normalized = type_name.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "text" | "bytea" | "json" | "jsonb" | "xml"
    ) || normalized.contains("character varying")
        || normalized.contains("varchar")
        || normalized.ends_with("[]")
}
