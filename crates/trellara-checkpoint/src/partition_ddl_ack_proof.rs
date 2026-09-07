use crate::parse_lsn;

const RELEASE_GATE: &str = "post_ddl_dml_release";
const DETAIL_PREFIX: &str = "partition visibility reached barrier with ";
const DETAIL_PARTITION_SEPARATOR: &str = " partitions at global_durable_lsn ";
const DETAIL_APPLIED_SEPARATOR: &str = " and global_applied_lsn ";

pub fn partition_visibility_ddl_ack_detail(
    observed_partition_count: u32,
    expected_partition_count: u32,
    durable_lsn: &str,
    applied_lsn: &str,
    partition_watermark_sha256: &str,
) -> String {
    format!(
        "{DETAIL_PREFIX}{observed_partition_count}/{expected_partition_count}{DETAIL_PARTITION_SEPARATOR}{durable_lsn}{DETAIL_APPLIED_SEPARATOR}{applied_lsn}; partition_watermark_sha256={partition_watermark_sha256}; release_gate={RELEASE_GATE}",
    )
}

pub fn partition_visibility_ddl_ack_detail_is_valid(
    detail: &str,
    ack_lsn: &str,
    barrier_lsn: u64,
) -> bool {
    let mut watermark_detail_valid = false;
    let mut partition_digest_valid = false;
    let mut release_gate_matches = false;

    for token in detail.split(';').map(str::trim) {
        if let Some((durable_lsn, applied_lsn)) = parse_visibility_watermark_detail(token) {
            watermark_detail_valid = parse_lsn(durable_lsn) >= barrier_lsn
                && parse_lsn(applied_lsn) == parse_lsn(ack_lsn);
        }
        if token
            .strip_prefix("partition_watermark_sha256=")
            .is_some_and(sha256_is_valid)
        {
            partition_digest_valid = true;
        }
        if token
            .strip_prefix("release_gate=")
            .is_some_and(|release_gate| release_gate == RELEASE_GATE)
        {
            release_gate_matches = true;
        }
    }

    watermark_detail_valid && partition_digest_valid && release_gate_matches
}

fn parse_visibility_watermark_detail(token: &str) -> Option<(&str, &str)> {
    let after_prefix = token.strip_prefix(DETAIL_PREFIX)?;
    let (_partition_counts, after_partitions) =
        after_prefix.split_once(DETAIL_PARTITION_SEPARATOR)?;
    let (durable_lsn, applied_lsn) = after_partitions.split_once(DETAIL_APPLIED_SEPARATOR)?;
    Some((durable_lsn.trim(), applied_lsn.trim()))
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}
