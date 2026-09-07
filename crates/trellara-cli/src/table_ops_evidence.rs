use sha2::{Digest, Sha256};

use crate::TableVerifySummary;

pub(crate) fn validation_evidence_sha256(tables: &[TableVerifySummary]) -> String {
    let mut rows = tables
        .iter()
        .map(|table| format!("{}={}", table.relation, table.evidence_sha256))
        .collect::<Vec<_>>();
    rows.sort_unstable();

    let mut hasher = Sha256::new();
    for row in rows {
        hasher.update(row.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChecksumStatus;

    #[test]
    fn validation_evidence_digest_is_stable_for_table_order() {
        let sales = table("public.sales", "a");
        let refunds = table("public.refunds", "b");

        let first = validation_evidence_sha256(&[sales.clone(), refunds.clone()]);
        let second = validation_evidence_sha256(&[refunds, sales]);

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    fn table(relation: &str, digest_prefix: &str) -> TableVerifySummary {
        TableVerifySummary {
            relation: relation.to_string(),
            target_relation: relation.to_string(),
            relation_match: true,
            row_filter: None,
            converged: true,
            checksum_status: ChecksumStatus::Match,
            source_row_count: 1,
            target_row_count: 1,
            source_checksum: 1,
            target_checksum: 1,
            missing_in_target_count: 0,
            extra_in_target_count: 0,
            mismatched_row_count: 0,
            drift_sample_limit: 20,
            missing_in_target: Vec::new(),
            extra_in_target: Vec::new(),
            mismatched_rows: Vec::new(),
            evidence_sha256: digest_prefix.repeat(64),
            recommended_action: String::new(),
        }
    }
}
