use trellara_pg_capture::TablePreflight;

use crate::SourceSafetyFactor;

pub(crate) fn source_table_factors(table: &TablePreflight) -> Vec<SourceSafetyFactor> {
    if !table.exists {
        return vec![SourceSafetyFactor::critical(
            "table_missing",
            30,
            format!("source table {} does not exist", table.qualified_name()),
            "fix the table selector or create the table before enabling CDC",
        )];
    }

    if !table.update_delete_safe || !table.issues.is_empty() {
        return vec![SourceSafetyFactor::critical(
            "table_cdc_unsafe",
            25,
            format!(
                "source table {} has {} issue(s): {}",
                table.qualified_name(),
                table.issues.len(),
                table.issues.join("; ")
            ),
            "repair replica identity, primary key, or table compatibility before enabling CDC",
        )];
    }

    if !table.contract_notes.is_empty() {
        return vec![SourceSafetyFactor::warning(
            "table_contract_note",
            5,
            format!(
                "source table {} has CDC contract note(s): {}",
                table.qualified_name(),
                table.contract_notes.join("; ")
            ),
            "review the table contract notes and pin schema fingerprints before production rollout",
        )];
    }

    Vec::new()
}

pub(crate) fn source_table_is_unsafe(table: &TablePreflight) -> bool {
    !table.exists || !table.update_delete_safe || !table.issues.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellara_pg_capture::{TableColumn, TablePreflight};
    use trellara_protocol::ReplicaIdentity;

    #[test]
    fn missing_table_is_critical_source_safety_factor() {
        let table = TablePreflight {
            exists: false,
            ..table()
        };

        let factors = source_table_factors(&table);

        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].code, "table_missing");
        assert!(factors[0].evidence.contains("public.sales"));
        assert!(source_table_is_unsafe(&table));
    }

    #[test]
    fn unsafe_cdc_table_is_critical_source_safety_factor() {
        let table = TablePreflight {
            update_delete_safe: false,
            issues: vec!["table has no primary key".to_string()],
            ..table()
        };

        let factors = source_table_factors(&table);

        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].code, "table_cdc_unsafe");
        assert!(factors[0].evidence.contains("1 issue(s)"));
        assert!(factors[0].evidence.contains("table has no primary key"));
        assert!(source_table_is_unsafe(&table));
    }

    #[test]
    fn contract_notes_are_warning_source_safety_factors() {
        let table = TablePreflight {
            contract_notes: vec!["replica identity full increases payload size".to_string()],
            ..table()
        };

        let factors = source_table_factors(&table);

        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].code, "table_contract_note");
        assert!(factors[0].evidence.contains("replica identity full"));
        assert!(!source_table_is_unsafe(&table));
    }

    #[test]
    fn clean_table_has_no_source_safety_factors() {
        let table = table();

        assert!(source_table_factors(&table).is_empty());
        assert!(!source_table_is_unsafe(&table));
    }

    fn table() -> TablePreflight {
        TablePreflight {
            schema: "public".to_string(),
            name: "sales".to_string(),
            exists: true,
            replica_identity: Some(ReplicaIdentity::Default),
            primary_key_columns: vec!["id".to_string()],
            columns: vec![TableColumn {
                ordinal_position: 1,
                name: "id".to_string(),
                type_oid: 25,
                type_name: "text".to_string(),
                nullable: false,
                is_key: true,
            }],
            schema_fingerprint: Some(42),
            update_delete_safe: true,
            issues: Vec::new(),
            contract_notes: Vec::new(),
        }
    }
}
