use super::*;
use std::collections::HashSet;

#[test]
fn strict_chunk_suite_covers_every_required_failure_point() {
    let reports = run_default_strict_chunk_suite(42);
    let covered = reports
        .iter()
        .map(|report| report.failure_point)
        .collect::<HashSet<_>>();

    assert_eq!(reports.len(), StrictChunkFailurePoint::ALL.len());
    assert!(StrictChunkFailurePoint::ALL
        .into_iter()
        .all(|failure_point| covered.contains(&failure_point)));
    assert!(reports.iter().all(|report| report.passed));
}
