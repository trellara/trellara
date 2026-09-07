# Development standards: trellara-sim

## Engineering goal

Make correctness failures reproducible and legible: the same seed, workload, and failure point must produce the same state transitions and report.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Keep simulations deterministic: use the crate RNG, explicit logical time, and stable iteration order.
- Prefer production protocol and plan types over duplicate simulator-only representations.
- Model transitions with typed state and named failure points.
- Avoid panics except for test assertions; scenario execution returns structured outcomes.
- Keep reports data-first and render them separately in the CLI.
- Avoid filesystem, network, environment, and wall-clock dependencies in core scenarios.

## Simulation guardrails

- A failure point identifies one exact boundary; do not make it trigger multiple unrelated side effects.
- Model crash semantics honestly: volatile state disappears, durable state remains, and ambiguous operations remain ambiguous.
- Keep source acknowledgement, target acknowledgement, checkpoint, dedup, and visibility as distinct facts.
- Exact replay and conflicting replay must have different outcomes.
- New production invariants require corresponding simulator assertions when the model covers that subsystem.
- Never change expected reports merely to make a regression pass without reviewing the underlying state transition.

## Testing expectations

- Assert determinism by repeating scenarios with the same seed.
- Cover failure immediately before and after every durable boundary.
- Add recovery and convergence assertions, not only expected error assertions.
- Keep scenario fixtures small enough that ordering and state can be audited manually.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-sim --all-targets -- -D warnings
cargo test -p trellara-sim
git diff --check
```
