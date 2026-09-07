use super::*;

proptest! {
    #[test]
    fn idempotency_key_property_is_deterministic(
        source_id in "[a-z][a-z0-9_-]{0,12}",
        commit_lsn_hi in 0_u32..10_000,
        commit_lsn_lo in 0_u32..10_000,
        transaction_id in "[a-z][a-z0-9_-]{0,16}",
        total_order in 0_u32..10_000,
    ) {
        let commit_lsn = format!("{commit_lsn_hi:X}/{commit_lsn_lo:X}");

        prop_assert_eq!(
            idempotency_key(&source_id, &commit_lsn, &transaction_id, total_order),
            idempotency_key(&source_id, &commit_lsn, &transaction_id, total_order),
        );
    }

    #[test]
    fn lsn_ordering_property_matches_numeric_order(left in any::<u64>(), right in any::<u64>()) {
        let left_text = format_lsn(left);
        let right_text = format_lsn(right);

        prop_assert_eq!(parse_lsn(&left_text).expect("left lsn"), left);
        prop_assert_eq!(parse_lsn(&right_text).expect("right lsn"), right);
        prop_assert_eq!(
            parse_lsn(&left_text).expect("left lsn").cmp(&parse_lsn(&right_text).expect("right lsn")),
            left.cmp(&right),
        );
    }
}
