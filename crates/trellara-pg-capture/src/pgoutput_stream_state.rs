use std::collections::HashSet;

use crate::{CaptureError, Result};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PgOutputStreamState {
    active_xid: Option<u32>,
    open_xids: HashSet<u32>,
}

impl PgOutputStreamState {
    pub(crate) fn active_xid(&self) -> Option<u32> {
        self.active_xid
    }

    pub(crate) fn start(&mut self, xid: u32, first_segment: bool) -> Result<()> {
        if let Some(active_xid) = self.active_xid {
            return Err(CaptureError::PgOutputParse(format!(
                "stream start for xid {xid} arrived while stream xid {active_xid} is still active"
            )));
        }
        if first_segment {
            if self.open_xids.contains(&xid) {
                return Err(CaptureError::PgOutputParse(format!(
                    "first stream segment for xid {xid} arrived after the transaction was already open"
                )));
            }
            self.open_xids.insert(xid);
        } else if !self.open_xids.contains(&xid) {
            return Err(CaptureError::PgOutputParse(format!(
                "non-first stream segment for xid {xid} arrived before a first segment"
            )));
        }
        self.active_xid = Some(xid);
        Ok(())
    }

    pub(crate) fn stop(&mut self) -> Result<()> {
        if self.active_xid.is_none() {
            return Err(CaptureError::PgOutputParse(
                "stream stop arrived without an active stream".to_string(),
            ));
        }
        self.active_xid = None;
        Ok(())
    }

    pub(crate) fn commit(&mut self, xid: u32) -> Result<()> {
        if self.active_xid.is_some() {
            return Err(CaptureError::PgOutputParse(format!(
                "stream commit for xid {xid} arrived before stream stop"
            )));
        }
        if !self.open_xids.remove(&xid) {
            return Err(CaptureError::PgOutputParse(format!(
                "stream commit for xid {xid} arrived without an open streamed transaction"
            )));
        }
        Ok(())
    }

    pub(crate) fn abort(&mut self, xid: u32, subxid: u32) -> Result<()> {
        if let Some(active_xid) = self.active_xid {
            if active_xid != xid {
                return Err(CaptureError::PgOutputParse(format!(
                    "stream abort for xid {xid} arrived while stream xid {active_xid} is still active"
                )));
            }
            if xid == subxid {
                self.active_xid = None;
            }
        }
        if !self.open_xids.contains(&xid) {
            return Err(CaptureError::PgOutputParse(format!(
                "stream abort for xid {xid} arrived without an open streamed transaction"
            )));
        }
        if xid == subxid {
            self.open_xids.remove(&xid);
        }
        Ok(())
    }
}
