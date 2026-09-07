#[path = "ack/command.rs"]
mod command;
#[path = "ack/evidence.rs"]
mod evidence;
#[path = "ack/sink_evidence.rs"]
mod sink_evidence;
#[path = "ack/sinks.rs"]
mod sinks;

use serde_json::Value;

pub(super) fn command_is_executable(item: &Value) -> bool {
    command::is_executable(item)
}

pub(super) fn command_collection_is_valid(items: &[Value], proof: &Value) -> bool {
    command::collection_is_valid(items, proof)
}

pub(super) fn evidence_item_is_valid(item: &Value, proof: &Value) -> bool {
    evidence::item_is_valid(item, proof)
}

pub(super) fn evidence_covers_required_sinks(items: &[Value], proof: &Value) -> bool {
    evidence::covers_required_sinks(items, proof)
}
