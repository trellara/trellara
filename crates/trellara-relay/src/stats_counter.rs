use crate::{RelayError, Result};

pub(crate) fn checked_relay_stat_add(left: u64, right: u64, field: &'static str) -> Result<u64> {
    left.checked_add(right)
        .ok_or(RelayError::StatOverflow { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_relay_stat_add_returns_sum() {
        assert_eq!(
            checked_relay_stat_add(8, 13, "counter").expect("checked add"),
            21
        );
    }

    #[test]
    fn checked_relay_stat_add_rejects_overflow() {
        let error = checked_relay_stat_add(u64::MAX, 1, "counter").expect_err("overflow");

        assert!(matches!(
            error,
            RelayError::StatOverflow { field: "counter" }
        ));
    }
}
