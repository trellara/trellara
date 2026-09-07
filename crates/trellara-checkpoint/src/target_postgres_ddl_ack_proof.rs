pub use trellara_protocol::POST_DDL_DML_RELEASE_GATE as TARGET_POSTGRES_DDL_RELEASE_GATE;

const TARGET_POSTGRES_APPLIED_PREFIX: &str = "target Postgres applied ";
const TARGET_POSTGRES_APPLIED_SUFFIX: &str = " DDL statements";

pub fn target_postgres_ddl_ack_detail(applied_statements: usize) -> String {
    format!(
        "{TARGET_POSTGRES_APPLIED_PREFIX}{applied_statements}{TARGET_POSTGRES_APPLIED_SUFFIX}; release_gate={TARGET_POSTGRES_DDL_RELEASE_GATE}"
    )
}

pub fn target_postgres_ddl_ack_detail_with_digests(
    applied_statements: usize,
    plan_sha256: &str,
    statement_sha256s: &[String],
) -> String {
    target_postgres_ddl_ack_detail_with_boundary(
        applied_statements,
        plan_sha256,
        statement_sha256s,
        None,
    )
}

pub fn target_postgres_ddl_ack_detail_with_boundary(
    applied_statements: usize,
    plan_sha256: &str,
    statement_sha256s: &[String],
    barrier_lsn: Option<&str>,
) -> String {
    let barrier_lsn = barrier_lsn
        .map(|lsn| format!("; barrier_lsn={lsn}"))
        .unwrap_or_default();
    format!(
        "{}; plan_sha256={}; statement_sha256={}{}; release_gate={}",
        target_postgres_ddl_ack_detail(applied_statements)
            .split(';')
            .next()
            .expect("applied statement token"),
        plan_sha256,
        statement_sha256s.join(","),
        barrier_lsn,
        TARGET_POSTGRES_DDL_RELEASE_GATE
    )
}

pub fn target_postgres_ddl_ack_detail_is_valid(detail: &str) -> bool {
    let mut applied_statement_count = None;
    let mut plan_sha256 = None;
    let mut statement_sha256_count = None;
    let mut barrier_lsn_valid = true;
    let mut release_gate_matches = false;

    for token in detail.split(';').map(str::trim) {
        if let Some(count) = parse_target_postgres_applied_statement_count(token) {
            applied_statement_count = Some(count);
        }
        if let Some(digest) = token.strip_prefix("plan_sha256=") {
            plan_sha256 = Some(digest);
        }
        if let Some(digests) = token.strip_prefix("statement_sha256=") {
            statement_sha256_count = Some(parse_statement_digest_count(digests));
        }
        if let Some(barrier_lsn) = token.strip_prefix("barrier_lsn=") {
            barrier_lsn_valid = lsn_is_valid(barrier_lsn);
        }
        if token
            .strip_prefix("release_gate=")
            .is_some_and(|release_gate| release_gate == TARGET_POSTGRES_DDL_RELEASE_GATE)
        {
            release_gate_matches = true;
        }
    }

    let Some(count) = applied_statement_count else {
        return false;
    };
    count > 0
        && release_gate_matches
        && barrier_lsn_valid
        && plan_sha256.is_some_and(sha256_is_valid)
        && statement_sha256_count == Some(count)
}

pub fn target_postgres_ddl_ack_detail_matches_barrier(
    detail: &str,
    expected_barrier_lsn: u64,
) -> bool {
    detail
        .split(';')
        .map(str::trim)
        .filter_map(|token| token.strip_prefix("barrier_lsn="))
        .all(|barrier_lsn| crate::parse_lsn(barrier_lsn) == expected_barrier_lsn)
}

fn parse_target_postgres_applied_statement_count(token: &str) -> Option<usize> {
    token
        .strip_prefix(TARGET_POSTGRES_APPLIED_PREFIX)?
        .strip_suffix(TARGET_POSTGRES_APPLIED_SUFFIX)?
        .parse()
        .ok()
}

fn parse_statement_digest_count(value: &str) -> usize {
    value
        .split(',')
        .map(str::trim)
        .filter(|digest| sha256_is_valid(digest))
        .count()
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn lsn_is_valid(value: &str) -> bool {
    crate::lsn_shape_is_valid(value) && crate::parse_lsn(value) > 0
}

#[cfg(test)]
#[path = "target_postgres_ddl_ack_proof_tests.rs"]
mod tests;
