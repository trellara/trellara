use crate::{
    local_stream_locate_metadata::metadata_conflicts, LocalStreamLocateBoundarySummary,
    LocalStreamLocateMatch, TrellaraConfig,
};

#[path = "local_stream_locate_boundary/rules.rs"]
mod rules;

impl LocalStreamLocateBoundarySummary {
    pub(crate) fn from_matches(
        config: &TrellaraConfig,
        matches: &[LocalStreamLocateMatch],
    ) -> Self {
        let mode = config.status_mode();
        let required_message_kinds = rules::required_message_kinds(&mode);
        let found_message_kinds = rules::found_message_kinds(matches);
        let missing_message_kinds =
            rules::missing_message_kinds(&required_message_kinds, &found_message_kinds);
        let metadata_conflicts = metadata_conflicts(matches);
        let participating_partition_count = rules::participating_partition_count(matches);
        let found_partition_ids = rules::found_partition_ids(matches);
        let missing_partition_ids =
            rules::missing_partition_ids(participating_partition_count, &found_partition_ids);
        let found_partition_count = found_partition_ids.len();
        let missing_partition_count = participating_partition_count.map(|expected| {
            if missing_partition_ids.is_empty() {
                expected.saturating_sub(found_partition_count)
            } else {
                missing_partition_ids.len()
            }
        });
        let partition_complete = if mode == "strict_transaction_order" {
            true
        } else {
            participating_partition_count.is_some()
                && missing_partition_ids.is_empty()
                && found_partition_count > 0
        };
        let complete =
            missing_message_kinds.is_empty() && metadata_conflicts.is_empty() && partition_complete;
        let status = if complete {
            "complete"
        } else if matches.is_empty() {
            "not_found"
        } else {
            "incomplete_boundary"
        }
        .to_string();

        Self {
            mode,
            complete,
            status,
            required_message_kinds,
            found_message_kinds,
            missing_message_kinds,
            metadata_conflicts,
            participating_partition_count,
            found_partition_ids,
            missing_partition_ids,
            found_partition_count,
            missing_partition_count,
        }
    }
}
