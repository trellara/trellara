use std::collections::BTreeSet;

use trellara_protocol::parse_lsn;

use crate::LakeError;

#[derive(Clone, Debug, Default)]
pub(crate) struct RawCdcFileAccumulator {
    pub(crate) table_name: String,
    pub(crate) relation: String,
    pub(crate) source_bucket: u32,
    pub(crate) source_ids: BTreeSet<String>,
    pub(crate) transactions: BTreeSet<String>,
    pub(crate) change_count: usize,
    pub(crate) min_commit_lsn: Option<String>,
    pub(crate) max_commit_lsn: Option<String>,
    pub(crate) checksum_rollup: u64,
    pub(crate) idempotency_keys: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RawCdcSourceAccumulator {
    pub(crate) start_lsn: Option<String>,
    pub(crate) end_lsn: Option<String>,
    pub(crate) transactions: BTreeSet<String>,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RawCdcTableAccumulator {
    pub(crate) transactions: BTreeSet<String>,
    pub(crate) change_count: usize,
    pub(crate) checksum_rollup: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RawCdcPartitionAccumulator {
    pub(crate) first_commit_lsn: Option<String>,
    pub(crate) last_commit_lsn: Option<String>,
    pub(crate) transactions: BTreeSet<String>,
    pub(crate) event_count: usize,
    pub(crate) checksum_rollup: u64,
}

pub(crate) fn advance_raw_cdc_lsn(
    start_lsn: &mut Option<String>,
    end_lsn: &mut Option<String>,
    commit_lsn: &str,
) -> Result<(), LakeError> {
    let commit_lsn_value = parse_lsn(commit_lsn)?;
    if commit_lsn_value == 0 {
        return Err(LakeError::InvalidCommitLsn {
            commit_lsn: commit_lsn.to_string(),
        });
    }
    let should_update_start = start_lsn
        .as_deref()
        .map(parse_lsn)
        .transpose()?
        .is_none_or(|current| commit_lsn_value < current);
    if should_update_start {
        *start_lsn = Some(commit_lsn.to_string());
    }
    let should_update_end = end_lsn
        .as_deref()
        .map(parse_lsn)
        .transpose()?
        .is_none_or(|current| commit_lsn_value > current);
    if should_update_end {
        *end_lsn = Some(commit_lsn.to_string());
    }
    Ok(())
}

pub(crate) fn checked_raw_cdc_count_add(
    current: usize,
    delta: usize,
    field: &'static str,
) -> Result<usize, LakeError> {
    current
        .checked_add(delta)
        .ok_or(LakeError::RawCdcCountOverflow { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_cdc_count_add_returns_sum() {
        assert_eq!(
            checked_raw_cdc_count_add(40, 2, "change_count").expect("count"),
            42
        );
    }

    #[test]
    fn raw_cdc_count_add_rejects_overflow() {
        let error = checked_raw_cdc_count_add(usize::MAX, 1, "change_count").expect_err("overflow");

        assert!(matches!(
            error,
            LakeError::RawCdcCountOverflow {
                field: "change_count"
            }
        ));
    }

    #[test]
    fn raw_cdc_lsn_advance_rejects_zero_commit_lsn() {
        let mut start_lsn = None;
        let mut end_lsn = None;

        let error = advance_raw_cdc_lsn(&mut start_lsn, &mut end_lsn, "0/0").expect_err("zero lsn");

        assert!(matches!(error, LakeError::InvalidCommitLsn { commit_lsn } if commit_lsn == "0/0"));
        assert_eq!(start_lsn, None);
        assert_eq!(end_lsn, None);
    }
}
