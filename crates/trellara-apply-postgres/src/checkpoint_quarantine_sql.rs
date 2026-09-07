pub(crate) const RECORD_QUARANTINED_ENVELOPE: &str = r#"
insert into trellara.apply_quarantine
    (source_id, database_id, dataset_id, transaction_id, commit_lsn, reason, detail)
values ($1, $2, $3, $4, $5, $6, $7)
on conflict (source_id, database_id, dataset_id, transaction_id, commit_lsn) do update
    set reason = excluded.reason,
        detail = excluded.detail,
        attempt_count = trellara.apply_quarantine.attempt_count + 1,
        last_seen_at = now()
"#;
