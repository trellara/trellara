use std::collections::BTreeSet;

use crate::epoch_lsn_window::advance_lsn_window;
use crate::{LakeEpochSourceState, LakeError};

#[derive(Clone, Debug)]
pub(crate) struct SourceAccumulator {
    pub(crate) state: LakeEpochSourceState,
    pub(crate) start_lsn: Option<String>,
    pub(crate) end_lsn: Option<String>,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
    pub(crate) gap_reason: Option<String>,
}

impl Default for SourceAccumulator {
    fn default() -> Self {
        Self {
            state: LakeEpochSourceState::Complete,
            start_lsn: None,
            end_lsn: None,
            transaction_count: 0,
            change_count: 0,
            checksum_rollup: 0,
            gap_reason: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TableAccumulator {
    pub(crate) transactions: BTreeSet<String>,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PartitionAccumulator {
    pub(crate) first_commit_lsn: Option<String>,
    pub(crate) last_commit_lsn: Option<String>,
    pub(crate) transaction_count: usize,
    pub(crate) event_count: usize,
    pub(crate) checksum_rollup: u64,
}

pub(crate) fn advance_source_lsn(
    source: &mut SourceAccumulator,
    commit_lsn: &str,
) -> Result<(), LakeError> {
    advance_lsn_window(&mut source.start_lsn, &mut source.end_lsn, commit_lsn)
}

pub(crate) fn advance_partition_lsn(
    partition: &mut PartitionAccumulator,
    commit_lsn: &str,
) -> Result<(), LakeError> {
    advance_lsn_window(
        &mut partition.first_commit_lsn,
        &mut partition.last_commit_lsn,
        commit_lsn,
    )
}

pub(crate) fn checked_epoch_count_add(
    current: usize,
    delta: usize,
    field: &'static str,
) -> Result<usize, LakeError> {
    current
        .checked_add(delta)
        .ok_or(LakeError::EpochCountOverflow { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_count_add_returns_sum() {
        assert_eq!(
            checked_epoch_count_add(40, 2, "change_count").expect("count"),
            42
        );
    }

    #[test]
    fn epoch_count_add_rejects_overflow() {
        let error = checked_epoch_count_add(usize::MAX, 1, "change_count").expect_err("overflow");

        assert!(matches!(
            error,
            LakeError::EpochCountOverflow {
                field: "change_count"
            }
        ));
    }

    #[test]
    fn source_lsn_advance_rejects_zero_commit_lsn() {
        let mut source = SourceAccumulator::default();

        let error = advance_source_lsn(&mut source, "0/0").expect_err("zero lsn");

        assert!(matches!(error, LakeError::InvalidCommitLsn { commit_lsn } if commit_lsn == "0/0"));
        assert_eq!(source.start_lsn, None);
        assert_eq!(source.end_lsn, None);
    }

    #[test]
    fn partition_lsn_advance_tracks_commit_window() {
        let mut partition = PartitionAccumulator::default();

        advance_partition_lsn(&mut partition, "0/16B6D00").expect("first lsn");
        advance_partition_lsn(&mut partition, "0/16B6C50").expect("earlier lsn");
        advance_partition_lsn(&mut partition, "0/16B6F00").expect("later lsn");

        assert_eq!(partition.first_commit_lsn.as_deref(), Some("0/16B6C50"));
        assert_eq!(partition.last_commit_lsn.as_deref(), Some("0/16B6F00"));
    }
}
