use crate::lsn::parse_lsn;
use crate::reader::PgOutputReader;
use crate::{error::protocol_byte_label, CaptureError, Result};

const LEGACY_COPY_BOTH_COLUMN_COUNT: u16 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct XLogData {
    pub(crate) wal_start: String,
    pub(crate) wal_end: String,
    pub(crate) send_timestamp_ms: i64,
    pub(crate) payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PrimaryKeepalive {
    pub(crate) wal_end: String,
    pub(crate) send_timestamp_ms: i64,
    pub(crate) reply_requested: bool,
}

pub(crate) enum ReplicationCopyData {
    XLogData(XLogData),
    PrimaryKeepalive(PrimaryKeepalive),
}

pub(crate) fn validate_copy_both_response(body: &[u8]) -> Result<()> {
    let mut reader = PgOutputReader::new(body);
    let overall_format = reader.read_u8()?;
    let column_count = reader.read_u16()?;
    let column_count = copy_both_column_format_count(column_count)?;
    let mut column_formats = Vec::with_capacity(column_count);
    for _ in 0..column_count {
        column_formats.push(reader.read_u16()?);
    }
    reader.expect_finished()?;
    if overall_format != 0 || !column_formats.iter().all(|format| *format == 0) {
        return Err(CaptureError::ReplicationProtocol(format!(
            "unexpected CopyBothResponse formats overall={overall_format} columns={column_formats:?}"
        )));
    }
    Ok(())
}

fn copy_both_column_format_count(column_count: u16) -> Result<usize> {
    if column_count != 0 && column_count != LEGACY_COPY_BOTH_COLUMN_COUNT {
        return Err(CaptureError::ReplicationProtocol(format!(
            "unexpected CopyBothResponse column count {column_count}, expected 0 or {LEGACY_COPY_BOTH_COLUMN_COUNT}"
        )));
    }
    Ok(usize::from(column_count))
}

pub(crate) fn parse_replication_copy_data(body: &[u8]) -> Result<ReplicationCopyData> {
    let mut reader = PgOutputReader::new(body);
    match reader.read_u8()? {
        b'w' => {
            let wal_start = reader.read_lsn()?;
            let wal_end = reader.read_lsn()?;
            let send_timestamp_ms = reader.read_pg_timestamp_ms()?;
            let payload = reader.read_remaining().to_vec();
            Ok(ReplicationCopyData::XLogData(XLogData {
                wal_start,
                wal_end,
                send_timestamp_ms,
                payload,
            }))
        }
        b'k' => {
            let keepalive = PrimaryKeepalive {
                wal_end: reader.read_lsn()?,
                send_timestamp_ms: reader.read_pg_timestamp_ms()?,
                reply_requested: reader.read_u8()? != 0,
            };
            reader.expect_finished()?;
            Ok(ReplicationCopyData::PrimaryKeepalive(keepalive))
        }
        tag => Err(CaptureError::ReplicationProtocol(format!(
            "unsupported replication CopyData tag {}",
            protocol_byte_label(tag)
        ))),
    }
}

pub(crate) fn replication_start_lsn(slot: &crate::LogicalSlotBootstrap) -> Result<String> {
    let Some(lsn) = slot.consistent_lsn.as_deref() else {
        return Err(CaptureError::ReplicationProtocol(
            "logical replication slot did not report a consistent or confirmed LSN".to_string(),
        ));
    };
    parse_lsn(lsn)?;
    Ok(lsn.to_string())
}
