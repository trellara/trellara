# trellara-pg-extension

`trellara-pg-extension` is Trellara's native PostgreSQL source-adapter foundation. It defines the in-process contracts for a low-overhead sequencer, bounded shared-memory handoff, supervised worker drain, and durable source feedback while keeping broker I/O outside PostgreSQL.

## Responsibilities

- Build PostgreSQL-major-specific native modules for the PostgreSQL 15, 16, 17, and 18 CI/package matrix.
- Define committed transaction frames, relation metadata, queue admission, and ordered drain contracts.
- Plan and validate shared-memory sizing and lifecycle gates.
- Define logical-decoding hook registration and background-worker supervision contracts.
- Define relay handoff and source-feedback proofs that cannot advance before durable publication.
- Report implemented and pending data-plane components through SQL.

## Two-phase commit

Two-phase commit is not supported. The output plugin registers every `filter_prepare_cb`,
`begin_prepare_cb`, `prepare_cb`, `commit_prepared_cb`, `rollback_prepared_cb`, and
`stream_prepare_cb` hook, but each fails closed with an explicit error rather than silently
dropping or misordering prepared-transaction changes. Databases captured by the native path
must not rely on `PREPARE TRANSACTION` for tables covered by the publication.

## Boundaries

This crate owns native PostgreSQL integration and the local handoff contract. It does not own broker clients, target apply, lake writes, or general connector orchestration. `trellara-relay` converts durable publish evidence into native feedback; `trellara-protocol` remains the cross-process transaction contract.

The code and extension workflow build and exercise PostgreSQL 15, 16, 17, and 18. The shared `trellara-runtime` release contract currently advertises native PostgreSQL 17 and 18; until those two sources are reconciled, treat 15-16 as build/package coverage rather than a released compatibility promise. The runtime preallocates a fixed shared-memory queue during postmaster preload, decodes committed relation/DML/truncate/message frames, preserves unchanged TOAST markers, filters replayed replication-origin changes, and uses a supervised worker to hand frames to the authenticated local relay. PostgreSQL advances the logical slot only after the relay has fsynced matching evidence and returned a verified acknowledgement.

## SQL surface

The extension currently provides:

- a PostgreSQL-major-specific native module for PostgreSQL 15, 16, 17, or 18;
- `trellara.extension_version()` and `trellara.protocol_version()`;
- `trellara.extension_status()` for machine-readable readiness;
- `trellara.data_plane_readiness()` for component-level native readiness;
- `trellara.runtime_status()` for queue, backpressure, worker, relay, and durable-LSN metrics;
- `trellara.shared_memory_plan(capacity_frames, max_frame_payload_bytes, max_buffered_payload_bytes)` for preload sizing;
- `trellara.capture_contract(source_id, dataset_id)` for the intended transaction, handoff, and acknowledgement boundary;
- a PostgreSQL-independent contract test suite that runs in the normal workspace.

Readiness remains fail-closed: runtime `data_plane_ready` stays false unless the module was preloaded, capture is enabled, the supervised worker has started, and every required identity, socket, and secret setting is valid.

## Server configuration

Both the module preload and PostgreSQL 15+ output-plugin allowlist are required. The relay socket and secret must match the relay service configuration.

```conf
shared_preload_libraries = 'trellara_pg_extension'
output_plugin_libraries = 'trellara_pg_extension'
wal_level = logical
trellara.enabled = on
trellara.database_name = 'postgres'
trellara.slot_name = 'trellara_native'
trellara.source_id = 'source-a'
trellara.dataset_id = 'orders'
trellara.queue_capacity = 8
trellara.relay_socket_path = '/run/trellara/native-relay.sock'
trellara.relay_secret = 'use-a-long-random-secret'
trellara.worker_poll_milliseconds = 100
trellara.relay_timeout_milliseconds = 5000
```

Every setting the extension registers:

| GUC | Type | Default | Range | Context |
| --- | --- | --- | --- | --- |
| `trellara.enabled` | bool | `off` | — | SIGHUP |
| `trellara.database_name` | string | `postgres` | — | postmaster |
| `trellara.slot_name` | string | `trellara_native` | — | SIGHUP |
| `trellara.source_id` | string | `postgres` | — | SIGHUP |
| `trellara.dataset_id` | string | `default` | — | SIGHUP |
| `trellara.relay_socket_path` | string | `/tmp/trellara-native-relay.sock` | — | SIGHUP |
| `trellara.relay_secret` | string | unset | — | SIGHUP, `SUPERUSER_ONLY \| NO_SHOW_ALL` |
| `trellara.queue_capacity` | int | `8` | 1-16 | postmaster |
| `trellara.worker_poll_milliseconds` | int | `100` | 10-60000 | SIGHUP |
| `trellara.relay_timeout_milliseconds` | int | `5000` | 100-60000 | SIGHUP |

`trellara.queue_capacity` is a postmaster setting. The shared-memory ring is statically sized at
`MAX_RUNTIME_QUEUE_FRAMES = 16` slots, so the GUC selects how many of those 16 slots admit frames;
each frame carries at most `MAX_LOGICAL_FRAME_BYTES = 64 KiB` of encoded payload and a larger
committed transaction is rejected rather than truncated. A full queue records backpressure and
leaves the logical slot unacknowledged; it never drops a committed frame. Changes to preload,
capacity, or database name require restart. Other settings reload on SIGHUP. The relay secret is
superuser-only and hidden from `SHOW ALL`.

## Toolchains

The workspace requires and pins Rust 1.97.1, including extension builds. This satisfies pgrx 0.19.2's Rust 1.96-or-newer requirement.

Install and initialize pgrx once:

```console
cargo install cargo-pgrx --version 0.19.2 --locked
cargo pgrx init --pg15 /path/to/postgresql-15/bin/pg_config --pg16 /path/to/postgresql-16/bin/pg_config --pg17 /path/to/postgresql-17/bin/pg_config --pg18 /path/to/postgresql-18/bin/pg_config
```

Run the PostgreSQL-independent tests from the repository root:

```console
cargo test -p trellara-pg-extension
```

Build and install one PostgreSQL major at a time from this directory:

```console
cargo pgrx install --pg-config /path/to/postgresql-17/bin/pg_config
psql -d postgres -c 'CREATE EXTENSION trellara_pg_extension'
psql -d postgres -c 'SELECT trellara.extension_status()'
psql -d postgres -c 'SELECT trellara.data_plane_readiness()'
psql -d postgres -c 'SELECT trellara.runtime_status()'
psql -d postgres -c 'SELECT trellara.shared_memory_plan(1024, 1048576, 1073741824)'
psql -d postgres -c "SELECT trellara.capture_contract('source-a', 'orders')"
```

For any major in the build matrix, use that major's `pg_config`. Never enable more than one PostgreSQL feature together: PostgreSQL extension binaries are major-version-specific.

Run the deterministic live/crash suite against an installed server tree:

```console
TRELLARA_PG_CONFIG=/path/to/postgresql-17/bin/pg_config \
  ../../scripts/test-native-postgres-extension.sh
```

The suite performs live installation, `CREATE EXTENSION`, every implemented decoding callback, abort suppression, bounded backpressure, crash-after-fsync-before-ACK recovery, slot advancement, and background-worker restart. Linux artifacts and operator steps are documented in [`packaging/postgres-extension`](../../packaging/postgres-extension/README.md).

See [skills.md](skills.md) for native-extension development standards and the [workspace map](../README.md) for the surrounding data path.
