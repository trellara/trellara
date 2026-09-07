use super::*;

#[test]
fn ddl_barrier_summary_lookup_rejects_malformed_identity() {
    for (flow, barrier_id) in [
        (
            DdlBarrierLookup::new("", "retail", "sales"),
            "ddl-barrier-123",
        ),
        (
            DdlBarrierLookup::new(" source-a ", "retail", "sales"),
            "ddl-barrier-123",
        ),
        (
            DdlBarrierLookup::new("source-a", "", "sales"),
            "ddl-barrier-123",
        ),
        (
            DdlBarrierLookup::new("source-a", " retail ", "sales"),
            "ddl-barrier-123",
        ),
        (
            DdlBarrierLookup::new("source-a", "retail", " sales "),
            "ddl-barrier-123",
        ),
        (DdlBarrierLookup::new("source-a", "retail", "sales"), ""),
        (
            DdlBarrierLookup::new("source-a", "retail", "sales"),
            " ddl-barrier-123 ",
        ),
    ] {
        let error =
            crate::ddl_barrier_identity::validate_summary_lookup_identity(&flow, barrier_id)
                .expect_err("malformed lookup identity");

        assert!(error.to_string().contains("DDL barrier lookup"));
    }
}
