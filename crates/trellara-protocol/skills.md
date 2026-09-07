# Development standards: trellara-protocol

## Engineering goal

Maintain a small, deterministic, versioned domain kernel. Protocol behavior must be explicit enough that independent producers and consumers reach the same decision from the same bytes.

## Rust standards

- Preserve the workspace MSRV and edition; use workspace-managed dependency versions.
- Prefer precise domain types and typed errors over strings, booleans, or loosely structured maps.
- Keep public APIs documented and intentionally exported from `lib.rs`; implementation modules stay private.
- Avoid panics, unchecked indexing, lossy casts, wall-clock reads, randomness, and I/O in production paths.
- Use checked arithmetic and explicit size limits for untrusted or serialized input.
- Keep functions focused. Split validation, planning, and reconstruction rather than mixing their side effects.
- Do not add `unsafe` code. If a future requirement makes it unavoidable, document every invariant and add focused misuse tests.

## Protocol guardrails

- Keep the two boundary vocabularies distinct. `DatasetMode` (configuration, owned by
  `trellara-cli`) has two values; `ManifestBoundaryMode` (wire) has three. There is no
  `strict_chunked` dataset mode, and a document or error message that implies one is a defect.
- Fail closed on unknown versions, missing metadata, conflicting duplicates, invalid ordering, or incomplete manifests.
- Never expose partial transaction visibility as committed state.
- Preserve deterministic serialization, routing, hashing, and error classification.
- Treat field removal, renaming, meaning changes, and default changes as compatibility work requiring an explicit migration plan.
- Keep transport- and database-specific concepts out of this crate unless they are part of the stable cross-system contract.

## Testing expectations

- Add success, boundary, malformed-input, and conflicting-duplicate tests for each new invariant.
- Use property tests for ordering, routing, reconstruction, LSN, and idempotency behavior where the input space matters.
- Add serialization round trips and compatibility fixtures for wire-visible changes.
- Update the protocol section of `docs/DESIGN.md` when a wire-visible invariant, boundary mode, or compatibility rule changes.
- Assert exact failure variants when fail-closed behavior is part of the contract.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-protocol --all-targets -- -D warnings
cargo test -p trellara-protocol
git diff --check
```
