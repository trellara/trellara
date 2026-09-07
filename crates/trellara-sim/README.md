# trellara-sim

`trellara-sim` provides deterministic models and failure scenarios for Trellara's correctness boundaries. It exercises relay, apply, checkpoint, snapshot, strict-chunk, and fleet fan-in behavior without requiring production infrastructure.

## Responsibilities

- Model source transactions, streams, targets, checkpoints, and apply state.
- Inject named failures at precise workflow steps.
- Simulate relay/apply recovery and generate structured reports.
- Exercise snapshot copy/handoff state machines.
- Exercise strict-chunk manifests, commit markers, target recovery, and ordering.
- Exercise fleet fan-in duplicates, quarantine, epoch completeness, and policy decisions.
- Exercise qualification/observability scenarios asserting operator-facing signals hold under each named failure point.
- Produce seeded, reproducible scenario outputs for CLI chaos/evidence workflows.

## Boundaries

Simulation is an executable model, not a production transport, database adapter, benchmark, or substitute for integration/chaos testing against real services. It should reuse production domain types and invariants instead of maintaining a divergent protocol.

## Key entry points

- Base failure-matrix simulation types and reports
- Snapshot simulation APIs
- Strict-chunk simulation APIs
- Fleet fan-in simulation APIs
- Named `FailurePoint` values and seeded RNG support

## Development

```console
cargo test -p trellara-sim
cargo clippy -p trellara-sim --all-targets -- -D warnings
```

See [skills.md](skills.md) and the [workspace map](../README.md).

## Suite inventory

Five independently runnable suites, 28 default scenarios total:

| Suite | Entry point | Failure points |
| --- | --- | --- |
| Base failure matrix | `run_default_suite` | 8 |
| Snapshot | `run_default_snapshot_suite` | 6 |
| Strict chunk | `run_default_strict_chunk_suite` | 5 |
| Qualification / observability | `run_default_qualification_suite` | 5 |
| Fleet fan-in | `run_default_fleet_fanin_suite` | 4 |

Update this table when a suite gains or loses a failure point; the correctness report derives its
counts from these suites.
