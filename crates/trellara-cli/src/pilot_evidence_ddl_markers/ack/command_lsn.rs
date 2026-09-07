use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

pub(super) fn ack_lsn_reaches_barrier(ack_lsn: Option<String>, barrier_lsn: Option<&str>) -> bool {
    let Some(ack_lsn) = ack_lsn.filter(|lsn| lsn_is_valid(lsn)) else {
        return false;
    };
    let Some(barrier_lsn) = barrier_lsn.filter(|lsn| lsn_is_valid(lsn)) else {
        return false;
    };

    parse_lsn(&ack_lsn) >= parse_lsn(barrier_lsn)
}

fn lsn_is_valid(lsn: &str) -> bool {
    lsn_shape_is_valid(lsn) && parse_lsn(lsn) > 0
}
