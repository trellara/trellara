use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ConsistencyContractSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) mode: String,
    pub(crate) selected_consumer_mode: String,
    pub(crate) stream_kind: String,
    pub(crate) topics: Vec<String>,
    pub(crate) source_capture_contract: String,
    pub(crate) transaction_boundary_contract: String,
    pub(crate) source_ack_contract: String,
    pub(crate) snapshot_handoff_contract: String,
    pub(crate) transport_durability_contract: String,
    pub(crate) consumer_visibility_contract: String,
    pub(crate) target_checkpoint_contract: String,
    pub(crate) replay_contract: String,
    pub(crate) reseed_contract: String,
    pub(crate) partition_contract: Option<ConsistencyPartitionContract>,
    pub(crate) lake_visibility_contract: String,
    pub(crate) invariants: Vec<ConsistencyInvariant>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ConsistencyPartitionContract {
    pub(crate) key_column: String,
    pub(crate) partition_count: u32,
    pub(crate) null_key_policy: String,
    pub(crate) key_change_policy: String,
    pub(crate) manifest_topic: String,
    pub(crate) commit_topic: String,
    pub(crate) partition_topic_pattern: String,
    pub(crate) global_visibility_rule: String,
    pub(crate) partition_local_visibility_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ConsistencyInvariant {
    pub(crate) code: String,
    pub(crate) rule: String,
    pub(crate) proof_command: String,
}
