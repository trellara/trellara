use std::path::Path;

use crate::{LocalStreamError, Result};

pub(crate) const MAX_FRAME_BODY_BYTES: u32 = 128 * 1024 * 1024;
pub(crate) const MAX_RECORD_FIELD_BYTES: u32 = 128 * 1024 * 1024;

pub(crate) fn checked_write_len(field: &'static str, len: usize) -> Result<u32> {
    len.try_into()
        .map_err(|_| LocalStreamError::FieldTooLarge { field })
}

pub(crate) fn checked_allocation_len(field: &'static str, len: u32) -> Result<usize> {
    usize::try_from(len).map_err(|_| LocalStreamError::FieldTooLarge { field })
}

pub(crate) fn checked_frame_body_len(path: &Path, len: u32) -> Result<usize> {
    checked_bounded_allocation_len(path, "frame_body", len, MAX_FRAME_BODY_BYTES)
}

pub(crate) fn checked_record_field_len(path: &Path, len: u32) -> Result<usize> {
    checked_bounded_allocation_len(path, "record_field", len, MAX_RECORD_FIELD_BYTES)
}

fn checked_bounded_allocation_len(
    path: &Path,
    field: &'static str,
    len: u32,
    max_len: u32,
) -> Result<usize> {
    if len > max_len {
        return Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        });
    }
    checked_allocation_len(field, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_frame_body_len_rejects_absurd_length_before_allocation() {
        let path = Path::new("stream.log");

        assert!(matches!(
            checked_frame_body_len(path, MAX_FRAME_BODY_BYTES + 1),
            Err(LocalStreamError::CorruptFrame { .. })
        ));
    }

    #[test]
    fn checked_record_field_len_rejects_absurd_length_before_allocation() {
        let path = Path::new("stream.log");

        assert!(matches!(
            checked_record_field_len(path, MAX_RECORD_FIELD_BYTES + 1),
            Err(LocalStreamError::CorruptFrame { .. })
        ));
    }

    #[test]
    fn checked_allocation_len_converts_valid_u32() {
        assert_eq!(
            checked_allocation_len("record_field", 42).expect("allocation len"),
            42
        );
    }
}
