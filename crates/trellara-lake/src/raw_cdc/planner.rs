use std::collections::BTreeMap;

use trellara_protocol::TransactionEnvelope;

use super::dedup::RawCdcDedupLedger;
use super::planner_admission::{admit_envelope, RawCdcEnvelopeAdmission, RawCdcPlannerCounters};
use crate::raw_cdc_accumulator::{
    RawCdcFileAccumulator, RawCdcPartitionAccumulator, RawCdcSourceAccumulator,
    RawCdcTableAccumulator,
};
use crate::raw_cdc_types::*;
use crate::{LakeError, LakePlanConfig};

pub(super) struct RawCdcEpochPlanner<'a> {
    pub(super) writer_config: &'a LakeRawCdcWriterConfig,
    pub(super) plan_config: &'a LakePlanConfig,
    dedup: RawCdcDedupLedger,
    pub(super) counters: RawCdcPlannerCounters,
    pub(super) files: BTreeMap<(String, u32), RawCdcFileAccumulator>,
    pub(super) sources: BTreeMap<String, RawCdcSourceAccumulator>,
    pub(super) tables: BTreeMap<String, RawCdcTableAccumulator>,
    pub(super) partitions: BTreeMap<(String, u32), RawCdcPartitionAccumulator>,
    pub(super) rows: Vec<LakeRawCdcRowIntent>,
}

impl<'a> RawCdcEpochPlanner<'a> {
    pub(super) fn new(
        writer_config: &'a LakeRawCdcWriterConfig,
        plan_config: &'a LakePlanConfig,
    ) -> Result<Self, LakeError> {
        if writer_config.source_bucket_count == 0 {
            return Err(LakeError::InvalidSourceBucketCount);
        }

        Ok(Self {
            writer_config,
            plan_config,
            dedup: RawCdcDedupLedger::default(),
            counters: RawCdcPlannerCounters::default(),
            files: BTreeMap::new(),
            sources: BTreeMap::new(),
            tables: BTreeMap::new(),
            partitions: BTreeMap::new(),
            rows: Vec::new(),
        })
    }

    pub(super) fn plan(
        mut self,
        envelopes: &[TransactionEnvelope],
    ) -> Result<LakeRawCdcEpochWritePlan, LakeError> {
        for envelope in envelopes {
            self.apply_envelope(envelope)?;
        }
        self.into_write_plan()
    }

    fn apply_envelope(&mut self, envelope: &TransactionEnvelope) -> Result<(), LakeError> {
        if !self.source_in_scope(&envelope.source_id) {
            return Ok(());
        }
        let RawCdcEnvelopeAdmission::Accumulate { transaction_id } = admit_envelope(
            envelope,
            &self.writer_config.dataset_id,
            &mut self.dedup,
            &mut self.counters,
        )?
        else {
            return Ok(());
        };
        self.accumulate_source(envelope, &transaction_id)?;
        self.accumulate_files_and_tables(envelope, &transaction_id)
    }

    fn source_in_scope(&self, source_id: &str) -> bool {
        self.writer_config.required_sources.is_empty()
            || self.writer_config.required_sources.contains(source_id)
    }
}
