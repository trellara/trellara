use bytes::Buf;

use crate::{lsn::format_lsn, CaptureError, Result};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PgOutputReader<'a> {
    remaining: &'a [u8],
}

impl<'a> PgOutputReader<'a> {
    pub(crate) fn new(payload: &'a [u8]) -> Self {
        Self { remaining: payload }
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8> {
        if self.remaining.is_empty() {
            return Err(CaptureError::PgOutputParse(
                "unexpected end of pgoutput message".to_string(),
            ));
        }
        let value = self.remaining[0];
        self.remaining = &self.remaining[1..];
        Ok(value)
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        let mut bytes = self.read_exact(2)?;
        Ok(bytes.get_u16())
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        let mut bytes = self.read_exact(4)?;
        Ok(bytes.get_u32())
    }

    pub(crate) fn read_stream_xid(
        &mut self,
        current_stream_xid: Option<u32>,
    ) -> Result<Option<String>> {
        if current_stream_xid.is_some() {
            Ok(Some(self.read_u32()?.to_string()))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn read_i32(&mut self) -> Result<i32> {
        let mut bytes = self.read_exact(4)?;
        Ok(bytes.get_i32())
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64> {
        let mut bytes = self.read_exact(8)?;
        Ok(bytes.get_i64())
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64> {
        let mut bytes = self.read_exact(8)?;
        Ok(bytes.get_u64())
    }

    pub(crate) fn read_lsn(&mut self) -> Result<String> {
        Ok(format_lsn(self.read_u64()?))
    }

    pub(crate) fn read_pg_timestamp_ms(&mut self) -> Result<i64> {
        const POSTGRES_EPOCH_UNIX_MS: i64 = 946_684_800_000;
        Ok(POSTGRES_EPOCH_UNIX_MS + (self.read_i64()? / 1_000))
    }

    pub(crate) fn read_cstr(&mut self) -> Result<String> {
        let Some(end) = self.remaining.iter().position(|byte| *byte == 0) else {
            return Err(CaptureError::PgOutputParse(
                "unterminated pgoutput string".to_string(),
            ));
        };
        let value = String::from_utf8_lossy(&self.remaining[..end]).into_owned();
        self.remaining = &self.remaining[end + 1..];
        Ok(value)
    }

    pub(crate) fn read_sized_bytes(&mut self) -> Result<&'a [u8]> {
        let len = self.read_i32()?;
        self.read_exact(pgoutput_tuple_value_len(len)?)
    }

    pub(crate) fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.remaining.len() < len {
            return Err(CaptureError::PgOutputParse(format!(
                "pgoutput message ended while reading {len} bytes"
            )));
        }
        let (value, rest) = self.remaining.split_at(len);
        self.remaining = rest;
        Ok(value)
    }

    pub(crate) fn read_remaining(&mut self) -> &'a [u8] {
        let value = self.remaining;
        self.remaining = &[];
        value
    }

    pub(crate) fn remaining_len(&self) -> usize {
        self.remaining.len()
    }

    pub(crate) fn expect_finished(&self) -> Result<()> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(CaptureError::PgOutputParse(format!(
                "pgoutput message has {} trailing bytes",
                self.remaining.len()
            )))
        }
    }
}

pub(crate) fn pgoutput_tuple_value_len(len: i32) -> Result<usize> {
    usize::try_from(len).map_err(|_| {
        CaptureError::PgOutputParse("negative pgoutput tuple value length".to_string())
    })
}
