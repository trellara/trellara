pub(crate) const INSERT_RESEED_EVENT: &str = r#"
    insert into trellara.reseed_events
        (source_id, dataset_id, watermark_lsn, table_count, copied_rows)
    values ($1, $2, $3, $4, $5)
    "#;

pub(crate) const LOAD_LATEST_RESEED_EVENT: &str = r#"
    select source_id,
           dataset_id,
           watermark_lsn,
           table_count,
           copied_rows,
           completed_at::text
      from trellara.reseed_events
     where source_id = $1
       and dataset_id = $2
     order by completed_at desc
     limit 1
    "#;

pub(crate) const INSERT_SNAPSHOT_HANDOFF_EVENT: &str = r#"
    insert into trellara.snapshot_handoff_events
        (source_id, dataset_id, relation, watermark_lsn, copied_rows)
    values ($1, $2, $3, $4, $5)
    "#;

pub(crate) const LOAD_LATEST_SNAPSHOT_HANDOFF_EVENT: &str = r#"
    select source_id,
           dataset_id,
           relation,
           watermark_lsn,
           copied_rows,
           completed_at::text
      from trellara.snapshot_handoff_events
     where source_id = $1
       and dataset_id = $2
     order by completed_at desc
     limit 1
    "#;

pub(crate) const INSERT_VALIDATION_EVENT: &str = r#"
    insert into trellara.validation_events
        (source_id, dataset_id, source_watermark_lsn, target_watermark_lsn,
         converged, table_count, drift_count, drift_relations, evidence_sha256)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

pub(crate) const LOAD_LATEST_VALIDATION_EVENT: &str = r#"
    select source_id,
           dataset_id,
           source_watermark_lsn,
           target_watermark_lsn,
           converged,
           table_count,
           drift_count,
           coalesce(drift_relations, '{}'::text[]),
           evidence_sha256,
           completed_at::text
      from trellara.validation_events
     where source_id = $1
       and dataset_id = $2
     order by completed_at desc
     limit 1
    "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_evidence_queries_order_by_completion_time() {
        for query in [
            LOAD_LATEST_RESEED_EVENT,
            LOAD_LATEST_SNAPSHOT_HANDOFF_EVENT,
            LOAD_LATEST_VALIDATION_EVENT,
        ] {
            assert!(query.contains("order by completed_at desc"));
            assert!(query.contains("limit 1"));
        }
    }

    #[test]
    fn validation_load_coalesces_missing_drift_relations_to_empty_array() {
        assert!(LOAD_LATEST_VALIDATION_EVENT.contains("coalesce(drift_relations, '{}'::text[])"));
    }

    #[test]
    fn validation_events_persist_evidence_digest() {
        assert!(INSERT_VALIDATION_EVENT.contains("evidence_sha256"));
        assert!(LOAD_LATEST_VALIDATION_EVENT.contains("evidence_sha256"));
    }

    #[test]
    fn evidence_inserts_target_the_expected_tables() {
        assert!(INSERT_RESEED_EVENT.contains("trellara.reseed_events"));
        assert!(INSERT_SNAPSHOT_HANDOFF_EVENT.contains("trellara.snapshot_handoff_events"));
        assert!(INSERT_VALIDATION_EVENT.contains("trellara.validation_events"));
    }
}
