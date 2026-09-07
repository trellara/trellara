-- Trellara Spark template: lake fan-in completeness dashboard queries.
--
-- Parameters:
--   ${catalog}
--   ${namespace}
--   ${epochs_table}
--   ${epoch_partitions_table}
--   ${verification_table}
--   ${quarantine_table}
--   ${target_table}
--   ${epoch_id}
--   ${accept_complete_with_gaps}
--   ${unsafe_allow_non_consumable_epoch}
--   ${unsafe_override_reason}

WITH consumable_epoch AS (
  SELECT
    e.epoch_id,
    e.dataset_id,
    e.state,
    e.policy,
    e.required_source_count,
    e.complete_source_count,
    e.missing_source_count,
    e.quarantined_source_count,
    e.transaction_count,
    e.change_count,
    e.checksum_rollup,
    e.manifest_digest,
    v.verification_id,
    v.checksum_status
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
epoch_release_gate AS (
  SELECT
    CASE
      WHEN count(*) = 1 THEN 'safe_to_publish'
      ELSE raise_error('Trellara epoch is not consumable: verification must match and gaps must be explicitly accepted unless an unsafe override is recorded')
    END AS release_decision
  FROM consumable_epoch
)
SELECT
  ce.epoch_id,
  ce.dataset_id,
  ce.state,
  ce.policy,
  ce.required_source_count,
  ce.complete_source_count,
  ce.missing_source_count,
  ce.quarantined_source_count,
  ce.transaction_count,
  ce.change_count,
  ce.checksum_status,
  '${unsafe_override_reason}' AS unsafe_override_reason,
  ce.verification_id,
  ce.manifest_digest,
  gate.release_decision
FROM consumable_epoch ce
CROSS JOIN epoch_release_gate gate
ORDER BY ce.epoch_id;

SELECT
  source_id,
  state,
  start_lsn,
  end_lsn,
  transaction_count,
  change_count,
  checksum_rollup,
  lag_reason
FROM ${catalog}.${namespace}.${target_table}
WHERE epoch_id = '${epoch_id}'
ORDER BY state, source_id;

SELECT
  source_id,
  partition_count,
  first_commit_lsn,
  last_commit_lsn,
  partition_transaction_count,
  partition_event_count,
  partition_checksum_rollup
FROM (
  SELECT
    source_id,
    count(*) AS partition_count,
    min(first_commit_lsn) AS first_commit_lsn,
    max(last_commit_lsn) AS last_commit_lsn,
    sum(transaction_count) AS partition_transaction_count,
    sum(event_count) AS partition_event_count,
    sum(checksum_rollup) AS partition_checksum_rollup
  FROM ${catalog}.${namespace}.${epoch_partitions_table}
  WHERE epoch_id = '${epoch_id}'
  GROUP BY source_id
) partition_evidence
ORDER BY source_id;

SELECT
  source_id,
  transaction_id,
  commit_lsn,
  reason,
  details,
  recovery_command
FROM ${catalog}.${namespace}.${quarantine_table}
WHERE epoch_id = '${epoch_id}'
ORDER BY reason, source_id, transaction_id, commit_lsn;
