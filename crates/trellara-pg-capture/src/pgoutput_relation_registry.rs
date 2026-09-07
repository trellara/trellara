use std::collections::HashMap;

use crate::{CaptureError, PgOutputRelation, Result};

#[derive(Default)]
pub(crate) struct PgOutputRelationRegistry {
    relations: HashMap<u32, PgOutputRelation>,
}

impl PgOutputRelationRegistry {
    pub(crate) fn register(&mut self, relation: PgOutputRelation) -> Result<()> {
        if let Some(previous) = self.relations.get(&relation.id.oid) {
            let previous_fingerprint = previous.schema_fingerprint();
            let new_fingerprint = relation.schema_fingerprint();
            if previous_fingerprint != new_fingerprint {
                return Err(CaptureError::PgOutputSchemaChanged {
                    relation: relation.id.display_name(),
                    previous_fingerprint,
                    new_fingerprint,
                });
            }
        }
        self.relations.insert(relation.id.oid, relation);
        Ok(())
    }

    pub(crate) fn get(&self, oid: u32) -> Result<&PgOutputRelation> {
        self.relations.get(&oid).ok_or_else(|| {
            CaptureError::PgOutputParse(format!("relation metadata for oid {oid} has not arrived"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PgOutputReader;

    #[test]
    fn registry_accepts_repeated_matching_relation_metadata() {
        let mut registry = PgOutputRelationRegistry::default();
        let relation = relation("public", "sales", b'd', &[("id", 25, true)]);

        registry
            .register(relation.clone())
            .expect("first relation metadata");
        registry
            .register(relation)
            .expect("matching relation metadata");

        assert_eq!(
            registry.get(16_384).expect("relation").id.display_name(),
            "public.sales"
        );
    }

    #[test]
    fn registry_rejects_schema_fingerprint_drift() {
        let mut registry = PgOutputRelationRegistry::default();
        registry
            .register(relation("public", "sales", b'd', &[("id", 25, true)]))
            .expect("first relation metadata");

        assert!(matches!(
            registry.register(relation(
                "public",
                "sales",
                b'd',
                &[("id", 25, true), ("amount_cents", 20, false)]
            )),
            Err(CaptureError::PgOutputSchemaChanged {
                relation,
                previous_fingerprint,
                new_fingerprint,
            }) if relation == "public.sales" && previous_fingerprint != new_fingerprint
        ));
    }

    #[test]
    fn registry_rejects_missing_relation_lookup() {
        let registry = PgOutputRelationRegistry::default();

        assert!(matches!(
            registry.get(42),
            Err(CaptureError::PgOutputParse(message))
                if message.contains("relation metadata for oid 42 has not arrived")
        ));
    }

    fn relation(
        schema: &str,
        table: &str,
        identity: u8,
        columns: &[(&str, u32, bool)],
    ) -> PgOutputRelation {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&16_384u32.to_be_bytes());
        bytes.extend_from_slice(schema.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(table.as_bytes());
        bytes.push(0);
        bytes.push(identity);
        bytes.extend_from_slice(&(columns.len() as u16).to_be_bytes());
        for (name, type_oid, is_key) in columns {
            bytes.push(u8::from(*is_key));
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&type_oid.to_be_bytes());
            bytes.extend_from_slice(&(-1i32).to_be_bytes());
        }
        let mut reader = PgOutputReader::new(&bytes);
        PgOutputRelation::parse(&mut reader).expect("parse relation")
    }
}
