use serde::Serialize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChecksumStatus {
    Unknown,
    Match,
    Mismatch,
}

impl ChecksumStatus {
    pub(crate) fn from_validation(converged: Option<bool>) -> Self {
        match converged {
            Some(true) => Self::Match,
            Some(false) => Self::Mismatch,
            None => Self::Unknown,
        }
    }

    pub(crate) fn from_checksums(
        source_checksum: u64,
        target_checksum: u64,
        source_row_count: usize,
        target_row_count: usize,
    ) -> Self {
        if source_checksum == target_checksum && source_row_count == target_row_count {
            Self::Match
        } else {
            Self::Mismatch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_status_tracks_validation_convergence() {
        assert_eq!(
            ChecksumStatus::from_validation(Some(true)),
            ChecksumStatus::Match
        );
        assert_eq!(
            ChecksumStatus::from_validation(Some(false)),
            ChecksumStatus::Mismatch
        );
        assert_eq!(
            ChecksumStatus::from_validation(None),
            ChecksumStatus::Unknown
        );
    }

    #[test]
    fn checksum_status_requires_hash_and_row_counts_to_match() {
        assert_eq!(
            ChecksumStatus::from_checksums(42, 42, 2, 2),
            ChecksumStatus::Match
        );
        assert_eq!(
            ChecksumStatus::from_checksums(42, 7, 2, 2),
            ChecksumStatus::Mismatch
        );
        assert_eq!(
            ChecksumStatus::from_checksums(42, 42, 2, 3),
            ChecksumStatus::Mismatch
        );
    }
}
