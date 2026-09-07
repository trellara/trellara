use super::*;

pub(in crate::tests) fn preflight_table(
    schema_fingerprint: u64,
) -> trellara_pg_capture::TablePreflight {
    trellara_pg_capture::TablePreflight {
        schema: "public".to_string(),
        name: "sales".to_string(),
        exists: true,
        replica_identity: Some(trellara_protocol::ReplicaIdentity::Full),
        primary_key_columns: vec!["id".to_string()],
        columns: vec![trellara_pg_capture::TableColumn {
            ordinal_position: 1,
            name: "id".to_string(),
            type_oid: 25,
            type_name: "text".to_string(),
            nullable: false,
            is_key: true,
        }],
        schema_fingerprint: Some(schema_fingerprint),
        update_delete_safe: true,
        issues: Vec::new(),
        contract_notes: Vec::new(),
    }
}

pub(in crate::tests) fn source_schema_drift() -> FlowSchemaDriftSummary {
    FlowSchemaDriftSummary {
            relation_count: 1,
            relations: vec!["public.sales".to_string()],
            reason: "configured source schema fingerprint no longer matches live pgoutput metadata"
                .to_string(),
            recommendation:
                "pause CDC, run schema-discover and contract-test, then create a fresh snapshot-to-stream handoff before resuming"
                    .to_string(),
        }
}

pub(in crate::tests) fn preflight_table_with_columns(
    columns: Vec<trellara_pg_capture::TableColumn>,
) -> trellara_pg_capture::TablePreflight {
    trellara_pg_capture::TablePreflight {
        columns,
        ..preflight_table(42)
    }
}

pub(in crate::tests) fn preflight_table_named(
    schema: &str,
    name: &str,
) -> trellara_pg_capture::TablePreflight {
    trellara_pg_capture::TablePreflight {
        schema: schema.to_string(),
        name: name.to_string(),
        ..preflight_table(42)
    }
}

pub(in crate::tests) fn source_column(
    name: &str,
    type_name: &str,
) -> trellara_pg_capture::TableColumn {
    trellara_pg_capture::TableColumn {
        ordinal_position: 1,
        name: name.to_string(),
        type_oid: 25,
        type_name: type_name.to_string(),
        nullable: false,
        is_key: name == "id",
    }
}

pub(in crate::tests) fn key_column(name: &str, nullable: bool) -> trellara_pg_capture::TableColumn {
    trellara_pg_capture::TableColumn {
        ordinal_position: 2,
        name: name.to_string(),
        type_oid: 25,
        type_name: "text".to_string(),
        nullable,
        is_key: true,
    }
}

pub(in crate::tests) fn target_inspection(
    columns: Vec<trellara_verify::PostgresColumnInspection>,
) -> PostgresRelationInspection {
    PostgresRelationInspection {
        relation: trellara_protocol::RelationId::new(0, "public", "sales"),
        exists: true,
        columns,
    }
}

pub(in crate::tests) fn missing_target_inspection() -> PostgresRelationInspection {
    PostgresRelationInspection {
        relation: trellara_protocol::RelationId::new(0, "public", "sales"),
        exists: false,
        columns: Vec::new(),
    }
}

pub(in crate::tests) fn target_column(
    name: &str,
    type_name: &str,
) -> trellara_verify::PostgresColumnInspection {
    trellara_verify::PostgresColumnInspection {
        ordinal_position: 1,
        name: name.to_string(),
        type_name: type_name.to_string(),
        nullable: false,
    }
}
