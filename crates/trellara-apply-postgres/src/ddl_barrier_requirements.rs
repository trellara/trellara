const TARGET_POSTGRES_SINK: &str = "target_postgres";
const PARTITION_VISIBILITY_SINK: &str = "partition_visibility";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlBarrierRequirements {
    pub required_sinks: Vec<String>,
    pub requires_global_partition_pause: bool,
}

impl TargetDdlBarrierRequirements {
    pub fn target_postgres_only() -> Self {
        Self::from_required_sinks([TARGET_POSTGRES_SINK], false)
    }

    pub fn partitioned_scale(required_sinks: Vec<String>) -> Self {
        Self::from_required_sinks(required_sinks, true)
    }

    pub fn from_required_sinks(
        required_sinks: impl IntoIterator<Item = impl Into<String>>,
        requires_global_partition_pause: bool,
    ) -> Self {
        let mut required_sinks = normalized_required_sinks(required_sinks);
        ensure_sink(&mut required_sinks, TARGET_POSTGRES_SINK);
        if requires_global_partition_pause {
            ensure_sink(&mut required_sinks, PARTITION_VISIBILITY_SINK);
        }
        Self {
            required_sinks,
            requires_global_partition_pause,
        }
    }
}

fn normalized_required_sinks(
    required_sinks: impl IntoIterator<Item = impl Into<String>>,
) -> Vec<String> {
    let mut normalized = Vec::new();
    for sink in required_sinks {
        let sink = sink.into();
        let sink = sink.trim().to_string();
        if !sink.is_empty() && !normalized.contains(&sink) {
            normalized.push(sink);
        }
    }
    normalized
}

fn ensure_sink(required_sinks: &mut Vec<String>, sink: &str) {
    if !required_sinks.iter().any(|required| required == sink) {
        required_sinks.push(sink.to_string());
    }
}
