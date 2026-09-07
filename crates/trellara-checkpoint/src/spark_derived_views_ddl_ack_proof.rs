pub use trellara_protocol::POST_DDL_DML_RELEASE_GATE as SPARK_DERIVED_VIEWS_DDL_RELEASE_GATE;

const SPARK_DERIVED_VIEWS_PREFIX: &str = "Spark-derived views accepted ";
const SPARK_DERIVED_VIEWS_SUFFIX: &str = " regenerated templates";

pub fn spark_derived_views_ddl_ack_detail_is_valid(detail: &str) -> bool {
    let mut view_count = None;
    let mut template_digest = None;
    let mut accepted_by = None;
    let mut release_gate_matches = false;

    for token in detail.split(';').map(str::trim) {
        if let Some(count) = parse_view_count(token) {
            view_count = Some(count);
        }
        if let Some(digest) = token.strip_prefix("template_digest=") {
            template_digest = Some(digest);
        }
        if let Some(reviewer) = token.strip_prefix("accepted_by=") {
            accepted_by = Some(reviewer);
        }
        if token
            .strip_prefix("release_gate=")
            .is_some_and(|release_gate| release_gate == SPARK_DERIVED_VIEWS_DDL_RELEASE_GATE)
        {
            release_gate_matches = true;
        }
    }

    view_count.is_some_and(|count| count > 0)
        && template_digest.is_some_and(sha256_is_valid)
        && accepted_by.is_some_and(clean_token)
        && release_gate_matches
}

fn parse_view_count(token: &str) -> Option<u32> {
    token
        .strip_prefix(SPARK_DERIVED_VIEWS_PREFIX)?
        .strip_suffix(SPARK_DERIVED_VIEWS_SUFFIX)?
        .parse()
        .ok()
}

fn clean_token(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "spark_derived_views_ddl_ack_proof_tests.rs"]
mod tests;
