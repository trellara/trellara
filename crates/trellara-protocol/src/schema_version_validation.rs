use std::collections::HashSet;

use crate::{ProtocolError, RelationSchemaVersion};

pub(crate) fn validate_schema_versions(
    schema_versions: &[RelationSchemaVersion],
) -> Result<(), ProtocolError> {
    let mut relation_keys = HashSet::with_capacity(schema_versions.len());
    for schema_version in schema_versions {
        validate_schema_version(schema_version, &mut relation_keys)?;
    }
    Ok(())
}

fn validate_schema_version(
    schema_version: &RelationSchemaVersion,
    relation_keys: &mut HashSet<(String, String)>,
) -> Result<(), ProtocolError> {
    let Some(relation) = schema_version.relation.as_ref() else {
        return Err(invalid_schema_version(
            "<unknown>",
            "relation metadata is required",
        ));
    };
    let relation_name = relation.display_name();
    if relation.schema.trim().is_empty() || relation.table.trim().is_empty() {
        return Err(invalid_schema_version(
            relation_name,
            "relation schema and table must be present",
        ));
    }
    if schema_version.version == 0 {
        return Err(invalid_schema_version(
            relation_name,
            "version must be greater than zero",
        ));
    }
    if !relation_keys.insert((relation.schema.clone(), relation.table.clone())) {
        return Err(invalid_schema_version(
            relation_name,
            "duplicate relation schema version",
        ));
    }
    Ok(())
}

fn invalid_schema_version(relation: impl Into<String>, reason: impl Into<String>) -> ProtocolError {
    ProtocolError::InvalidSchemaVersionEvidence {
        relation: relation.into(),
        reason: reason.into(),
    }
}
