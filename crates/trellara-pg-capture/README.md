# trellara-pg-capture

`trellara-pg-capture` is the client-side PostgreSQL source adapter. It inspects a source, consumes logical replication, decodes pgoutput, and assembles committed changes into Trellara protocol transactions.

## Responsibilities

- Inspect publications, slots, table metadata, replica identity, and conflicting subscriptions.
- Authenticate and start PostgreSQL logical replication streams.
- Parse replication frames, keepalives, pgoutput messages, tuples, relation metadata, DML, truncate, commit, abort, and streamed transactions.
- Track relation/schema identities and assemble ordered transaction envelopes.
- Spill large or streaming transactions within configured memory and disk limits.
- Report only the last durable acknowledged LSN in replication feedback.
- Retain `test_decoding` support for focused compatibility and test paths.

## Capture configuration defaults

`PgOutputProtocolConfig` defaults to `protocol_version: 2` with `streaming: true`, and validates
that combination: the version must be `1` or `2`, and `streaming` requires version 2. Version 1 is
only valid with streaming disabled, and the `streaming` start option is omitted from the
`START_REPLICATION` command in that case.

Spill is bounded by `stream_spill_threshold_changes`, defaulting to
`DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES = 1024` and capped at
`MAX_STREAM_SPILL_THRESHOLD_CHANGES = 1_000_000`; zero is rejected. `stream_spill_dir` selects
where spill artifacts land and must not be empty when set.

`SourceCaptureKind` is `pg_output` (default) or `test_decoding`; the kind determines the plugin name
preflight expects on the slot. `test_decoding` exists for focused compatibility work, not
production capture.

## Boundaries

The `trellara-check` standalone diagnostic reuses this crate's read-only source-safety inspection and its exact query manifest. Treat `source_safety_read_only_queries` and the safety-inspection types as an external compatibility surface, not an internal detail.

Capture observes and assembles source changes; it does not decide that a transaction is durably published. `trellara-relay` owns publish/checkpoint/source-ACK sequencing. Protocol types belong in `trellara-protocol`, and native in-process capture belongs in `trellara-pg-extension`.

Do not perform target apply, broker-specific operations, or arbitrary transformations here.

## Key entry points

- `PgOutputStreamCapture`, `PgOutputDecoder`, and `LogicalEvent`
- `TransactionAssembler` and `TransactionAssemblerConfig`
- `PgChangeSource` and source-inspection APIs
- `PgCapture` and `TestDecodingCapture`

## Development

```console
cargo test -p trellara-pg-capture
cargo clippy -p trellara-pg-capture --all-targets -- -D warnings
```

Most decoder and assembler tests are service-free. Tests that exercise a real replication connection require an explicitly configured PostgreSQL instance. See [skills.md](skills.md) and the [workspace map](../README.md).
