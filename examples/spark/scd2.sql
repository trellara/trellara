-- Trellara Spark template: raw CDC to SCD2 history table.
-- Required parameters:
--   ${catalog}
--   ${namespace}
--   ${raw_cdc_table}
--   ${epochs_table}
--   ${verification_table}
--   ${target_table}
--   ${epoch_id}
--   ${primary_key_column}
--   ${accept_complete_with_gaps}
--   ${unsafe_allow_non_consumable_epoch}
--   ${unsafe_override_reason}

WITH consumable_epoch AS (
  SELECT e.epoch_id
  FROM ${catalog}.${namespace}.${epochs_table} e
  JOIN ${catalog}.${namespace}.${verification_table} v
    ON e.epoch_id = v.epoch_id
  WHERE e.epoch_id = '${epoch_id}'
    AND (v.checksum_status = 'match' OR ${unsafe_allow_non_consumable_epoch} = true)
    AND (
      e.state = 'complete'
      OR (
        e.state = 'complete_with_gaps'
        AND e.policy = 'publish_with_gaps'
        AND ${accept_complete_with_gaps} = true
      )
      OR ${unsafe_allow_non_consumable_epoch} = true
    )
)
SELECT
  CASE
    WHEN count(*) = 1 THEN 'ready'
    ELSE raise_error('Trellara epoch is not consumable: verification must match and gaps must be explicitly accepted unless an unsafe override is recorded')
  END AS trellara_epoch_consumable
FROM consumable_epoch;

CREATE TABLE IF NOT EXISTS ${catalog}.${namespace}.${target_table} (
  __trellara_epoch_id STRING NOT NULL,
  __trellara_source_id STRING NOT NULL,
  __trellara_relation STRING NOT NULL,
  __trellara_record_key STRING NOT NULL,
  __trellara_transaction_id STRING NOT NULL,
  __trellara_commit_lsn STRING NOT NULL,
  __trellara_commit_lsn_numeric DECIMAL(38, 0) NOT NULL,
  __trellara_total_order INT NOT NULL,
  __trellara_valid_from TIMESTAMP NOT NULL,
  __trellara_valid_to TIMESTAMP,
  __trellara_is_current BOOLEAN NOT NULL,
  __trellara_idempotency_key STRING NOT NULL,
  __trellara_close_epoch_id STRING,
  __trellara_close_commit_lsn STRING,
  row_after_json STRING NOT NULL
) USING iceberg;

WITH consumable_epoch AS (
  SELECT e.epoch_id
  FROM ${catalog}.${namespace}.${epochs_table} e
  JOIN ${catalog}.${namespace}.${verification_table} v
    ON e.epoch_id = v.epoch_id
  WHERE e.epoch_id = '${epoch_id}'
    AND (v.checksum_status = 'match' OR ${unsafe_allow_non_consumable_epoch} = true)
    AND (
      e.state = 'complete'
      OR (
        e.state = 'complete_with_gaps'
        AND e.policy = 'publish_with_gaps'
        AND ${accept_complete_with_gaps} = true
      )
      OR ${unsafe_allow_non_consumable_epoch} = true
    )
),
ordered_changes AS (
  SELECT
    c.epoch_id AS __trellara_epoch_id,
    c.source_id AS __trellara_source_id,
    c.relation AS __trellara_relation,
    c.transaction_id AS __trellara_transaction_id,
    c.commit_lsn AS __trellara_commit_lsn,
    (
      CAST(conv(element_at(split(c.commit_lsn, '/'), 1), 16, 10) AS DECIMAL(38, 0))
        * CAST(4294967296 AS DECIMAL(38, 0))
      + CAST(conv(element_at(split(c.commit_lsn, '/'), 2), 16, 10) AS DECIMAL(38, 0))
    ) AS __trellara_commit_lsn_numeric,
    timestamp_millis(c.commit_timestamp_ms) AS __trellara_commit_timestamp,
    c.total_order AS __trellara_total_order,
    c.operation AS __trellara_operation,
    c.idempotency_key AS __trellara_idempotency_key,
    c.record_key AS __trellara_record_key,
    c.payload_after_json AS row_after_json,
    '${primary_key_column}' AS __trellara_primary_key_column,
    '${unsafe_override_reason}' AS __trellara_unsafe_override_reason
  FROM ${catalog}.${namespace}.${raw_cdc_table} c
  JOIN consumable_epoch e
    ON c.epoch_id = e.epoch_id
  WHERE c.operation IN ('insert', 'update', 'delete')
    AND c.record_key IS NOT NULL
),
sequenced_changes AS (
  SELECT
    *,
    ROW_NUMBER() OVER (
      PARTITION BY __trellara_source_id, __trellara_relation, __trellara_record_key
      ORDER BY __trellara_commit_lsn_numeric, __trellara_total_order
    ) AS __trellara_sequence_ascending,
    LEAD(__trellara_commit_timestamp) OVER (
      PARTITION BY __trellara_source_id, __trellara_relation, __trellara_record_key
      ORDER BY __trellara_commit_lsn_numeric, __trellara_total_order
    ) AS __trellara_next_commit_timestamp
  FROM ordered_changes
),
first_changes AS (
  SELECT *
  FROM sequenced_changes
  WHERE __trellara_sequence_ascending = 1
)
MERGE INTO ${catalog}.${namespace}.${target_table} AS target
USING first_changes AS source
ON target.__trellara_source_id = source.__trellara_source_id
  AND target.__trellara_relation = source.__trellara_relation
  AND target.__trellara_record_key = source.__trellara_record_key
  AND target.__trellara_is_current = true
  AND target.__trellara_epoch_id <> source.__trellara_epoch_id
  AND (
    source.__trellara_commit_lsn_numeric > target.__trellara_commit_lsn_numeric
    OR (
      source.__trellara_commit_lsn_numeric = target.__trellara_commit_lsn_numeric
      AND source.__trellara_total_order > target.__trellara_total_order
    )
  )
WHEN MATCHED THEN UPDATE SET
  target.__trellara_valid_to = source.__trellara_commit_timestamp,
  target.__trellara_is_current = false,
  target.__trellara_close_epoch_id = source.__trellara_epoch_id,
  target.__trellara_close_commit_lsn = source.__trellara_commit_lsn;

WITH consumable_epoch AS (
  SELECT e.epoch_id
  FROM ${catalog}.${namespace}.${epochs_table} e
  JOIN ${catalog}.${namespace}.${verification_table} v
    ON e.epoch_id = v.epoch_id
  WHERE e.epoch_id = '${epoch_id}'
    AND (v.checksum_status = 'match' OR ${unsafe_allow_non_consumable_epoch} = true)
    AND (
      e.state = 'complete'
      OR (
        e.state = 'complete_with_gaps'
        AND e.policy = 'publish_with_gaps'
        AND ${accept_complete_with_gaps} = true
      )
      OR ${unsafe_allow_non_consumable_epoch} = true
    )
),
ordered_changes AS (
  SELECT
    c.epoch_id AS __trellara_epoch_id,
    c.source_id AS __trellara_source_id,
    c.relation AS __trellara_relation,
    c.transaction_id AS __trellara_transaction_id,
    c.commit_lsn AS __trellara_commit_lsn,
    (
      CAST(conv(element_at(split(c.commit_lsn, '/'), 1), 16, 10) AS DECIMAL(38, 0))
        * CAST(4294967296 AS DECIMAL(38, 0))
      + CAST(conv(element_at(split(c.commit_lsn, '/'), 2), 16, 10) AS DECIMAL(38, 0))
    ) AS __trellara_commit_lsn_numeric,
    timestamp_millis(c.commit_timestamp_ms) AS __trellara_commit_timestamp,
    c.total_order AS __trellara_total_order,
    c.operation AS __trellara_operation,
    c.idempotency_key AS __trellara_idempotency_key,
    c.record_key AS __trellara_record_key,
    c.payload_after_json AS row_after_json
  FROM ${catalog}.${namespace}.${raw_cdc_table} c
  JOIN consumable_epoch e
    ON c.epoch_id = e.epoch_id
  WHERE c.operation IN ('insert', 'update', 'delete')
    AND c.record_key IS NOT NULL
),
sequenced_changes AS (
  SELECT
    *,
    LEAD(__trellara_commit_timestamp) OVER (
      PARTITION BY __trellara_source_id, __trellara_relation, __trellara_record_key
      ORDER BY __trellara_commit_lsn_numeric, __trellara_total_order
    ) AS __trellara_next_commit_timestamp,
    LEAD(__trellara_operation) OVER (
      PARTITION BY __trellara_source_id, __trellara_relation, __trellara_record_key
      ORDER BY __trellara_commit_lsn_numeric, __trellara_total_order
    ) AS __trellara_next_operation
  FROM ordered_changes
),
opening_candidates AS (
  SELECT *
  FROM sequenced_changes
  WHERE __trellara_operation IN ('insert', 'update')
),
opening_changes AS (
  SELECT
    source.*,
    (
      SELECT MIN(target.__trellara_valid_from)
      FROM ${catalog}.${namespace}.${target_table} target
      WHERE target.__trellara_source_id = source.__trellara_source_id
        AND target.__trellara_relation = source.__trellara_relation
        AND target.__trellara_record_key = source.__trellara_record_key
        AND (
          target.__trellara_commit_lsn_numeric > source.__trellara_commit_lsn_numeric
          OR (
            target.__trellara_commit_lsn_numeric = source.__trellara_commit_lsn_numeric
            AND target.__trellara_total_order > source.__trellara_total_order
          )
        )
    ) AS __trellara_next_existing_valid_from
  FROM opening_candidates source
)
MERGE INTO ${catalog}.${namespace}.${target_table} AS target
USING opening_changes AS source
ON target.__trellara_source_id = source.__trellara_source_id
  AND target.__trellara_relation = source.__trellara_relation
  AND target.__trellara_record_key = source.__trellara_record_key
  AND target.__trellara_idempotency_key = source.__trellara_idempotency_key
WHEN NOT MATCHED THEN INSERT (
  __trellara_epoch_id,
  __trellara_source_id,
  __trellara_relation,
  __trellara_record_key,
  __trellara_transaction_id,
  __trellara_commit_lsn,
  __trellara_commit_lsn_numeric,
  __trellara_total_order,
  __trellara_valid_from,
  __trellara_valid_to,
  __trellara_is_current,
  __trellara_idempotency_key,
  row_after_json
) VALUES (
  source.__trellara_epoch_id,
  source.__trellara_source_id,
  source.__trellara_relation,
  source.__trellara_record_key,
  source.__trellara_transaction_id,
  source.__trellara_commit_lsn,
  source.__trellara_commit_lsn_numeric,
  source.__trellara_total_order,
  source.__trellara_commit_timestamp,
  CASE
    WHEN source.__trellara_next_commit_timestamp IS NULL
      THEN source.__trellara_next_existing_valid_from
    WHEN source.__trellara_next_existing_valid_from IS NULL
      THEN source.__trellara_next_commit_timestamp
    ELSE least(
      source.__trellara_next_commit_timestamp,
      source.__trellara_next_existing_valid_from
    )
  END,
  source.__trellara_next_commit_timestamp IS NULL
    AND source.__trellara_next_existing_valid_from IS NULL,
  source.__trellara_idempotency_key,
  source.row_after_json
);
