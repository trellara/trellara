use trellara_protocol::{RelationId, ReplicaIdentity, RowImage};

use crate::{
    error::protocol_byte_label, pgoutput_relation_fingerprint::schema_fingerprint,
    pgoutput_tuple::parse_tuple, CaptureError, PgOutputReader, Result,
};

const MAX_PGOUTPUT_RELATION_COLUMNS: u16 = 1600;
const PGOUTPUT_RELATION_KEY_FLAG: u8 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PgOutputRelation {
    pub(crate) id: RelationId,
    pub(crate) replica_identity: ReplicaIdentity,
    columns: Vec<PgOutputColumn>,
}
impl PgOutputRelation {
    pub(crate) fn parse(reader: &mut PgOutputReader<'_>) -> Result<Self> {
        let oid = reader.read_u32()?;
        let schema = reader.read_cstr()?;
        let table = reader.read_cstr()?;
        let replica_identity = decode_pgoutput_replica_identity(reader.read_u8()?)?;
        let column_count = reader.read_u16()?;
        let column_count = pgoutput_relation_column_count(&schema, &table, column_count)?;
        let mut columns = Vec::with_capacity(column_count);
        for _ in 0..column_count {
            let flags = reader.read_u8()?;
            let name = reader.read_cstr()?;
            columns.push(PgOutputColumn {
                is_key: pgoutput_relation_column_is_key(&schema, &table, &name, flags)?,
                name,
                type_oid: reader.read_u32()?,
                type_modifier: reader.read_i32()?,
            });
        }
        Ok(Self {
            id: RelationId::new(oid, schema, table),
            replica_identity,
            columns,
        })
    }

    pub(crate) fn parse_tuple(&self, reader: &mut PgOutputReader<'_>) -> Result<RowImage> {
        parse_tuple(&self.id, &self.columns, reader)
    }

    pub(crate) fn schema_fingerprint(&self) -> u64 {
        schema_fingerprint(self.replica_identity, &self.columns)
    }
}

fn decode_pgoutput_replica_identity(value: u8) -> Result<ReplicaIdentity> {
    match value {
        b'd' => Ok(ReplicaIdentity::Default),
        b'i' => Ok(ReplicaIdentity::Index),
        b'f' => Ok(ReplicaIdentity::Full),
        b'n' => Ok(ReplicaIdentity::Nothing),
        value => Err(CaptureError::PgOutputParse(format!(
            "unsupported pgoutput replica identity {}",
            protocol_byte_label(value)
        ))),
    }
}

fn pgoutput_relation_column_count(schema: &str, table: &str, column_count: u16) -> Result<usize> {
    if column_count > MAX_PGOUTPUT_RELATION_COLUMNS {
        return Err(CaptureError::PgOutputParse(format!(
            "relation {schema}.{table} declares {column_count} columns, above supported maximum {MAX_PGOUTPUT_RELATION_COLUMNS}"
        )));
    }
    Ok(usize::from(column_count))
}

fn pgoutput_relation_column_is_key(
    schema: &str,
    table: &str,
    column: &str,
    flags: u8,
) -> Result<bool> {
    if flags & !PGOUTPUT_RELATION_KEY_FLAG != 0 {
        return Err(CaptureError::PgOutputParse(format!(
            "relation {schema}.{table} column {column} has unsupported flag bits {}",
            protocol_byte_label(flags)
        )));
    }
    Ok(flags & PGOUTPUT_RELATION_KEY_FLAG == PGOUTPUT_RELATION_KEY_FLAG)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PgOutputColumn {
    pub(crate) is_key: bool,
    pub(crate) name: String,
    pub(crate) type_oid: u32,
    pub(crate) type_modifier: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relation_message(column_count: u16) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(&42_u32.to_be_bytes());
        message.extend_from_slice(b"public\0");
        message.extend_from_slice(b"wide_table\0");
        message.push(b'd');
        message.extend_from_slice(&column_count.to_be_bytes());
        message
    }

    #[test]
    fn pgoutput_relation_column_count_accepts_postgres_limit() {
        assert_eq!(
            pgoutput_relation_column_count("public", "wide_table", MAX_PGOUTPUT_RELATION_COLUMNS)
                .expect("column count"),
            usize::from(MAX_PGOUTPUT_RELATION_COLUMNS)
        );
    }

    #[test]
    fn pgoutput_relation_column_count_rejects_absurd_relation_before_allocation() {
        let error = pgoutput_relation_column_count(
            "public",
            "wide_table",
            MAX_PGOUTPUT_RELATION_COLUMNS + 1,
        )
        .expect_err("wide relation");

        assert!(
            matches!(error, CaptureError::PgOutputParse(message) if message.contains("wide_table") && message.contains("above supported maximum"))
        );
    }

    #[test]
    fn relation_parse_rejects_absurd_column_count_before_reading_columns() {
        let mut message = relation_message(MAX_PGOUTPUT_RELATION_COLUMNS + 1);
        message.extend_from_slice(b"column-payload");
        let mut reader = PgOutputReader::new(&message);

        let error = PgOutputRelation::parse(&mut reader).expect_err("wide relation");

        assert!(
            matches!(error, CaptureError::PgOutputParse(message) if message.contains("wide_table") && message.contains("above supported maximum"))
        );
        assert_eq!(reader.remaining_len(), b"column-payload".len());
    }
}
