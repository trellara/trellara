use crate::lsn::{format_lsn, parse_lsn};
use crate::{CaptureError, Result};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DurableSourceAckBoundary {
    last_acknowledged_lsn: u64,
}

impl DurableSourceAckBoundary {
    pub(crate) fn last_acknowledged_lsn(&self) -> u64 {
        self.last_acknowledged_lsn
    }

    pub(crate) fn acknowledge(&mut self, lsn: &str) -> Result<u64> {
        let parsed_lsn = parse_lsn(lsn)?;
        if parsed_lsn == 0 {
            return Err(CaptureError::ReplicationProtocol(
                "cannot acknowledge durable source LSN 0/0".to_string(),
            ));
        }
        if parsed_lsn < self.last_acknowledged_lsn {
            return Err(CaptureError::ReplicationProtocol(format!(
                "cannot move durable source acknowledgement backward from {} to {}",
                format_lsn(self.last_acknowledged_lsn),
                format_lsn(parsed_lsn)
            )));
        }
        self.last_acknowledged_lsn = parsed_lsn;
        Ok(parsed_lsn)
    }
}
