use super::*;

const DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[test]
fn spark_derived_views_ddl_ack_detail_accepts_template_review_and_release_gate() {
    assert!(spark_derived_views_ddl_ack_detail_is_valid(&valid_detail()));
}

#[test]
fn spark_derived_views_ddl_ack_detail_rejects_zero_view_count() {
    assert!(!spark_derived_views_ddl_ack_detail_is_valid(&format!(
        "Spark-derived views accepted 0 regenerated templates; template_digest={DIGEST}; accepted_by=platform-review; release_gate=post_ddl_dml_release"
    )));
}

#[test]
fn spark_derived_views_ddl_ack_detail_rejects_bad_template_digest() {
    assert!(!spark_derived_views_ddl_ack_detail_is_valid(
        "Spark-derived views accepted 2 regenerated templates; template_digest=not-a-digest; accepted_by=platform-review; release_gate=post_ddl_dml_release"
    ));
}

#[test]
fn spark_derived_views_ddl_ack_detail_rejects_blank_reviewer() {
    assert!(!spark_derived_views_ddl_ack_detail_is_valid(&format!(
        "Spark-derived views accepted 2 regenerated templates; template_digest={DIGEST}; accepted_by=; release_gate=post_ddl_dml_release"
    )));
}

#[test]
fn spark_derived_views_ddl_ack_detail_rejects_spoofed_release_gate_token() {
    assert!(!spark_derived_views_ddl_ack_detail_is_valid(&format!(
        "Spark-derived views accepted 2 regenerated templates; template_digest={DIGEST}; accepted_by=platform-review; previous_release_gate=post_ddl_dml_release"
    )));
}

fn valid_detail() -> String {
    format!(
        "Spark-derived views accepted 2 regenerated templates; template_digest={DIGEST}; accepted_by=platform-review; release_gate=post_ddl_dml_release"
    )
}
