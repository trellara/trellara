use serde::Serialize;

pub const REQUIRED_REPLICA_IDENTITY: &str = "default_or_full_with_stable_key";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeRelationMetadata {
    pub relation_oid: u32,
    pub namespace: String,
    pub relation_name: String,
    pub schema_version_fingerprint: u64,
    pub required_replica_identity: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeRelationMetadataError {
    InvalidRelationOid,
    BlankNamespace,
    BlankRelationName,
    PaddedNamespace,
    PaddedRelationName,
    MissingSchemaFingerprint,
}

pub fn native_relation_metadata(
    relation_oid: u32,
    namespace: &str,
    relation_name: &str,
    schema_version_fingerprint: u64,
) -> Result<NativeRelationMetadata, NativeRelationMetadataError> {
    if relation_oid == 0 {
        return Err(NativeRelationMetadataError::InvalidRelationOid);
    }
    if schema_version_fingerprint == 0 {
        return Err(NativeRelationMetadataError::MissingSchemaFingerprint);
    }

    Ok(NativeRelationMetadata {
        relation_oid,
        namespace: validate_identifier(namespace, MetadataField::Namespace)?.to_owned(),
        relation_name: validate_identifier(relation_name, MetadataField::RelationName)?.to_owned(),
        schema_version_fingerprint,
        required_replica_identity: REQUIRED_REPLICA_IDENTITY,
    })
}

enum MetadataField {
    Namespace,
    RelationName,
}

fn validate_identifier(
    value: &str,
    field: MetadataField,
) -> Result<&str, NativeRelationMetadataError> {
    let trimmed = value.trim();
    match field {
        MetadataField::Namespace if trimmed.is_empty() => {
            Err(NativeRelationMetadataError::BlankNamespace)
        }
        MetadataField::RelationName if trimmed.is_empty() => {
            Err(NativeRelationMetadataError::BlankRelationName)
        }
        MetadataField::Namespace if trimmed != value => {
            Err(NativeRelationMetadataError::PaddedNamespace)
        }
        MetadataField::RelationName if trimmed != value => {
            Err(NativeRelationMetadataError::PaddedRelationName)
        }
        _ => Ok(value),
    }
}
