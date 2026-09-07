use super::*;

impl NativeLogicalFrameBuilder {
    pub(super) fn require_active(&self) -> Result<(), NativeLogicalFrameError> {
        if self.active {
            Ok(())
        } else {
            Err(NativeLogicalFrameError::NotActive)
        }
    }

    pub(super) fn push_len_prefixed_u8(
        &mut self,
        value: &[u8],
    ) -> Result<(), NativeLogicalFrameError> {
        let length =
            u8::try_from(value.len()).map_err(|_| NativeLogicalFrameError::IdentityTooLong {
                field: "logical_identifier",
                length: value.len(),
            })?;
        self.push_u8(length)?;
        self.push_bytes(value)
    }

    pub(super) fn push_u8(&mut self, value: u8) -> Result<(), NativeLogicalFrameError> {
        self.push_bytes(&[value])
    }

    pub(super) fn push_u16(&mut self, value: u16) -> Result<(), NativeLogicalFrameError> {
        self.push_bytes(&value.to_be_bytes())
    }

    pub(super) fn push_u32(&mut self, value: u32) -> Result<(), NativeLogicalFrameError> {
        self.push_bytes(&value.to_be_bytes())
    }

    pub(super) fn push_u64(&mut self, value: u64) -> Result<(), NativeLogicalFrameError> {
        self.push_bytes(&value.to_be_bytes())
    }

    pub(super) fn push_bytes(&mut self, value: &[u8]) -> Result<(), NativeLogicalFrameError> {
        let end =
            self.len
                .checked_add(value.len())
                .ok_or(NativeLogicalFrameError::FrameTooLarge {
                    attempted_bytes: usize::MAX,
                    max_bytes: MAX_LOGICAL_FRAME_BYTES,
                })?;
        if end > MAX_LOGICAL_FRAME_BYTES {
            return Err(NativeLogicalFrameError::FrameTooLarge {
                attempted_bytes: end,
                max_bytes: MAX_LOGICAL_FRAME_BYTES,
            });
        }
        self.bytes[self.len..end].copy_from_slice(value);
        self.len = end;
        Ok(())
    }
}
