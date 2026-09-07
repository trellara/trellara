use std::collections::BTreeSet;

use trellara_protocol::parse_lsn;

use crate::LakeEpochSummary;

pub(crate) fn validate_quarantine_entry_rows(epoch: &LakeEpochSummary) -> Result<(), String> {
    let mut seen_entries = BTreeSet::new();
    let quarantined_sources = epoch
        .source_watermarks
        .iter()
        .filter(|source| source.state == "quarantined")
        .map(|source| source.source_id.clone())
        .collect::<BTreeSet<_>>();
    for entry in &epoch.quarantine_entries {
        validate_clean_quarantine_field("source_id", &entry.source_id)?;
        if !quarantined_sources.contains(&entry.source_id) {
            return Err(format!(
                "quarantine entry source_id {} does not match a quarantined source watermark",
                entry.source_id
            ));
        }
        validate_optional_quarantine_field("transaction_id", entry.transaction_id.as_deref())?;
        validate_optional_commit_lsn(entry.commit_lsn.as_deref())?;
        validate_clean_quarantine_field("reason", &entry.reason)?;
        validate_clean_quarantine_field("details", &entry.details)?;
        validate_clean_quarantine_field("recovery_command", &entry.recovery_command)?;

        let identity = (
            entry.source_id.clone(),
            entry.transaction_id.clone(),
            entry.commit_lsn.clone(),
            entry.reason.clone(),
        );
        if !seen_entries.insert(identity) {
            return Err(format!(
                "quarantine entry has duplicate identity source_id {} transaction_id {} commit_lsn {} reason {}",
                entry.source_id,
                entry.transaction_id.as_deref().unwrap_or("<none>"),
                entry.commit_lsn.as_deref().unwrap_or("<none>"),
                entry.reason
            ));
        }
    }
    for source_id in quarantined_sources {
        if !epoch
            .quarantine_entries
            .iter()
            .any(|entry| entry.source_id == source_id)
        {
            return Err(format!(
                "quarantined source watermark {source_id} is missing quarantine entry evidence"
            ));
        }
    }
    Ok(())
}

fn validate_optional_commit_lsn(commit_lsn: Option<&str>) -> Result<(), String> {
    let Some(commit_lsn) = commit_lsn else {
        return Ok(());
    };
    validate_optional_quarantine_field("commit_lsn", Some(commit_lsn))?;
    let value = parse_lsn(commit_lsn)
        .map_err(|_| format!("quarantine entry has invalid commit_lsn {commit_lsn}"))?;
    if value == 0 {
        Err(format!(
            "quarantine entry has invalid commit_lsn {commit_lsn}"
        ))
    } else {
        Ok(())
    }
}

fn validate_optional_quarantine_field(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_clean_quarantine_field(field, value)
}

fn validate_clean_quarantine_field(field: &'static str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.trim() != value {
        Err(format!("quarantine entry has invalid {field} {value:?}"))
    } else {
        Ok(())
    }
}
