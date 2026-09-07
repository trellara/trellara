use crate::{CliError, Result};

pub(crate) fn checked_runtime_add(left: u64, right: u64, field: &'static str) -> Result<u64> {
    left.checked_add(right)
        .ok_or(CliError::RuntimeStatOverflow { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_runtime_add_returns_sum() {
        assert_eq!(
            checked_runtime_add(2, 3, "counter").expect("checked add"),
            5
        );
    }

    #[test]
    fn checked_runtime_add_rejects_overflow() {
        let error = checked_runtime_add(u64::MAX, 1, "counter").expect_err("overflow");

        assert!(matches!(
            error,
            CliError::RuntimeStatOverflow { field: "counter" }
        ));
    }
}
