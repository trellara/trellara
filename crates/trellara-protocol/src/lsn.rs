use std::num::ParseIntError;

use crate::ProtocolError;

pub fn format_lsn(lsn: u64) -> String {
    format!("{:X}/{:X}", lsn >> 32, lsn & 0xffff_ffff)
}

pub fn parse_lsn(lsn: &str) -> Result<u64, ProtocolError> {
    let Some((high, low)) = lsn.split_once('/') else {
        return Err(invalid_lsn(lsn, "expected HIGH/LOW hexadecimal form"));
    };
    if high.is_empty() || low.is_empty() {
        return Err(invalid_lsn(
            lsn,
            "both HIGH and LOW hexadecimal fields must be present",
        ));
    }
    let high = parse_lsn_part(lsn, high)?;
    let low = parse_lsn_part(lsn, low)?;
    if high > 0xffff_ffff {
        return Err(invalid_lsn(
            lsn,
            "HIGH hexadecimal field must fit in 32 bits",
        ));
    }
    if low > 0xffff_ffff {
        return Err(invalid_lsn(
            lsn,
            "LOW hexadecimal field must fit in 32 bits",
        ));
    }

    Ok((high << 32) | low)
}

fn parse_lsn_part(lsn: &str, value: &str) -> Result<u64, ProtocolError> {
    u64::from_str_radix(value, 16)
        .map_err(|error: ParseIntError| invalid_lsn(lsn, error.to_string()))
}

fn invalid_lsn(lsn: &str, reason: impl Into<String>) -> ProtocolError {
    ProtocolError::InvalidLsn {
        lsn: lsn.to_string(),
        reason: reason.into(),
    }
}
