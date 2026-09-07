use trellara_protocol::ReplicaIdentity;

use crate::TableColumn;

pub(crate) fn replica_identity_contract_notes(
    replica_identity: Option<ReplicaIdentity>,
    primary_key_columns: &[String],
    columns: &[TableColumn],
) -> Vec<String> {
    let mut notes = Vec::new();
    match replica_identity {
        Some(ReplicaIdentity::Default) if !primary_key_columns.is_empty() => {
            notes.push(format!(
                "replica identity DEFAULT is accepted because primary key ({}) can identify UPDATE/DELETE rows; REPLICA IDENTITY FULL is not required for this table",
                primary_key_columns.join(", ")
            ));
            append_toast_contract_note(&mut notes, columns);
        }
        Some(ReplicaIdentity::Index) => notes.push(
            "replica identity uses a configured identity index; UPDATE/DELETE rows are key-addressable without FULL"
                .to_string(),
        ),
        Some(ReplicaIdentity::Full) => notes.push(
            "replica identity FULL is safe but can increase WAL volume; ordinary primary-key tables can usually use DEFAULT"
                .to_string(),
        ),
        _ => {}
    }
    notes
}

fn append_toast_contract_note(notes: &mut Vec<String>, columns: &[TableColumn]) {
    let toastable = columns
        .iter()
        .filter(|column| !column.is_key && is_potentially_toasted_type(&column.type_name))
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    if !toastable.is_empty() {
        notes.push(format!(
            "replica identity DEFAULT may omit unchanged TOAST values for {}; downstream apply must treat absent non-key columns as unchanged",
            toastable.join(", ")
        ));
    }
}

fn is_potentially_toasted_type(type_name: &str) -> bool {
    let normalized = type_name.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "text" | "bytea" | "json" | "jsonb" | "xml"
    ) || normalized.contains("character varying")
        || normalized.contains("varchar")
        || normalized.ends_with("[]")
}
