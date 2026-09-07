pub(crate) fn transaction_boundary_guarantee(mode: &str) -> &'static str {
    if is_partitioned_mode(mode) {
        "partition manifest and commit marker barrier reconstructs a complete source transaction before atomic apply"
    } else if is_strict_chunked_mode(mode) {
        "strict chunk manifest and commit marker barrier reconstructs a complete source transaction before atomic apply"
    } else {
        "one committed source transaction is published and applied as one ordered atomic envelope"
    }
}

pub(crate) fn transaction_visibility_contract(mode: &str) -> &'static str {
    if is_partitioned_mode(mode) {
        "barrier-aware consumers wait for the manifest, commit marker, and every partition chunk before global visibility; partition-local consumers may read a lane earlier but must treat it as local-only"
    } else if is_strict_chunked_mode(mode) {
        "strict-chunk consumers wait for the manifest, commit marker, and every strict chunk before visibility"
    } else {
        "strict consumers see one committed source transaction as one globally visible ordered envelope"
    }
}

pub(crate) fn parallel_replay_contract(mode: &str) -> &'static str {
    if is_partitioned_mode(mode) {
        "partition workers may replay DML-only committed transactions by partition, while DDL-only and mixed DDL/DML transactions must use the DDL barrier release path before post-DDL DML becomes globally visible"
    } else if is_strict_chunked_mode(mode) {
        "strict chunks preserve source transaction order; DDL-only and mixed DDL/DML transactions must use the DDL barrier release path before post-DDL DML replay"
    } else {
        "parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope"
    }
}

pub(crate) fn is_manifest_barrier_mode(mode: &str) -> bool {
    is_partitioned_mode(mode) || is_strict_chunked_mode(mode)
}

pub(crate) fn is_partitioned_mode(mode: &str) -> bool {
    mode == "partitioned_scale_mode"
}

pub(crate) fn is_strict_chunked_mode(mode: &str) -> bool {
    mode == "strict_chunked_transaction_order"
}
