use super::*;

#[test]
fn snapshot_run_lookup_rejects_malformed_identity() {
    for (flow, run_id) in [
        (FlowKey::new("", "sales"), "snapshot-1"),
        (FlowKey::new(" source ", "sales"), "snapshot-1"),
        (FlowKey::new("source", " sales "), "snapshot-1"),
        (FlowKey::new("source", "sales"), ""),
        (FlowKey::new("source", "sales"), " snapshot-1 "),
    ] {
        let error = snapshot_lookup_validation::validate_snapshot_run_lookup(&flow, run_id)
            .expect_err("malformed snapshot run lookup");

        assert!(
            error.to_string().contains("snapshot run lookup")
                || error.to_string().contains("flow key")
        );
    }
}

#[test]
fn snapshot_table_progress_lookup_rejects_malformed_identity() {
    for (flow, run_id, relation) in [
        (FlowKey::new("", "sales"), "snapshot-1", "public.sales"),
        (FlowKey::new("source", "sales"), "", "public.sales"),
        (FlowKey::new("source", "sales"), "snapshot-1", ""),
        (
            FlowKey::new("source", "sales"),
            " snapshot-1 ",
            "public.sales",
        ),
        (
            FlowKey::new("source", "sales"),
            "snapshot-1",
            " public.sales ",
        ),
    ] {
        let error = snapshot_lookup_validation::validate_snapshot_table_progress_lookup(
            &flow, run_id, relation,
        )
        .expect_err("malformed table progress lookup");

        assert!(
            error.to_string().contains("snapshot table progress lookup")
                || error.to_string().contains("flow key")
        );
    }
}
