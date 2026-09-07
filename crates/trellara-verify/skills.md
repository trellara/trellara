# Development standards: trellara-verify

## Engineering goal

Produce deterministic, explainable convergence evidence without hiding uncertainty, sampling bias, unsupported values, or incomplete relation identity.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Separate PostgreSQL inspection, snapshot acquisition, canonicalization, comparison, reporting, and reseed execution.
- Use typed drift records and errors; avoid stringly typed classifications.
- Keep hashing deterministic across process, platform, row order, and JSON object-key order.
- Use checked limits for rows, samples, value sizes, and queries.
- Parameterize values and validate/quote identifiers through centralized SQL helpers.
- Avoid logging credentials or unrestricted row contents.

## Verification guardrails

- Require a stable relation identity before claiming row-level convergence.
- Canonicalization must distinguish SQL NULL, JSON null, absent fields, numeric representations, byte strings, and temporal values intentionally.
- Sort inputs or use order-independent aggregation where the report claims order independence.
- Never report sampled equality as full equality.
- Reseed is an explicit mutation path: validate source/target relations and keep target changes transactional.
- Verification watermarks and evidence must identify exactly which source boundary was compared.

## Testing expectations

- Add canonicalization fixtures for every supported PostgreSQL value shape.
- Cover missing/extra/mismatched rows, duplicate keys, empty tables, ordering differences, and bounded samples.
- Test deterministic hashes and reports across repeated runs.
- Keep unit tests database-free; put live PostgreSQL coverage behind explicit integration setup.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-verify --all-targets -- -D warnings
cargo test -p trellara-verify
git diff --check
```
