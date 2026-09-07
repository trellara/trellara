use trellara_protocol::{PartitionPlanConfig, StrictChunkPlanConfig};
use trellara_relay::RelayMode;

use crate::{DatasetMode, Result, TrellaraConfig};

impl TrellaraConfig {
    pub fn to_relay_mode(&self) -> Result<RelayMode> {
        self.validate()?;
        match self.dataset.mode {
            DatasetMode::StrictTransactionOrder => {
                if let Some(strict_chunking) = &self.dataset.strict_chunking {
                    Ok(RelayMode::StrictChunked(StrictChunkPlanConfig {
                        max_changes_per_chunk: strict_chunking.max_changes_per_chunk,
                    }))
                } else {
                    Ok(RelayMode::Strict)
                }
            }
            DatasetMode::PartitionedScaleMode => {
                let partition = self
                    .dataset
                    .partition
                    .as_ref()
                    .expect("validated partition config");
                Ok(RelayMode::Partitioned(PartitionPlanConfig {
                    partition_count: partition.partition_count,
                    key_column: partition.key_column.clone(),
                    null_key_policy: partition.null_key_policy.to_protocol_policy(),
                    key_change_policy: partition.key_change_policy.to_protocol_policy(),
                }))
            }
        }
    }

    pub(crate) fn uses_barrier_apply(&self) -> bool {
        self.dataset.mode == DatasetMode::PartitionedScaleMode
            || self.dataset.strict_chunking.is_some()
    }

    pub(crate) fn status_mode(&self) -> String {
        if self.dataset.mode == DatasetMode::StrictTransactionOrder
            && self.dataset.strict_chunking.is_some()
        {
            "strict_chunked_transaction_order".to_string()
        } else {
            self.dataset.mode.to_string()
        }
    }
}
