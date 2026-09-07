# Trellara Spark Fan-In Templates

These templates derive analytical tables from Trellara's append-only raw CDC Iceberg history.

They intentionally do not read open, quarantined, or unverified epochs. By default, templates consume only epochs whose `_trellara_epochs.state` is `complete` and whose `_trellara_verification.checksum_status` is `match`. To process `complete_with_gaps`, set the template parameter `${accept_complete_with_gaps}` to `true` and keep that choice in the job run evidence.

## Parameters

| Placeholder | Meaning |
| --- | --- |
| `${catalog}` | Spark catalog name. |
| `${namespace}` | Iceberg namespace or database. |
| `${raw_cdc_table}` | Trellara raw CDC table. |
| `${epochs_table}` | Trellara `_trellara_epochs` metadata table. |
| `${epoch_partitions_table}` | Trellara `_trellara_epoch_partitions` metadata table for partition-level transaction evidence. |
| `${verification_table}` | Trellara `_trellara_verification` metadata table. |
| `${target_table}` | Current-state or SCD2 output table. |
| `${epoch_id}` | Epoch id to consume. |
| `${primary_key_column}` | Reviewed source primary-key declaration recorded with the rendered job; row identity is the writer's canonical `record_key`. |
| `${accept_complete_with_gaps}` | `true` only when the caller accepts explicit epoch gaps. |

## Templates

- `current_state.sql` builds or refreshes a latest-row table from the actual raw CDC columns, rejecting stale LSN/order updates and deletes.
- `scd2.sql` closes previous versions and inserts replay-safe valid-time versions, including correct handling when a newer epoch already exists.
- `maintenance.sql` runs verification-gated Iceberg bin-pack, snapshot expiry, and orphan-file procedures for raw CDC, all metadata tables, and one derived target table.
- `completeness_dashboard.sql` renders read-only epoch and per-source completeness queries for data-platform review.

Pilot packages also include `spark-current-state.py`, `spark-scd2.py`, `spark-maintenance.py`, and `spark-completeness-dashboard.py`. These PySpark runners execute the rendered SQL files with `spark-submit`, refuse unresolved `${...}` placeholders, and print the epoch plus `complete_with_gaps` acceptance choice as run evidence.

Each runner also prints `template_sha256`, computed from the SQL file at execution time. Use that digest in `trellara schema ddl-barrier ack --sink spark_derived_views --template-digest ...` so the DDL barrier ACK is bound to the exact rendered Spark view or maintenance artifact that ran.

The derivation templates are idempotent when the target table enforces Trellara idempotency columns and the job is rerun for the same `${epoch_id}`. The maintenance template is safe to rerun because it changes file layout and snapshot retention, not transaction visibility. The verification preflight is part of the consumption contract: if stream-to-lake reconciliation reports anything other than `match`, the derived table job fails before applying changes for that epoch.
