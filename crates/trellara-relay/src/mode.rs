use trellara_protocol::{PartitionPlanConfig, StrictChunkPlanConfig};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelayMode {
    Strict,
    StrictChunked(StrictChunkPlanConfig),
    Partitioned(PartitionPlanConfig),
}
