# Development standards: trellara-cli

## Engineering goal

Provide predictable, scriptable, safe operator workflows while keeping domain correctness in the packages that own it.

## Rust standards

- Preserve the workspace MSRV and use workspace dependencies.
- Keep command parsing, validation, execution, and rendering in separate modules.
- Use typed configuration with explicit defaults, validation, and conversion into domain types.
- Keep reusable logic out of `main.rs`; expose it from the library for tests.
- Return typed errors with actionable context and stable exit behavior. Do not panic on operator input or external failures.
- Keep output deterministic. Separate human text from JSON/YAML artifacts and avoid changing machine schemas casually.
- Use structured tracing; redact passwords, tokens, connection strings, certificates, and unrestricted row values.

## CLI guardrails

- Validate the complete plan before mutation. Dry-run/explain output must reflect the same plan execution will use.
- Require explicit flags or confirmations for destructive recovery, cursor movement, reseed, or unsafe DDL actions.
- Preserve automation compatibility: command names, flags, exit codes, and structured-output fields are public interfaces.
- Do not duplicate protocol, SQL planning, source-ACK, checkpoint, or verification rules in command modules.
- Keep command modules cohesive. Update module manifests when adding or moving a product surface.
- Keep the nine visible top-level commands (`init`, `check`, `preflight`, `run`, `verify`, `status`, `fleet`, `lake`, `config`) focused on the public loop and primary namespaces; hidden compatibility/operator commands still require stable parsing and tests. Anything new that is not part of the public loop gets `#[command(hide = true)]`.
- Check every command string you write in docs or evidence against `args_cli.rs` and `args_commands.rs` before it ships. Subcommand sets are narrower than they look — `repair` has only `plan`, `contract` only `test`, `native` only `worker-report`.
- Keep `TrellaraConfig` `deny_unknown_fields`. A new configuration field is a schema change: bump `CURRENT_CONFIG_VERSION`, add the migration step, and add a redaction test if it can carry a secret.
- Error messages should state what failed, why it matters, and the safest next action without leaking secrets.

## Testing expectations

- Test parsing, validation, execution, and rendering independently.
- Cover default configuration, conflicting flags, missing requirements, invalid files, and external failure mapping.
- Snapshot only stable output surfaces; assert structured fields for machine-readable formats.
- Use in-memory/local fixtures for normal unit tests and mark real-service flows as explicit integration tests.

## Required checks

```console
cargo fmt --all -- --check
cargo clippy -p trellara-cli --all-targets -- -D warnings
cargo test -p trellara-cli
cargo run -p trellara-cli -- --help
git diff --check
```

The default feature set is `local-stream` only, so the commands above never compile the Kafka or
lake paths. When a change touches those, add the build CI runs before release:

```console
cargo build -p trellara-cli --no-default-features --features local-stream
cargo build -p trellara-cli --features full
```
