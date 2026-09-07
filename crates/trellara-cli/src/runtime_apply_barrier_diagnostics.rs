pub(crate) fn barrier_pending_blockers(
    stats: &trellara_apply_postgres::BarrierPendingStats,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if stats.missing_manifest > 0 {
        blockers.push(format!(
            "{} transaction(s) missing manifest",
            stats.missing_manifest
        ));
    }
    if stats.missing_commit_marker > 0 {
        blockers.push(format!(
            "{} transaction(s) missing commit marker",
            stats.missing_commit_marker
        ));
    }
    if stats.invalid_commit_marker > 0 {
        blockers.push(format!(
            "{} transaction(s) have commit marker mismatches",
            stats.invalid_commit_marker
        ));
    }
    if stats.missing_chunks > 0 {
        blockers.push(format!(
            "{} partition chunk(s) missing",
            stats.missing_chunks
        ));
    }
    if stats.extra_chunks > 0 {
        blockers.push(format!(
            "{} partition chunk(s) are not listed in the manifest",
            stats.extra_chunks
        ));
    }
    blockers
}

pub(crate) fn barrier_pending_blocker_codes(
    stats: &trellara_apply_postgres::BarrierPendingStats,
) -> Vec<String> {
    let mut codes = Vec::new();
    if stats.missing_manifest > 0 {
        codes.push("missing_manifest".to_string());
    }
    if stats.missing_commit_marker > 0 {
        codes.push("missing_commit_marker".to_string());
    }
    if stats.invalid_commit_marker > 0 {
        codes.push("invalid_commit_marker".to_string());
    }
    if stats.missing_chunks > 0 {
        codes.push("missing_chunks".to_string());
    }
    if stats.extra_chunks > 0 {
        codes.push("extra_chunks".to_string());
    }
    codes
}

pub(crate) fn barrier_pending_recovery_actions(
    stats: &trellara_apply_postgres::BarrierPendingStats,
) -> Vec<String> {
    let mut actions = Vec::new();
    if stats.missing_manifest > 0 {
        actions.push(
            "replay manifest topic for pending barrier transactions before target apply"
                .to_string(),
        );
    }
    if stats.missing_commit_marker > 0 {
        actions.push(
            "replay commit-marker topic for pending barrier transactions before target apply"
                .to_string(),
        );
    }
    if stats.invalid_commit_marker > 0 {
        actions.push(
            "republish matching manifest and commit marker evidence before target apply"
                .to_string(),
        );
    }
    if stats.missing_chunks > 0 {
        actions.push(
            "replay partition chunk topics until every manifest partition is buffered".to_string(),
        );
    }
    if stats.extra_chunks > 0 {
        actions.push(
            "quarantine or rewind unlisted partition chunks, then replay from the manifest boundary before target apply"
                .to_string(),
        );
    }
    actions
}
