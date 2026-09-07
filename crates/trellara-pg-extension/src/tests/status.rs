use super::*;

#[test]
fn supports_qualified_postgres_majors() {
    assert!(supports_postgres_major(15));
    assert!(supports_postgres_major(16));
    assert!(supports_postgres_major(17));
    assert!(supports_postgres_major(18));
}

#[test]
fn fails_closed_for_unbuilt_postgres_majors() {
    assert!(!supports_postgres_major(14));
    assert!(!supports_postgres_major(19));
}

#[test]
fn status_is_honest_about_foundation_readiness() {
    let status = NativeExtensionStatus::for_postgres_major(18);

    assert!(status.postgres_major_supported);
    assert!(status.sql_api_ready);
    assert!(!status.data_plane_ready);
    assert!(status.shared_preload_required_for_data_plane);
}
