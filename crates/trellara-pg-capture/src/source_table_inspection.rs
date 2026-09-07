use trellara_protocol::ReplicaIdentity;
use xxhash_rust::xxh3::xxh3_64;

use crate::source_replica_identity_contract::replica_identity_contract_notes;
use crate::{decode_replica_identity, CaptureError, Result, TableColumn, TablePreflight};

pub(crate) fn table_preflight_from_row(row: tokio_postgres::Row) -> TablePreflight {
    let schema = row.get::<_, String>("schema_name");
    let name = row.get::<_, String>("table_name");
    let replica_identity_code = row.get::<_, Option<String>>("relreplident");
    let primary_key_columns = row.get::<_, Vec<String>>("primary_key_columns");
    let columns = table_columns_from_row(&row, &primary_key_columns);
    let schema_fingerprint = if columns.is_empty() {
        None
    } else {
        Some(schema_fingerprint(&columns))
    };
    let replica_identity = replica_identity_code.map(decode_replica_identity);
    let exists = replica_identity.is_some();
    let update_delete_safe = match replica_identity {
        Some(ReplicaIdentity::Full | ReplicaIdentity::Index) => true,
        Some(ReplicaIdentity::Default) => !primary_key_columns.is_empty(),
        _ => false,
    };
    let mut issues = Vec::new();
    if !exists {
        issues.push("table does not exist".to_string());
    } else if !update_delete_safe {
        issues.push(
            "UPDATE/DELETE capture is unsafe without replica identity FULL, identity index, or primary key"
                .to_string(),
        );
    }

    let contract_notes =
        replica_identity_contract_notes(replica_identity, &primary_key_columns, &columns);

    TablePreflight {
        schema,
        name,
        exists,
        replica_identity,
        primary_key_columns,
        columns,
        schema_fingerprint,
        update_delete_safe,
        issues,
        contract_notes,
    }
}

pub(crate) fn schema_fingerprint(columns: &[TableColumn]) -> u64 {
    let mut input = String::new();
    for column in columns {
        input.push_str(&column.ordinal_position.to_string());
        input.push('\x1f');
        input.push_str(&column.name);
        input.push('\x1f');
        input.push_str(&column.type_oid.to_string());
        input.push('\x1f');
        input.push_str(&column.type_name);
        input.push('\x1f');
        input.push_str(if column.nullable {
            "nullable"
        } else {
            "required"
        });
        input.push('\x1f');
        input.push_str(if column.is_key { "key" } else { "value" });
        input.push('\x1e');
    }
    xxh3_64(input.as_bytes())
}

pub(crate) fn fail_on_preflight_issues(preflight: &[TablePreflight]) -> Result<()> {
    let issues = preflight
        .iter()
        .flat_map(|table| {
            table
                .issues
                .iter()
                .map(|issue| format!("{}: {issue}", table.qualified_name()))
        })
        .collect::<Vec<_>>();
    if issues.is_empty() {
        Ok(())
    } else {
        Err(CaptureError::PreflightFailed { issues })
    }
}

fn table_columns_from_row(
    row: &tokio_postgres::Row,
    primary_key_columns: &[String],
) -> Vec<TableColumn> {
    let ordinals = row.get::<_, Vec<i32>>("column_ordinals");
    let names = row.get::<_, Vec<String>>("column_names");
    let type_oids = row.get::<_, Vec<u32>>("column_type_oids");
    let type_names = row.get::<_, Vec<String>>("column_type_names");
    let nullable = row.get::<_, Vec<bool>>("column_nullable");

    names
        .into_iter()
        .enumerate()
        .map(|(index, name)| TableColumn {
            ordinal_position: ordinals[index],
            type_oid: type_oids[index],
            type_name: type_names[index].clone(),
            nullable: nullable[index],
            is_key: primary_key_columns.contains(&name),
            name,
        })
        .collect()
}
