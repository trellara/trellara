use crate::pgoutput_changes::{decode_delete, decode_insert, decode_update};
use crate::pgoutput_relation_registry::PgOutputRelationRegistry;
use crate::pgoutput_stream_state::PgOutputStreamState;
use crate::pgoutput_truncate::decode_truncate;
use crate::{
    error::protocol_byte_label, CaptureError, LogicalEvent, PgOutputReader, PgOutputRelation,
    Result,
};

#[derive(Default)]
pub struct PgOutputDecoder {
    relations: PgOutputRelationRegistry,
    stream: PgOutputStreamState,
}

impl PgOutputDecoder {
    pub fn decode(&mut self, payload: &[u8]) -> Result<Option<LogicalEvent>> {
        let mut reader = PgOutputReader::new(payload);
        let message_type = reader.read_u8()?;
        let event = match message_type {
            b'B' => {
                let final_lsn = reader.read_lsn()?;
                let _commit_timestamp_ms = reader.read_pg_timestamp_ms()?;
                let xid = reader.read_u32()?;
                Some(LogicalEvent::Begin {
                    transaction_id: xid.to_string(),
                    begin_lsn: final_lsn,
                })
            }
            b'C' => {
                let _flags = reader.read_u8()?;
                let _commit_lsn = reader.read_lsn()?;
                let end_lsn = reader.read_lsn()?;
                let commit_timestamp_ms = reader.read_pg_timestamp_ms()?;
                Some(LogicalEvent::Commit {
                    commit_lsn: end_lsn,
                    commit_timestamp_ms,
                })
            }
            b'R' => {
                reader.read_stream_xid(self.stream.active_xid())?;
                let relation = PgOutputRelation::parse(&mut reader)?;
                let schema_fingerprint = relation.schema_fingerprint();
                let relation_id = relation.id.clone();
                reader.expect_finished()?;
                self.relations.register(relation)?;
                Some(LogicalEvent::RelationMetadata {
                    relation: relation_id,
                    schema_fingerprint,
                })
            }
            b'I' => Some(decode_insert(&self.relations, &self.stream, &mut reader)?),
            b'U' => Some(decode_update(&self.relations, &self.stream, &mut reader)?),
            b'D' => Some(decode_delete(&self.relations, &self.stream, &mut reader)?),
            b'T' => Some(decode_truncate(&self.relations, &self.stream, &mut reader)?),
            b'O' => {
                let _origin_lsn = reader.read_lsn()?;
                let _origin_name = reader.read_cstr()?;
                None
            }
            b'Y' => {
                reader.read_stream_xid(self.stream.active_xid())?;
                let _type_oid = reader.read_u32()?;
                let _schema = reader.read_cstr()?;
                let _name = reader.read_cstr()?;
                None
            }
            b'S' => {
                let xid = reader.read_u32()?;
                let first_segment = stream_start_first_segment(reader.read_u8()?)?;
                reader.expect_finished()?;
                self.stream.start(xid, first_segment)?;
                Some(LogicalEvent::StreamStart {
                    transaction_id: xid.to_string(),
                    first_segment,
                })
            }
            b'E' => {
                reader.expect_finished()?;
                self.stream.stop()?;
                Some(LogicalEvent::StreamStop)
            }
            b'c' => {
                let xid = reader.read_u32()?;
                let _flags = reader.read_u8()?;
                let _commit_lsn = reader.read_lsn()?;
                let end_lsn = reader.read_lsn()?;
                let commit_timestamp_ms = reader.read_pg_timestamp_ms()?;
                reader.expect_finished()?;
                self.stream.commit(xid)?;
                Some(LogicalEvent::StreamCommit {
                    transaction_id: xid.to_string(),
                    commit_lsn: end_lsn,
                    commit_timestamp_ms,
                })
            }
            b'A' => {
                let xid = reader.read_u32()?;
                let subxid = reader.read_u32()?;
                reader.expect_finished()?;
                self.stream.abort(xid, subxid)?;
                Some(LogicalEvent::StreamAbort {
                    transaction_id: xid.to_string(),
                    subtransaction_id: subxid.to_string(),
                })
            }
            message_type => {
                return Err(CaptureError::PgOutputParse(format!(
                    "unsupported pgoutput message type {}",
                    protocol_byte_label(message_type)
                )));
            }
        };
        reader.expect_finished()?;
        Ok(event)
    }
}

fn stream_start_first_segment(value: u8) -> Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(CaptureError::PgOutputParse(format!(
            "stream start first-segment flag must be 0 or 1, got {}",
            protocol_byte_label(value)
        ))),
    }
}
