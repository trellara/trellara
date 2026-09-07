use crate::{
    ddl_barrier_validation_fields::{validate_no_surrounding_whitespace, validate_non_empty},
    DdlBarrier, DdlBarrierAck, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DdlBarrierLookup {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
}

impl DdlBarrierLookup {
    pub fn new(
        source_id: impl Into<String>,
        database_id: impl Into<String>,
        dataset_id: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            database_id: database_id.into(),
            dataset_id: dataset_id.into(),
        }
    }

    pub fn from_barrier(barrier: &DdlBarrier) -> Self {
        Self::new(
            &barrier.source_id,
            &barrier.database_id,
            &barrier.dataset_id,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct DdlBarrierKey {
    source_id: String,
    database_id: String,
    dataset_id: String,
    barrier_id: String,
}

impl DdlBarrierKey {
    fn new(
        source_id: impl Into<String>,
        database_id: impl Into<String>,
        dataset_id: impl Into<String>,
        barrier_id: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            database_id: database_id.into(),
            dataset_id: dataset_id.into(),
            barrier_id: barrier_id.into(),
        }
    }

    pub(crate) fn from_barrier(barrier: &DdlBarrier) -> Self {
        Self::new(
            &barrier.source_id,
            &barrier.database_id,
            &barrier.dataset_id,
            &barrier.barrier_id,
        )
    }

    pub(crate) fn from_lookup(lookup: &DdlBarrierLookup, barrier_id: &str) -> Self {
        Self::new(
            &lookup.source_id,
            &lookup.database_id,
            &lookup.dataset_id,
            barrier_id,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct DdlBarrierAckKey {
    source_id: String,
    database_id: String,
    dataset_id: String,
    barrier_id: String,
    sink: String,
}

impl DdlBarrierAckKey {
    pub(crate) fn from_ack(ack: &DdlBarrierAck) -> Self {
        Self {
            source_id: ack.source_id.clone(),
            database_id: ack.database_id.clone(),
            dataset_id: ack.dataset_id.clone(),
            barrier_id: ack.barrier_id.clone(),
            sink: ack.sink.clone(),
        }
    }
}

pub(crate) fn validate_summary_lookup_identity(
    lookup: &DdlBarrierLookup,
    barrier_id: &str,
) -> Result<()> {
    validate_lookup_field("DDL barrier lookup", "source_id", &lookup.source_id)?;
    validate_lookup_field("DDL barrier lookup", "database_id", &lookup.database_id)?;
    validate_lookup_field("DDL barrier lookup", "dataset_id", &lookup.dataset_id)?;
    validate_lookup_field("DDL barrier lookup", "barrier_id", barrier_id)
}

fn validate_lookup_field(context: &str, field: &'static str, value: &str) -> Result<()> {
    validate_non_empty(context, field, value)?;
    validate_no_surrounding_whitespace(context, field, value)
}
