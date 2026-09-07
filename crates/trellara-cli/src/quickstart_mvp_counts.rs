use crate::{
    count_chaos_scenarios, ChaosRunSummary, MVP_PARTITIONED_SCALE_SCENARIOS,
    MVP_SCHEMA_CHANGE_SCENARIOS, MVP_SNAPSHOT_SCENARIOS, MVP_SOURCE_FAILOVER_SCENARIOS,
    MVP_STRICT_CHUNK_SCENARIOS,
};

pub(crate) struct MvpProofCounts {
    pub(crate) snapshot: usize,
    pub(crate) strict_chunk: usize,
    pub(crate) partitioned_scale: usize,
    pub(crate) source_failover: usize,
    pub(crate) schema_change: usize,
}

impl MvpProofCounts {
    pub(crate) fn from_chaos(chaos: &ChaosRunSummary) -> Self {
        Self {
            snapshot: count_chaos_scenarios(chaos, MVP_SNAPSHOT_SCENARIOS),
            strict_chunk: count_chaos_scenarios(chaos, MVP_STRICT_CHUNK_SCENARIOS),
            partitioned_scale: count_chaos_scenarios(chaos, MVP_PARTITIONED_SCALE_SCENARIOS),
            source_failover: count_chaos_scenarios(chaos, MVP_SOURCE_FAILOVER_SCENARIOS),
            schema_change: count_chaos_scenarios(chaos, MVP_SCHEMA_CHANGE_SCENARIOS),
        }
    }
}
