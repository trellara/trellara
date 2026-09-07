use trellara_protocol::ProtocolError;

use crate::{CaptureError, Result};

pub(crate) fn format_lsn(lsn: u64) -> String {
    trellara_protocol::format_lsn(lsn)
}

pub(crate) fn parse_lsn(lsn: &str) -> Result<u64> {
    trellara_protocol::parse_lsn(lsn).map_err(|error| match error {
        ProtocolError::InvalidLsn { lsn, reason } => {
            let reason = if reason.contains("field must fit in 32 bits") {
                "LSN halves must fit in 32 bits".to_string()
            } else {
                reason
            };
            CaptureError::ReplicationProtocol(format!("invalid LSN {lsn}: {reason}"))
        }
        other => CaptureError::ReplicationProtocol(other.to_string()),
    })
}
