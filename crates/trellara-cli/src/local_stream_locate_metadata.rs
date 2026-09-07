use std::collections::BTreeSet;

use crate::LocalStreamLocateMatch;

pub(crate) fn metadata_conflicts(matches: &[LocalStreamLocateMatch]) -> Vec<String> {
    let mut conflicts = Vec::new();
    push_distinct_value_conflict(
        &mut conflicts,
        "source IDs",
        matches
            .iter()
            .filter_map(|matched| matched.source_id.clone()),
    );
    push_distinct_value_conflict(
        &mut conflicts,
        "dataset IDs",
        matches
            .iter()
            .filter_map(|matched| matched.dataset_id.clone()),
    );
    push_distinct_value_conflict(
        &mut conflicts,
        "commit LSNs",
        matches
            .iter()
            .filter_map(|matched| matched.commit_lsn.clone()),
    );

    let partition_counts = distinct_values(
        matches
            .iter()
            .filter(|matched| {
                matched.message_kind == "manifest" || matched.message_kind == "commit_marker"
            })
            .filter_map(|matched| matched.partition_count)
            .map(|count| count.to_string()),
    );
    if partition_counts.len() > 1 {
        conflicts.push(format!(
            "manifest and commit_marker partition counts disagree: {}",
            partition_counts.join(", ")
        ));
    }

    conflicts
}

fn push_distinct_value_conflict(
    conflicts: &mut Vec<String>,
    label: &'static str,
    values: impl Iterator<Item = String>,
) {
    let distinct = distinct_values(values);
    if distinct.len() > 1 {
        conflicts.push(format!(
            "boundary {label} disagree: {}",
            distinct.join(", ")
        ));
    }
}

fn distinct_values(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}
