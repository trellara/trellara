use crate::index::IndexRead;
use crate::{LocalStreamError, Result};

pub(crate) const INDEX_ENTRY_BYTES: usize = 8;

pub(crate) fn parse_index_entries(bytes: &[u8], log_len: u64) -> IndexRead {
    if !bytes.len().is_multiple_of(INDEX_ENTRY_BYTES) {
        return IndexRead::Corrupt;
    }

    let mut positions = Vec::with_capacity(bytes.len() / INDEX_ENTRY_BYTES);
    let mut previous = None;
    for chunk in bytes.chunks_exact(INDEX_ENTRY_BYTES) {
        let mut position_bytes = [0; INDEX_ENTRY_BYTES];
        position_bytes.copy_from_slice(chunk);
        let position = u64::from_le_bytes(position_bytes);
        if position >= log_len || previous.is_some_and(|previous| position <= previous) {
            return IndexRead::Corrupt;
        }
        positions.push(position);
        previous = Some(position);
    }
    IndexRead::Valid(positions)
}

pub(crate) fn encode_index_entries(positions: &[u64]) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(index_entries_byte_len(positions.len())?);
    for position in positions {
        bytes.extend_from_slice(&position.to_le_bytes());
    }
    Ok(bytes)
}

fn index_entries_byte_len(position_count: usize) -> Result<usize> {
    position_count
        .checked_mul(INDEX_ENTRY_BYTES)
        .ok_or(LocalStreamError::FieldTooLarge {
            field: "index_entries",
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_entries_parse_monotonic_offsets() {
        assert_eq!(
            parse_index_entries(
                &encode_index_entries(&[4, 16, 32]).expect("encode index"),
                64
            ),
            IndexRead::Valid(vec![4, 16, 32])
        );
    }

    #[test]
    fn index_entries_reject_partial_entry_bytes() {
        assert_eq!(parse_index_entries(&[1, 2, 3], 64), IndexRead::Corrupt);
    }

    #[test]
    fn index_entries_reject_non_increasing_offsets() {
        assert_eq!(
            parse_index_entries(
                &encode_index_entries(&[4, 16, 16]).expect("encode index"),
                64
            ),
            IndexRead::Corrupt
        );
    }

    #[test]
    fn index_entries_reject_offsets_at_or_past_log_end() {
        assert_eq!(
            parse_index_entries(&encode_index_entries(&[4, 16]).expect("encode index"), 16),
            IndexRead::Corrupt
        );
    }

    #[test]
    fn index_entries_byte_len_rejects_capacity_overflow() {
        assert!(matches!(
            index_entries_byte_len(usize::MAX / INDEX_ENTRY_BYTES + 1),
            Err(LocalStreamError::FieldTooLarge {
                field: "index_entries"
            })
        ));
    }
}
