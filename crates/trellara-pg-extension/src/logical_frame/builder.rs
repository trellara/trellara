use super::*;

impl NativeLogicalFrameBuilder {
    pub fn begin(
        &mut self,
        xid: u32,
        begin_lsn: u64,
        source_id: &str,
        dataset_id: &str,
    ) -> Result<(), NativeLogicalFrameError> {
        if self.active {
            return Err(NativeLogicalFrameError::AlreadyActive);
        }
        validate_identity(source_id, "source_id")?;
        validate_identity(dataset_id, "dataset_id")?;
        self.len = 0;
        self.event_count = 0;
        self.active = true;
        self.push_bytes(FRAME_MAGIC)?;
        self.push_u16(FRAME_VERSION)?;
        self.push_u32(xid)?;
        self.push_u64(begin_lsn)?;
        self.push_u64(0)?;
        self.push_u32(0)?;
        self.push_len_prefixed_u8(source_id.as_bytes())?;
        self.push_len_prefixed_u8(dataset_id.as_bytes())?;
        Ok(())
    }

    pub fn append_change_start(
        &mut self,
        operation: NativeLogicalOperation,
        relation_oid: u32,
        namespace: &[u8],
        relation_name: &[u8],
    ) -> Result<(), NativeLogicalFrameError> {
        self.require_active()?;
        self.push_u8(operation as u8)?;
        self.push_u32(relation_oid)?;
        self.push_len_prefixed_u8(namespace)?;
        self.push_len_prefixed_u8(relation_name)?;
        Ok(())
    }

    pub fn append_message(
        &mut self,
        message_lsn: u64,
        transactional: bool,
        prefix: &[u8],
        message: &[u8],
    ) -> Result<(), NativeLogicalFrameError> {
        self.require_active()?;
        let message_len =
            u32::try_from(message.len()).map_err(|_| NativeLogicalFrameError::ValueTooLong {
                length: message.len(),
            })?;
        self.push_u8(NativeLogicalOperation::Message as u8)?;
        self.push_u64(message_lsn)?;
        self.push_u8(u8::from(transactional))?;
        self.push_len_prefixed_u8(prefix)?;
        self.push_u32(message_len)?;
        self.push_bytes(message)?;
        Ok(())
    }

    pub fn append_tuple_start(
        &mut self,
        attribute_count: u16,
    ) -> Result<(), NativeLogicalFrameError> {
        self.require_active()?;
        self.push_u16(attribute_count)
    }

    pub fn append_tuple_presence(&mut self, present: bool) -> Result<(), NativeLogicalFrameError> {
        self.require_active()?;
        self.push_u8(u8::from(present))
    }

    pub fn append_column(
        &mut self,
        type_oid: u32,
        status: NativeLogicalColumnStatus,
        value: &[u8],
    ) -> Result<(), NativeLogicalFrameError> {
        self.require_active()?;
        let length =
            u32::try_from(value.len()).map_err(|_| NativeLogicalFrameError::ValueTooLong {
                length: value.len(),
            })?;
        self.push_u32(type_oid)?;
        self.push_u8(status as u8)?;
        self.push_u32(length)?;
        self.push_bytes(value)
    }

    pub fn finish_event(&mut self) -> Result<(), NativeLogicalFrameError> {
        self.require_active()?;
        self.event_count =
            self.event_count
                .checked_add(1)
                .ok_or(NativeLogicalFrameError::CountOverflow {
                    field: "event_count",
                })?;
        Ok(())
    }

    pub fn finish(&mut self, commit_lsn: u64) -> Result<&[u8], NativeLogicalFrameError> {
        self.require_active()?;
        if self.event_count == 0 {
            return Err(NativeLogicalFrameError::EmptyTransaction);
        }
        self.bytes[COMMIT_LSN_OFFSET..COMMIT_LSN_OFFSET + 8]
            .copy_from_slice(&commit_lsn.to_be_bytes());
        self.bytes[EVENT_COUNT_OFFSET..EVENT_COUNT_OFFSET + 4]
            .copy_from_slice(&self.event_count.to_be_bytes());
        self.active = false;
        Ok(&self.bytes[..self.len])
    }

    pub fn abort(&mut self) {
        self.len = 0;
        self.event_count = 0;
        self.active = false;
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub fn has_events(&self) -> bool {
        self.event_count != 0
    }
}

fn validate_identity(value: &str, field: &'static str) -> Result<(), NativeLogicalFrameError> {
    if value.is_empty() {
        return Err(NativeLogicalFrameError::EmptyIdentity { field });
    }
    if value.len() > usize::from(u8::MAX) {
        return Err(NativeLogicalFrameError::IdentityTooLong {
            field,
            length: value.len(),
        });
    }
    Ok(())
}
