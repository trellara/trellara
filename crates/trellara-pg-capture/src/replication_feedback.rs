use std::time::{SystemTime, UNIX_EPOCH};

use bytes::{BufMut, BytesMut};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StandbyStatusBoundary {
    pub(crate) acknowledged_lsn: u64,
    pub(crate) written_lsn: u64,
    pub(crate) flushed_lsn: u64,
    pub(crate) applied_lsn: u64,
    pub(crate) timestamp_micros: i64,
    pub(crate) reply_requested: bool,
}

impl StandbyStatusBoundary {
    pub(crate) fn acknowledged(
        acknowledged_lsn: u64,
        timestamp_micros: i64,
        reply_requested: bool,
    ) -> Self {
        Self {
            acknowledged_lsn,
            written_lsn: acknowledged_lsn,
            flushed_lsn: acknowledged_lsn,
            applied_lsn: acknowledged_lsn,
            timestamp_micros,
            reply_requested,
        }
    }

    pub(crate) fn uses_only_acknowledged_lsn(&self) -> bool {
        self.written_lsn == self.acknowledged_lsn
            && self.flushed_lsn == self.acknowledged_lsn
            && self.applied_lsn == self.acknowledged_lsn
    }
}

pub(crate) fn standby_status_update_payload(
    lsn: u64,
    timestamp_micros: i64,
    reply_requested: bool,
) -> BytesMut {
    standby_status_boundary_payload(&StandbyStatusBoundary::acknowledged(
        lsn,
        timestamp_micros,
        reply_requested,
    ))
}

pub(crate) fn standby_status_boundary_payload(boundary: &StandbyStatusBoundary) -> BytesMut {
    debug_assert!(boundary.uses_only_acknowledged_lsn());
    let mut payload = BytesMut::with_capacity(34);
    payload.put_u8(b'r');
    payload.put_u64(boundary.written_lsn);
    payload.put_u64(boundary.flushed_lsn);
    payload.put_u64(boundary.applied_lsn);
    payload.put_i64(boundary.timestamp_micros);
    payload.put_u8(u8::from(boundary.reply_requested));
    payload
}

pub(crate) fn postgres_epoch_now_micros() -> i64 {
    const PG_EPOCH_UNIX_OFFSET_MICROS: i64 = 946_684_800_000_000;
    let unix_micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros()
        .min(i64::MAX as u128) as i64;
    unix_micros - PG_EPOCH_UNIX_OFFSET_MICROS
}
