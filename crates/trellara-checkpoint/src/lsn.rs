use trellara_protocol::Checkpoint;

pub fn parse_lsn(lsn: &str) -> u64 {
    let Some((high, low)) = lsn.split_once('/') else {
        return 0;
    };
    let high = u64::from_str_radix(high, 16).unwrap_or(0);
    let low = u64::from_str_radix(low, 16).unwrap_or(0);
    (high << 32) + low
}

pub(crate) fn format_lsn(lsn: u64) -> String {
    format!("{:X}/{:X}", lsn >> 32, lsn & 0xffff_ffff)
}

pub fn lsn_shape_is_valid(lsn: &str) -> bool {
    let Some((high, low)) = lsn.split_once('/') else {
        return false;
    };
    !high.is_empty() && !low.is_empty() && lsn_part_is_valid(high) && lsn_part_is_valid(low)
}

fn lsn_part_is_valid(value: &str) -> bool {
    u64::from_str_radix(value, 16).is_ok_and(|parsed| parsed <= u32::MAX as u64)
}

pub(crate) fn merge_checkpoint(existing: &Checkpoint, incoming: Checkpoint) -> Checkpoint {
    Checkpoint {
        source_id: incoming.source_id,
        dataset_id: incoming.dataset_id,
        last_seen_lsn: max_lsn_string(&existing.last_seen_lsn, &incoming.last_seen_lsn),
        last_durable_lsn: max_lsn_string(&existing.last_durable_lsn, &incoming.last_durable_lsn),
        last_applied_lsn: max_lsn_string(&existing.last_applied_lsn, &incoming.last_applied_lsn),
    }
}

fn max_lsn_string(left: &str, right: &str) -> String {
    if parse_lsn(left) >= parse_lsn(right) {
        left.to_string()
    } else {
        right.to_string()
    }
}
