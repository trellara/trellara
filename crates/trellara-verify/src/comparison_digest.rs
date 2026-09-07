use sha2::{Digest, Sha256};

use crate::TableComparison;

pub(crate) fn comparison_evidence_sha256(comparison: &TableComparison) -> String {
    let mut hasher = Sha256::new();
    hash_line(&mut hasher, "relation", &comparison.relation.display_name());
    hash_line(
        &mut hasher,
        "target_relation",
        &comparison.target_relation.display_name(),
    );
    hash_line(
        &mut hasher,
        "relation_match",
        bool_token(comparison.relation_match),
    );
    hash_line(
        &mut hasher,
        "source_watermark_lsn",
        &comparison.source_watermark_lsn,
    );
    hash_line(
        &mut hasher,
        "target_watermark_lsn",
        &comparison.target_watermark_lsn,
    );
    hash_line(
        &mut hasher,
        "target_caught_up",
        bool_token(comparison.target_caught_up),
    );
    hash_line(
        &mut hasher,
        "source_row_count",
        &comparison.source_row_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "target_row_count",
        &comparison.target_row_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "source_checksum",
        &comparison.source_checksum.to_string(),
    );
    hash_line(
        &mut hasher,
        "target_checksum",
        &comparison.target_checksum.to_string(),
    );
    hash_line(
        &mut hasher,
        "missing_count",
        &comparison.missing_in_target_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "extra_count",
        &comparison.extra_in_target_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "mismatch_count",
        &comparison.mismatched_row_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "drift_sample_limit",
        &comparison.drift_sample_limit.to_string(),
    );
    hash_vec(&mut hasher, "missing", &comparison.missing_in_target);
    hash_vec(&mut hasher, "extra", &comparison.extra_in_target);
    hash_vec(&mut hasher, "mismatch", &comparison.mismatched_rows);
    format!("{:x}", hasher.finalize())
}

fn hash_vec(hasher: &mut Sha256, key: &str, values: &[String]) {
    for value in values {
        hash_line(hasher, key, value);
    }
}

fn hash_line(hasher: &mut Sha256, key: &str, value: &str) {
    hasher.update(key.as_bytes());
    hasher.update(b"=");
    hasher.update(value.as_bytes());
    hasher.update(b"\n");
}

fn bool_token(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}
