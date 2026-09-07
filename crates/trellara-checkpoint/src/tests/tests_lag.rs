use super::*;
use proptest::prelude::*;
use trellara_protocol::Checkpoint;

#[test]
fn checkpoint_lag_reports_watermark_gaps() {
    let lag = CheckpointLag::from_checkpoint(&Checkpoint {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        last_seen_lsn: "0/16B7000".to_string(),
        last_durable_lsn: "0/16B6C50".to_string(),
        last_applied_lsn: "0/16B6B00".to_string(),
    });

    assert_eq!(lag.seen_to_durable_bytes, 944);
    assert_eq!(lag.durable_to_applied_bytes, 336);
    assert_eq!(lag.seen_to_applied_bytes, 1280);
    assert!(!lag.source_is_durable);
    assert!(!lag.target_is_caught_up);
}

#[test]
fn checkpoint_lag_treats_empty_lsn_as_zero() {
    let lag = CheckpointLag::from_checkpoint(&Checkpoint {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        last_seen_lsn: "0/16B6C50".to_string(),
        last_durable_lsn: "0/16B6C50".to_string(),
        last_applied_lsn: String::new(),
    });

    assert_eq!(lag.durable_to_applied_bytes, parse_lsn("0/16B6C50"));
    assert!(lag.source_is_durable);
    assert!(!lag.target_is_caught_up);
}

proptest! {
    #[test]
    fn checkpoint_lag_property_matches_ordered_lsn_distances(
        applied in 0u64..(1u64 << 40),
        durable_delta in 0u64..1_000_000,
        seen_delta in 0u64..1_000_000,
    ) {
        let durable = applied + durable_delta;
        let seen = durable + seen_delta;
        let checkpoint = Checkpoint {
            source_id: "source-a".to_string(),
            dataset_id: "sales".to_string(),
            last_seen_lsn: lsn_string(seen),
            last_durable_lsn: lsn_string(durable),
            last_applied_lsn: lsn_string(applied),
        };

        let lag = CheckpointLag::from_checkpoint(&checkpoint);

        prop_assert_eq!(lag.seen_to_durable_bytes, seen - durable);
        prop_assert_eq!(lag.durable_to_applied_bytes, durable - applied);
        prop_assert_eq!(lag.seen_to_applied_bytes, seen - applied);
        prop_assert_eq!(lag.source_is_durable, seen_delta == 0);
        prop_assert_eq!(lag.target_is_caught_up, durable_delta == 0);
    }
}
