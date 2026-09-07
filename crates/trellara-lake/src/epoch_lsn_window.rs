use trellara_protocol::parse_lsn;

use crate::LakeError;

pub(crate) fn advance_lsn_window(
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
    if should_update_start(start_lsn.as_deref(), commit_lsn_value)? {
        *start_lsn = Some(commit_lsn.to_string());
    }
    if should_update_end(end_lsn.as_deref(), commit_lsn_value)? {
        *end_lsn = Some(commit_lsn.to_string());
    }
    Ok(())
}

fn should_update_start(
    current_lsn: Option<&str>,
    commit_lsn_value: u64,
) -> Result<bool, LakeError> {
    Ok(current_lsn
        .map(parse_lsn)
        .transpose()?
        .is_none_or(|current| commit_lsn_value < current))
}

fn should_update_end(current_lsn: Option<&str>, commit_lsn_value: u64) -> Result<bool, LakeError> {
    Ok(current_lsn
        .map(parse_lsn)
        .transpose()?
        .is_none_or(|current| commit_lsn_value > current))
}
