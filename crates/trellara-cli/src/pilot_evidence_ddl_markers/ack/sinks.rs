use std::collections::BTreeSet;

use serde_json::Value;

pub(super) fn sink_is_supported(sink: &str) -> bool {
    matches!(
        sink,
        "target_postgres" | "raw_cdc_lake" | "spark_derived_views" | "partition_visibility"
    )
}

pub(super) fn required_sink_set(proof: &Value) -> Option<BTreeSet<String>> {
    let required_sinks = proof.get("required_sinks").and_then(Value::as_array)?;
    let mut sinks = BTreeSet::new();

    for sink in required_sinks {
        let sink = sink.as_str()?.trim();
        if sink.is_empty() || !sink_is_supported(sink) || !sinks.insert(sink.to_string()) {
            return None;
        }
    }

    (!sinks.is_empty()).then_some(sinks)
}
