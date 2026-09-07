use trellara_stream::StreamMessage;

use crate::barrier_header_clean::validate_optional_clean_header_value;
use crate::barrier_header_lookup::optional_header;
use crate::ApplyWorkerResult;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ManifestEvidenceHeaders {
    pub(crate) manifest_checksum: String,
    pub(crate) manifest_event_count: String,
    pub(crate) envelope_event_count: String,
    pub(crate) event_count_coverage: String,
    pub(crate) participating_partition_ids: String,
    pub(crate) visibility_contract: String,
}

impl ManifestEvidenceHeaders {
    pub(crate) fn from_message(message: &StreamMessage) -> ApplyWorkerResult<Self> {
        Ok(Self {
            manifest_checksum: optional_evidence_header(
                message,
                "trellara.partitioned_scale_manifest_checksum",
            )?,
            manifest_event_count: optional_evidence_header(
                message,
                "trellara.partitioned_scale_manifest_event_count",
            )?,
            envelope_event_count: optional_evidence_header(
                message,
                "trellara.partitioned_scale_envelope_event_count",
            )?,
            event_count_coverage: optional_evidence_header(
                message,
                "trellara.partitioned_scale_event_count_coverage",
            )?,
            participating_partition_ids: optional_evidence_header(
                message,
                "trellara.partitioned_scale_participating_partition_ids",
            )?,
            visibility_contract: optional_evidence_header(
                message,
                "trellara.partitioned_scale_visibility_contract",
            )?,
        })
    }

    pub(crate) fn validate_clean(&self) -> ApplyWorkerResult<()> {
        validate_optional_clean_header_value(
            "trellara.partitioned_scale_manifest_checksum",
            &self.manifest_checksum,
        )?;
        validate_optional_clean_header_value(
            "trellara.partitioned_scale_manifest_event_count",
            &self.manifest_event_count,
        )?;
        validate_optional_clean_header_value(
            "trellara.partitioned_scale_envelope_event_count",
            &self.envelope_event_count,
        )?;
        validate_optional_clean_header_value(
            "trellara.partitioned_scale_event_count_coverage",
            &self.event_count_coverage,
        )?;
        validate_optional_clean_header_value(
            "trellara.partitioned_scale_participating_partition_ids",
            &self.participating_partition_ids,
        )?;
        validate_optional_clean_header_value(
            "trellara.partitioned_scale_visibility_contract",
            &self.visibility_contract,
        )
    }
}

fn optional_evidence_header(
    message: &StreamMessage,
    key: &'static str,
) -> ApplyWorkerResult<String> {
    Ok(optional_header(&message.headers, key)?.unwrap_or_default())
}
