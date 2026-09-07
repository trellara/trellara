use trellara_protocol::Operation;

use crate::pgoutput_relation_registry::PgOutputRelationRegistry;
use crate::pgoutput_stream_state::PgOutputStreamState;
use crate::{error::protocol_byte_label, CaptureError, LogicalEvent, PgOutputReader, Result};

pub(crate) fn decode_insert(
    relations: &PgOutputRelationRegistry,
    stream: &PgOutputStreamState,
    reader: &mut PgOutputReader<'_>,
) -> Result<LogicalEvent> {
    let transaction_id = reader.read_stream_xid(stream.active_xid())?;
    let relation_oid = reader.read_u32()?;
    let tuple_tag = reader.read_u8()?;
    if tuple_tag != b'N' {
        return Err(CaptureError::PgOutputParse(format!(
            "insert expected new tuple tag N, got {}",
            protocol_byte_label(tuple_tag)
        )));
    }
    let relation = relations.get(relation_oid)?.clone();
    let after = relation.parse_tuple(reader)?;
    Ok(LogicalEvent::Change {
        transaction_id,
        relation: relation.id,
        operation: Operation::Insert,
        replica_identity: relation.replica_identity,
        before: None,
        after: Some(after),
    })
}

pub(crate) fn decode_update(
    relations: &PgOutputRelationRegistry,
    stream: &PgOutputStreamState,
    reader: &mut PgOutputReader<'_>,
) -> Result<LogicalEvent> {
    let transaction_id = reader.read_stream_xid(stream.active_xid())?;
    let relation_oid = reader.read_u32()?;
    let relation = relations.get(relation_oid)?.clone();
    let mut before = None;
    let first_tag = reader.read_u8()?;
    let new_tag = match first_tag {
        b'K' | b'O' => {
            before = Some(relation.parse_tuple(reader)?);
            reader.read_u8()?
        }
        b'N' => b'N',
        _ => {
            return Err(CaptureError::PgOutputParse(format!(
                "update expected tuple tag K, O, or N, got {}",
                protocol_byte_label(first_tag)
            )));
        }
    };
    if new_tag != b'N' {
        return Err(CaptureError::PgOutputParse(format!(
            "update expected new tuple tag N, got {}",
            protocol_byte_label(new_tag)
        )));
    }
    let after = relation.parse_tuple(reader)?;
    Ok(LogicalEvent::Change {
        transaction_id,
        relation: relation.id,
        operation: Operation::Update,
        replica_identity: relation.replica_identity,
        before,
        after: Some(after),
    })
}

pub(crate) fn decode_delete(
    relations: &PgOutputRelationRegistry,
    stream: &PgOutputStreamState,
    reader: &mut PgOutputReader<'_>,
) -> Result<LogicalEvent> {
    let transaction_id = reader.read_stream_xid(stream.active_xid())?;
    let relation_oid = reader.read_u32()?;
    let relation = relations.get(relation_oid)?.clone();
    let tuple_tag = reader.read_u8()?;
    if !matches!(tuple_tag, b'K' | b'O') {
        return Err(CaptureError::PgOutputParse(format!(
            "delete expected tuple tag K or O, got {}",
            protocol_byte_label(tuple_tag)
        )));
    }
    let before = relation.parse_tuple(reader)?;
    Ok(LogicalEvent::Change {
        transaction_id,
        relation: relation.id,
        operation: Operation::Delete,
        replica_identity: relation.replica_identity,
        before: Some(before),
        after: None,
    })
}
