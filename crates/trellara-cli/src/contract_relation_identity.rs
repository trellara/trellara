use trellara_protocol::ReplicaIdentity;

use crate::{ContractCheck, ContractSeverity};

pub(crate) fn cdc_identity_contract_checks(
    table: &trellara_pg_capture::TablePreflight,
) -> Vec<ContractCheck> {
    let relation = table.qualified_name();
    let name = format!("cdc_identity:{relation}");
    match table.replica_identity {
        Some(ReplicaIdentity::Default) if !table.primary_key_columns.is_empty() => {
            vec![ContractCheck::passed_with_severity(
                name,
                ContractSeverity::Warning,
                format!(
                    "{relation} is safe with primary key apply on ({}) under replica identity DEFAULT; REPLICA IDENTITY FULL is not required, but apply must use key predicates",
                    table.primary_key_columns.join(", ")
                ),
            )]
        }
        Some(ReplicaIdentity::Default) => vec![unsafe_identity_check(
            name,
            format!(
                "{relation} is unsafe until identity configured; replica identity DEFAULT has no primary key for UPDATE/DELETE rows"
            ),
        )],
        Some(ReplicaIdentity::Index) => vec![ContractCheck::passed(
            name,
            format!(
                "{relation} is safe with identity index apply; UPDATE/DELETE rows are key-addressable without requiring FULL"
            ),
        )],
        Some(ReplicaIdentity::Full) => vec![ContractCheck::passed(
            name,
            format!(
                "{relation} has FULL row identity; this is safe for UPDATE/DELETE capture and is the requires FULL posture for tables without a stable key"
            ),
        )],
        Some(ReplicaIdentity::Nothing) | Some(ReplicaIdentity::Unspecified) | None => {
            vec![unsafe_identity_check(
                name,
                format!(
                    "{relation} is unsafe until identity configured; UPDATE/DELETE rows are not key-addressable"
                ),
            )]
        }
    }
}

fn unsafe_identity_check(name: String, message: String) -> ContractCheck {
    ContractCheck::failed(
        name,
        ContractSeverity::Error,
        message,
        "add a primary key, configure a replica identity index, or set REPLICA IDENTITY FULL before enabling CDC",
    )
}
