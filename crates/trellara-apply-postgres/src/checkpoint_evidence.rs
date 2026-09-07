use trellara_protocol::{parse_lsn, TransactionEnvelope};

use crate::{checkpoint_manifest_evidence::validate_checkpoint_manifest, ApplyError, Result};

pub(crate) fn validate_apply_checkpoint_evidence(envelope: &TransactionEnvelope) -> Result<()> {
    if envelope.source_id.trim().is_empty() {
        return Err(ApplyError::MissingCheckpointEvidence { field: "source_id" });
    }
    if envelope.dataset_id.trim().is_empty() {
        return Err(ApplyError::MissingCheckpointEvidence {
            field: "dataset_id",
        });
    }
    validate_non_zero_lsn("commit_lsn", &envelope.commit_lsn)?;
    envelope.validate()?;

    let Some(manifest) = &envelope.manifest else {
        return Ok(());
    };

    validate_checkpoint_manifest(envelope, manifest)?;
    validate_non_zero_lsn("manifest.source_commit_lsn", &manifest.source_commit_lsn)?;

    Ok(())
}

fn validate_non_zero_lsn(field: &'static str, lsn: &str) -> Result<()> {
    if parse_lsn(lsn)? == 0 {
        return Err(ApplyError::MissingCheckpointEvidence { field });
    }
    Ok(())
}
