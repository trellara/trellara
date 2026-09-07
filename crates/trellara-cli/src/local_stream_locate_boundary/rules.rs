use std::collections::BTreeSet;

use crate::LocalStreamLocateMatch;

pub(super) fn required_message_kinds(mode: &str) -> Vec<String> {
    match mode {
        "strict_transaction_order" => vec!["strict_transaction".to_string()],
        "strict_chunked_transaction_order" => vec![
            "manifest".to_string(),
            "commit_marker".to_string(),
            "strict_chunk".to_string(),
        ],
        _ => vec![
            "manifest".to_string(),
            "commit_marker".to_string(),
            "partition_chunk".to_string(),
        ],
    }
}

pub(super) fn found_message_kinds(matches: &[LocalStreamLocateMatch]) -> Vec<String> {
    matches
        .iter()
        .map(|matched| matched.message_kind.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn missing_message_kinds(required: &[String], found: &[String]) -> Vec<String> {
    required
        .iter()
        .filter(|kind| !found.iter().any(|candidate| candidate == *kind))
        .cloned()
        .collect()
}

pub(super) fn participating_partition_count(matches: &[LocalStreamLocateMatch]) -> Option<usize> {
    matches
        .iter()
        .filter(|matched| {
            matched.message_kind == "manifest" || matched.message_kind == "commit_marker"
        })
        .filter_map(|matched| matched.partition_count)
        .max()
}

pub(super) fn found_partition_ids(matches: &[LocalStreamLocateMatch]) -> Vec<u32> {
    matches
        .iter()
        .filter(|matched| {
            matched.message_kind == "partition_chunk" || matched.message_kind == "strict_chunk"
        })
        .filter_map(|matched| matched.partition_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn missing_partition_ids(
    participating_partition_count: Option<usize>,
    found_partition_ids: &[u32],
) -> Vec<u32> {
    let Some(expected) = participating_partition_count else {
        return Vec::new();
    };
    (0..expected)
        .filter_map(|partition_id| u32::try_from(partition_id).ok())
        .filter(|partition_id| !found_partition_ids.contains(partition_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_message_kinds_match_boundary_modes() {
        assert_eq!(
            required_message_kinds("strict_transaction_order"),
            vec!["strict_transaction"]
        );
        assert_eq!(
            required_message_kinds("strict_chunked_transaction_order"),
            vec!["manifest", "commit_marker", "strict_chunk"]
        );
        assert_eq!(
            required_message_kinds("partitioned_scale_mode"),
            vec!["manifest", "commit_marker", "partition_chunk"]
        );
    }

    #[test]
    fn missing_message_kinds_preserve_required_order() {
        let required = required_message_kinds("partitioned_scale_mode");
        let found = vec!["commit_marker".to_string()];

        assert_eq!(
            missing_message_kinds(&required, &found),
            vec!["manifest".to_string(), "partition_chunk".to_string()]
        );
    }

    #[test]
    fn missing_partition_ids_name_exact_absent_partitions() {
        assert_eq!(missing_partition_ids(Some(4), &[0, 2]), vec![1, 3]);
    }
}
