use super::*;

#[test]
fn snapshot_run_state_round_trips_as_storage_value() {
    let states = [
        SnapshotRunState::Planned,
        SnapshotRunState::SlotCreated,
        SnapshotRunState::SnapshotExported,
        SnapshotRunState::CopyingTable,
        SnapshotRunState::CopyComplete,
        SnapshotRunState::StreamHandoffReady,
        SnapshotRunState::Streaming,
        SnapshotRunState::Verified,
        SnapshotRunState::FailedRecoverable,
    ];

    for state in states {
        assert_eq!(
            state
                .to_string()
                .parse::<SnapshotRunState>()
                .expect("parse"),
            state
        );
    }
    assert!("unknown".parse::<SnapshotRunState>().is_err());
}

#[test]
fn snapshot_state_machine_design_doc_matches_storage_states() {
    let design = include_str!("../../../../../docs/DESIGN.md");
    let states = [
        SnapshotRunState::Planned,
        SnapshotRunState::SlotCreated,
        SnapshotRunState::SnapshotExported,
        SnapshotRunState::CopyingTable,
        SnapshotRunState::CopyComplete,
        SnapshotRunState::StreamHandoffReady,
        SnapshotRunState::Streaming,
        SnapshotRunState::Verified,
        SnapshotRunState::FailedRecoverable,
    ];

    assert!(design.contains("Initial Snapshot State Machine"));
    assert!(design.contains("trellara.snapshot_runs"));
    assert!(design.contains("trellara.snapshot_table_progress"));
    assert!(design.contains("trellara.snapshot_handoff_events"));
    assert!(design.contains("crash after handoff event before relay starts"));
    for state in states {
        assert!(
            design.contains(&format!("`{state}`")),
            "design doc should name snapshot state {state}"
        );
    }
}

#[test]
fn snapshot_run_state_allows_forward_and_recoverable_transitions() {
    assert!(SnapshotRunState::Planned.can_start_as());
    assert!(SnapshotRunState::SlotCreated.can_start_as());
    assert!(!SnapshotRunState::CopyingTable.can_start_as());

    assert!(SnapshotRunState::Planned
        .validate_transition(SnapshotRunState::SlotCreated)
        .is_ok());
    assert!(SnapshotRunState::SlotCreated
        .validate_transition(SnapshotRunState::SnapshotExported)
        .is_ok());
    assert!(SnapshotRunState::SnapshotExported
        .validate_transition(SnapshotRunState::CopyingTable)
        .is_ok());
    assert!(SnapshotRunState::CopyingTable
        .validate_transition(SnapshotRunState::CopyComplete)
        .is_ok());
    assert!(SnapshotRunState::CopyComplete
        .validate_transition(SnapshotRunState::StreamHandoffReady)
        .is_ok());
    assert!(SnapshotRunState::StreamHandoffReady
        .validate_transition(SnapshotRunState::Streaming)
        .is_ok());
    assert!(SnapshotRunState::Streaming
        .validate_transition(SnapshotRunState::Verified)
        .is_ok());
    assert!(SnapshotRunState::CopyingTable
        .validate_transition(SnapshotRunState::FailedRecoverable)
        .is_ok());
    assert!(SnapshotRunState::FailedRecoverable
        .validate_transition(SnapshotRunState::CopyingTable)
        .is_ok());
    assert!(SnapshotRunState::Planned
        .validate_transition(SnapshotRunState::FailedRecoverable)
        .is_err());
}

#[test]
fn snapshot_run_state_rejects_unsafe_backward_transitions() {
    assert!(SnapshotRunState::StreamHandoffReady
        .validate_transition(SnapshotRunState::CopyingTable)
        .is_err());
    assert!(SnapshotRunState::Verified
        .validate_transition(SnapshotRunState::Streaming)
        .is_err());
    assert!(SnapshotRunState::Planned
        .validate_transition(SnapshotRunState::StreamHandoffReady)
        .is_err());
}
