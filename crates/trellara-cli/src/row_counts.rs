use crate::{CliError, Result};

pub(crate) fn u64_to_i64_count(field: &'static str, value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| CliError::CountOverflow { field, value })
}

pub(crate) fn usize_to_i64_count(field: &'static str, value: usize) -> Result<i64> {
    let value = u64::try_from(value).map_err(|_| CliError::CountOverflow {
        field,
        value: u64::MAX,
    })?;
    u64_to_i64_count(field, value)
}

pub(crate) fn i64_to_u64_count(field: &'static str, value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|_| CliError::NegativeCount { field, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_to_i64_count_rejects_overflow() {
        let error = u64_to_i64_count("copied_rows", i64::MAX as u64 + 1).expect_err("overflow");

        assert!(matches!(
            error,
            CliError::CountOverflow {
                field: "copied_rows",
                value,
            } if value == i64::MAX as u64 + 1
        ));
    }

    #[test]
    fn i64_to_u64_count_rejects_negative_values() {
        let error = i64_to_u64_count("copied_rows", -1).expect_err("negative count");

        assert!(matches!(
            error,
            CliError::NegativeCount {
                field: "copied_rows",
                value: -1,
            }
        ));
    }

    #[test]
    fn usize_to_i64_count_accepts_normal_table_counts() {
        assert_eq!(usize_to_i64_count("table_count", 42).expect("count"), 42);
    }
}
