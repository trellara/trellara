use trellara_protocol::RelationId;

use crate::{decode_replica_identity, CapturedRelation, PgCapture, Result};

const LOAD_RELATIONS_SQL: &str = r#"
                select c.oid as oid,
                       n.nspname as schema_name,
                       c.relname as table_name,
                       c.relreplident::text as replica_identity
                  from pg_class c
                  join pg_namespace n on n.oid = c.relnamespace
                 where c.relkind in ('r', 'p')
                   and (n.nspname, c.relname) in (
                       select * from unnest($1::text[], $2::text[])
                   )
                 order by n.nspname, c.relname
                "#;

impl PgCapture {
    pub async fn load_relations(&self) -> Result<Vec<CapturedRelation>> {
        let rows = self
            .client
            .query(
                LOAD_RELATIONS_SQL,
                &[
                    &self
                        .config
                        .tables
                        .iter()
                        .map(|table| table.schema.clone())
                        .collect::<Vec<_>>(),
                    &self
                        .config
                        .tables
                        .iter()
                        .map(|table| table.name.clone())
                        .collect::<Vec<_>>(),
                ],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| CapturedRelation {
                id: RelationId::new(
                    row.get::<_, u32>("oid"),
                    row.get::<_, String>("schema_name"),
                    row.get::<_, String>("table_name"),
                ),
                replica_identity: decode_replica_identity(row.get::<_, String>("replica_identity")),
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "tests/tests_capture_relations.rs"]
mod tests;
