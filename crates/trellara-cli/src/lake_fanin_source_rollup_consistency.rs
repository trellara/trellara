use crate::{lake_epoch_watermark_rollup, LakeEpochSummary};

pub(super) fn validate_source_watermark_rollup(epoch: &LakeEpochSummary) -> Result<(), String> {
    let rollup = lake_epoch_watermark_rollup(&epoch.source_watermarks);
    if rollup.invalid_lsn_source_count > 0 {
        return Err(format!(
            "source watermarks contain invalid LSNs: {}",
            rollup.invalid_lsn_sources.join(",")
        ));
    }
    if rollup.complete_source_count != epoch.complete_source_count
        || rollup.lagging_source_count + rollup.missing_source_count != epoch.missing_source_count
        || rollup.quarantined_source_count != epoch.quarantined_source_count
    {
        return Err(format!(
            "source watermark rollup inconsistent: epoch complete={} missing={} quarantined={}, source_rows complete={} lagging={} missing={} quarantined={}",
            epoch.complete_source_count,
            epoch.missing_source_count,
            epoch.quarantined_source_count,
            rollup.complete_source_count,
            rollup.lagging_source_count,
            rollup.missing_source_count,
            rollup.quarantined_source_count
        ));
    }
    if rollup != epoch.watermark_rollup {
        return Err("stored watermark_rollup does not match source_watermarks".to_string());
    }
    Ok(())
}

pub(super) fn validate_source_watermark_cardinality(
    epoch: &LakeEpochSummary,
) -> Result<(), String> {
    if epoch.source_watermarks.len() == epoch.required_source_count {
        Ok(())
    } else {
        Err(format!(
            "source watermark cardinality inconsistent: required={} source_rows={}",
            epoch.required_source_count,
            epoch.source_watermarks.len()
        ))
    }
}

pub(super) fn validate_source_count_rollup(epoch: &LakeEpochSummary) -> Result<(), String> {
    let observed_sources = epoch
        .complete_source_count
        .saturating_add(epoch.missing_source_count)
        .saturating_add(epoch.quarantined_source_count);
    if observed_sources == epoch.required_source_count {
        Ok(())
    } else {
        Err(format!(
            "source counts inconsistent: required={} complete={} missing={} quarantined={}",
            epoch.required_source_count,
            epoch.complete_source_count,
            epoch.missing_source_count,
            epoch.quarantined_source_count
        ))
    }
}
