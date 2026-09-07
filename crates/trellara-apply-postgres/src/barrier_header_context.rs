use trellara_stream::StreamMessage;

use crate::barrier_header_clean::{
    validate_optional_clean_header_value, validate_required_clean_header_value,
};
use crate::barrier_header_lookup::{optional_header, required_message_header};
use crate::barrier_manifest_evidence_headers::ManifestEvidenceHeaders;
use crate::ApplyWorkerResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderContext {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) database_id: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) partitioned_scale_decision: String,
    pub(crate) partition_parallel_safe: String,
    pub(crate) requires_ddl_barrier: String,
    pub(crate) dml_replay_after_ddl_barrier_required: String,
    pub(crate) partitioned_scale_reason: String,
    pub(crate) partition_id: String,
    pub(crate) partition_event_count: String,
    pub(crate) partition_checksum: String,
    pub(crate) global_event_count: String,
    pub(crate) partition_count: String,
    pub(crate) manifest_checksum: String,
    pub(crate) manifest_evidence: ManifestEvidenceHeaders,
}

impl HeaderContext {
    pub(crate) fn from_message(message: &StreamMessage) -> ApplyWorkerResult<Self> {
        let context = Self {
            source_id: required_message_header(message, "trellara.source_id")?,
            dataset_id: required_message_header(message, "trellara.dataset_id")?,
            database_id: required_message_header(message, "trellara.database_id")?,
            transaction_id: required_message_header(message, "trellara.transaction_id")?,
            commit_lsn: required_message_header(message, "trellara.commit_lsn")?,
            partitioned_scale_decision: required_message_header(
                message,
                "trellara.partitioned_scale_decision",
            )?,
            partition_parallel_safe: required_message_header(
                message,
                "trellara.partition_parallel_safe",
            )?,
            requires_ddl_barrier: required_message_header(
                message,
                "trellara.requires_ddl_barrier",
            )?,
            dml_replay_after_ddl_barrier_required: required_message_header(
                message,
                "trellara.dml_replay_after_ddl_barrier_required",
            )?,
            partitioned_scale_reason: required_message_header(
                message,
                "trellara.partitioned_scale_reason",
            )?,
            partition_id: optional_header(&message.headers, "trellara.partition_id")?
                .unwrap_or_default(),
            partition_event_count: optional_header(
                &message.headers,
                "trellara.partition_event_count",
            )?
            .unwrap_or_default(),
            partition_checksum: optional_header(&message.headers, "trellara.partition_checksum")?
                .unwrap_or_default(),
            global_event_count: optional_header(&message.headers, "trellara.global_event_count")?
                .unwrap_or_default(),
            partition_count: optional_header(&message.headers, "trellara.partition_count")?
                .unwrap_or_default(),
            manifest_checksum: optional_header(&message.headers, "trellara.manifest_checksum")?
                .unwrap_or_default(),
            manifest_evidence: ManifestEvidenceHeaders::from_message(message)?,
        };
        context.validate_clean_identity()?;
        Ok(context)
    }

    fn validate_clean_identity(&self) -> ApplyWorkerResult<()> {
        validate_required_clean_header_value("trellara.source_id", &self.source_id)?;
        validate_required_clean_header_value("trellara.dataset_id", &self.dataset_id)?;
        validate_required_clean_header_value("trellara.database_id", &self.database_id)?;
        validate_required_clean_header_value("trellara.transaction_id", &self.transaction_id)?;
        validate_required_clean_header_value("trellara.commit_lsn", &self.commit_lsn)?;
        validate_required_clean_header_value(
            "trellara.partitioned_scale_decision",
            &self.partitioned_scale_decision,
        )?;
        validate_required_clean_header_value(
            "trellara.partition_parallel_safe",
            &self.partition_parallel_safe,
        )?;
        validate_required_clean_header_value(
            "trellara.requires_ddl_barrier",
            &self.requires_ddl_barrier,
        )?;
        validate_required_clean_header_value(
            "trellara.dml_replay_after_ddl_barrier_required",
            &self.dml_replay_after_ddl_barrier_required,
        )?;
        validate_required_clean_header_value(
            "trellara.partitioned_scale_reason",
            &self.partitioned_scale_reason,
        )?;
        validate_optional_clean_header_value("trellara.partition_id", &self.partition_id)?;
        validate_optional_clean_header_value(
            "trellara.partition_event_count",
            &self.partition_event_count,
        )?;
        validate_optional_clean_header_value(
            "trellara.partition_checksum",
            &self.partition_checksum,
        )?;
        validate_optional_clean_header_value(
            "trellara.global_event_count",
            &self.global_event_count,
        )?;
        validate_optional_clean_header_value("trellara.partition_count", &self.partition_count)?;
        validate_optional_clean_header_value(
            "trellara.manifest_checksum",
            &self.manifest_checksum,
        )?;
        self.manifest_evidence.validate_clean()
    }

    pub(crate) fn transaction_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.source_id, self.dataset_id, self.commit_lsn, self.transaction_id
        )
    }
}
